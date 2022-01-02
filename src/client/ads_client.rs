use anyhow::anyhow;
use anyhow::{Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::hash_map;
use std::collections::HashMap;
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::net::{Ipv4Addr, SocketAddr};
use std::result;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::ads_services::system_services::*;
use crate::client::plc_types::Var;
use crate::client::read::AdsReader;
use crate::error::AdsError;
use crate::proto::ads_state::*;
use crate::proto::ads_transition_mode::AdsTransMode;
use crate::proto::ams_address::{AmsAddress, AmsNetId};
use crate::proto::ams_header::*;
use crate::proto::command_id::CommandID;
use crate::proto::proto_traits::*;
use crate::proto::request::*;
use crate::proto::response::*;
use crate::proto::state_flags::*;
use crate::proto::sumup::sumup_request::{
    SumupReadRequest, SumupReadWriteRequest, SumupWriteRequest,
};
use crate::proto::sumup::sumup_response::{SumupReadResponse, SumupWriteResponse};

use std::convert::TryInto;

/// UDO ADS-Protocol port dicovery
pub const ADS_UDP_SERVER_PORT: u16 = 48899;
/// TCP ADS-Protocol port not secured
pub const ADS_TCP_SERVER_PORT: u16 = 48898;
/// ADS-Protocol port secured
pub const ADS_SECURE_TCP_SERVER_PORT: u16 = 8016;
//Tcp Header size without response data
pub const AMS_HEADER_SIZE: usize = 38;

pub type ClientResult<T> = result::Result<T, anyhow::Error>;
type SymHandle = u32;
type ReaderTX = Sender<Result<Response, AdsError>>;
type ReaderNotifyTX = Sender<Result<AdsNotificationStream, AdsError>>;

#[derive(Debug)]
pub struct Client {
    route: Ipv4Addr,
    ams_targed_address: AmsAddress,
    ams_source_address: AmsAddress,
    stream: Option<TcpStream>,
    sym_handle: HashMap<String, SymHandle>,
    notification_channels: Arc<Mutex<HashMap<u32, ReaderTX>>>,
    device_notification_stream_channels: Arc<Mutex<HashMap<u32, ReaderNotifyTX>>>,    
    notification_handles: HashMap<String, u32>,
}

///Connect to a remote ads device and read, write data,
///add and receive notifications or read device information.
impl Client {
    ///Crates new client which is not yet connected. Use connect() after creating.
    pub fn new(route: Ipv4Addr, ams_targed_address: AmsAddress) -> Self {
        Client {
            route,
            ams_targed_address,
            ams_source_address: AmsAddress::new(AmsNetId::from([0, 0, 0, 0, 0, 0]), 0),
            stream: None,
            sym_handle: HashMap::new(),            
            notification_channels: Arc::new(Mutex::new(HashMap::new())),
            device_notification_stream_channels: Arc::new(Mutex::new(HashMap::new())),            
            notification_handles: HashMap::new(),
        }
    }

    ///Connect to remote device
    pub fn connect(&mut self) -> ClientResult<()> {
        if self.is_connected() {
            return Ok(());
        }

        let socket_addr = SocketAddr::from((self.route, ADS_TCP_SERVER_PORT));
        let stream = TcpStream::connect(socket_addr)?;
        stream.set_nodelay(true)?;        
        stream.set_write_timeout(Some(Duration::from_millis(1000)));
        self.ams_source_address            
            .update_from_socket_addr(stream.local_addr()?.to_string().as_str())?;
        self.stream = Some(stream);
        self.run_reader_thread()?;
        Ok(())
    }

    pub fn connect_secure(&mut self) -> ClientResult<()> {
        unimplemented!()
    }

    ///A tcp stream is available -> stream is not None
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    ///Sends the request to a remote device
    fn request(&mut self, request: Request, invoke_id: u32) -> ClientResult<usize> {
        let mut buffer = Vec::new();
        self.create_payload(request, StateFlags::req_default(), invoke_id, &mut buffer)?;
        self.stream_write(&mut buffer)
    }

    ///Write the request data to the byte buffer for sending
    fn create_payload(
        &mut self,
        request: Request,
        state_flag: StateFlags,
        invoke_id: u32,
        mut buffer: &mut Vec<u8>,
    ) -> ClientResult<()> {
        let ams_header = AmsHeader::new(
            self.ams_targed_address.clone(),
            self.ams_source_address.clone(),
            state_flag,
            invoke_id,
            request,
        );
        AmsTcpHeader::from(ams_header).write_to(&mut buffer)?;
        Ok(())
    }

    ///Writes request to remote device
    fn stream_write(&mut self, buffer: &mut [u8]) -> ClientResult<usize> {
        if let Some(s) = &mut self.stream {
            return Ok(s.write(buffer)?);
        }
        Err(anyhow!(AdsError::ErrPortNotConnected))
    }

    //test
    fn run_reader_thread(&mut self) -> ClientResult<()> {        
        let mut reader: AdsReader;

        if let Some(s) = &mut self.stream {
            let mut stream = s.try_clone()?;
            reader = AdsReader::new(stream);
        } else {
            return Err(anyhow!(AdsError::ErrPortNotConnected))            
        }

        let notificatino_channels = Arc::clone(&self.notification_channels);
        let notification_stream_channels = Arc::clone(&self.device_notification_stream_channels);

        let _: JoinHandle<ClientResult<()>> = thread::spawn(move || {
            let mut cancel: bool = false;
            let mut buf: Vec<u8> = vec![0; AMS_HEADER_SIZE];
            let mut tcp_ams_header: AmsTcpHeader;
            loop {
                tcp_ams_header = match reader.read_response() {
                    Ok(a) => a,
                    Err(e) => {
                        let mut channels;
                        match notification_stream_channels.lock() {
                            Ok(c) => channels = c,
                            Err(_) => panic!("Failed to get lock!"),
                        };

                        for (handle, sender) in channels.iter() {
                            sender.send(Err(AdsError::AdsErrClientW32Error));
                        }                       
                        continue;
                    }
                };

                match tcp_ams_header.command_id() {
                    CommandID::DeviceNotification => {
                        let mut channels = match notification_stream_channels.lock() {
                            Ok(c) => c,
                            Err(_) => panic!("Failed to get lock!"),
                        };

                        let stream: AdsNotificationStream =
                            tcp_ams_header.response()?.try_into()?;
                        let mut handle = 0;
                        for header in stream.ads_stamp_headers {
                            for sample in header.notification_samples {
                                handle = sample.notification_handle;
                                if let Some(sender) = channels.get(&handle) {
                                    if tcp_ams_header.ads_error() == &AdsError::ErrNoError {
                                        let response: AdsNotificationStream =
                                            tcp_ams_header.response()?.try_into()?;
                                        sender.send(Ok(response));
                                    } else {
                                        sender.send(Err(tcp_ams_header.ads_error().clone()));
                                    }
                                } else {
                                    println!(
                                        "No sender for invoke id {:?} found ....{:?}",
                                        &tcp_ams_header.invoke_id(),
                                        &tcp_ams_header.command_id()
                                    )
                                }
                            }
                        }                        
                    }
                    _ => {
                        let mut channels = match notificatino_channels.lock() {
                            Ok(c) => c,
                            Err(_) => panic!("Failed to get lock!"),
                        };

                        if let Some(sender) = channels.get(&tcp_ams_header.invoke_id()) {
                            if tcp_ams_header.ads_error() == &AdsError::ErrNoError {
                                let response = tcp_ams_header.response()?;
                                sender.send(Ok(response));
                            } else {
                                sender.send(Err(tcp_ams_header.ads_error().clone()));
                            }
                        } else {
                            println!(
                                "No sender for invoke id {:?} found ....{:?}...",
                                &tcp_ams_header.invoke_id(),
                                &tcp_ams_header.command_id()
                            )
                        }
                    }
                }                
            }            
            Ok(())
        });
        Ok(())
    }

    ///Creates an channel to receive the requested data
    fn create_response_channel(
        &mut self,
        invoke_id: u32,
    ) -> ClientResult<Receiver<Result<Response, AdsError>>> {
        let mut channels = match self.notification_channels.lock() {
            Ok(c) => c,
            Err(_) => panic!("Failed to get lock!"),
        };

        let (tx, rx) = channel::<Result<Response, AdsError>>();
        channels.insert(invoke_id, tx);
        Ok(rx)
    }

    ///Creates a channel to receive ads notifications
    fn create_device_notification_channel(
        &mut self,
        handle: u32,
    ) -> ClientResult<Receiver<Result<AdsNotificationStream, AdsError>>> {
        let mut channels = match self.device_notification_stream_channels.lock() {
            Ok(c) => c,
            Err(_) => panic!("Failed to get lock!"),
        };

        let (tx, rx) = channel::<Result<AdsNotificationStream, AdsError>>();
        channels.insert(handle, tx);
        Ok(rx)
    }

    ///Request handle for a single variable
    pub fn get_symhandle(&mut self, var: &Var, invoke_id: u32) -> ClientResult<u32> {
        if self.sym_handle.contains_key(&var.name) {
            if let Some(handle) = self.sym_handle.get(&var.name) {
                return Ok(*handle);
            }
        }

        let request = Request::ReadWrite(ReadWriteRequest::new(
            GET_SYMHANDLE_BY_NAME.index_group,
            GET_SYMHANDLE_BY_NAME.index_offset_start,
            4, //allways u32 for get_symhandle
            var.name.as_bytes().to_vec(),
        ));
        let rx = self.create_response_channel(invoke_id)?;
        self.request(request, invoke_id)?;
        let response: ReadWriteResponse = rx.recv()??.try_into()?; //blocking call to the channel rx
        Client::check_ads_error(&response.result)?;
        let handle = response.data.as_slice().read_u32::<LittleEndian>()?;
        self.sym_handle.insert(var.name.to_string(), handle);
        Ok(handle)
    }

    ///Request handles for multiple variables.
    pub fn sumup_get_symhandle(&mut self, var_list: &[Var], invoke_id: u32) -> ClientResult<bool> {
        //Check for already available handles
        let mut request_handle_list: Vec<ReadWriteRequest> = Vec::new();
        let remaining_var_list = self.check_available_handles(var_list, &mut request_handle_list);
        //Request not available handles
        let mut data_buf: Vec<u8> = Vec::new();
        let request_count = request_handle_list.len() as u32;
        SumupReadWriteRequest::new(request_handle_list).write_to(&mut data_buf)?;
        let request = Request::ReadWrite(ReadWriteRequest::new(
            ADSIGRP_SUMUP_READWRITE.index_group,
            ADSIGRP_SUMUP_READWRITE.index_offset_start + request_count,
            (request_count * 12),
            data_buf,
        ));
        let rx = self.create_response_channel(invoke_id)?;
        self.request(request, invoke_id)?;
        let read_write_response: ReadWriteResponse = rx.recv()??.try_into()?; //blocking call to the channel rx
        Client::check_ads_error(&read_write_response.result)?;
        let sumup_response: SumupReadResponse =
            SumupReadResponse::read_from(&mut read_write_response.data.as_slice())?;

        self.collect_handles(&remaining_var_list, &sumup_response);
        Ok(true)
    }

    ///check if some of the requested var handles are already available.
    /// Returns a list with variables which have no handle available yet.
    fn check_available_handles(
        &self,
        var_list: &[Var],
        request_handle_list: &mut Vec<ReadWriteRequest>,
    ) -> Vec<Var> {
        let mut remaining_var_list: Vec<Var> = Vec::new();
        for var in var_list {
            if !self.sym_handle.contains_key(&var.name) {
                let request = ReadWriteRequest::new(
                    GET_SYMHANDLE_BY_NAME.index_group,
                    GET_SYMHANDLE_BY_NAME.index_offset_start,
                    4, //u32 for GET_SYMHANDLE_BY_NAME
                    var.name.as_bytes().to_vec(),
                );
                request_handle_list.push(request);
                remaining_var_list.push(var.clone());
            }
        }
        remaining_var_list
    }

    ///Collects the requested handles from the response and stores them inside the Client
    fn collect_handles(
        &mut self,
        var_list: &[Var],
        sumup_response: &SumupReadResponse,
    ) -> ClientResult<()> {
        for (n, var) in var_list.iter().enumerate() {
            Client::check_ads_error(&sumup_response.read_responses[n].result)?;
            self.sym_handle.insert(
                var.name.clone(),
                sumup_response.read_responses[n]
                    .data
                    .as_slice()
                    .read_u32::<LittleEndian>()?,
            );
        }
        Ok(())
    }   

    ///Read a value of a single var from a remote device
    pub fn read_by_name(&mut self, var: &Var, invoke_id: u32) -> ClientResult<Vec<u8>> {
        if !self.sym_handle.contains_key(&var.name) {
            return Err(anyhow!("Symhandle for {:?} missing", var.name));
        }

        if let Some(handle) = self.sym_handle.get(&var.name) {
            let request = Request::Read(ReadRequest::new(
                READ_WRITE_SYMVAL_BY_HANDLE.index_group,
                *handle,
                var.plc_type.size() as u32,
            ));
            self.request(request, invoke_id)?;
            let mut response = self.create_response_channel(invoke_id)?.recv()?;
            let response: ReadResponse = response?.try_into()?;

            match Client::check_ads_error(&response.result) {
                Ok(()) => Ok(response.data),
                Err(e) => {
                    self.is_handle_invalid(&e);
                    Err(anyhow!(e))
                }
            }
        } else {
            Err(anyhow!("No symHandle"))
        }
    }

    ///Remove a handle if remote device returns error AdsErrDeviceSymbolVersionInvalid
    ///Returns true if handle is invalid else false
    fn is_handle_invalid(&mut self, ads_error: &AdsError) -> bool {
        if ads_error == &AdsError::AdsErrDeviceSymbolVersionInvalid {
            self.sym_handle.clear();
            return true;
        }
        false
    }

    ///Read values of multiple variables in a single call.
    pub fn sumup_read_by_name(
        &mut self,
        var_list: &[Var],
        invoke_id: u32,
    ) -> ClientResult<HashMap<String, Vec<u8>>> {
        self.handles_available(var_list)?; // Fails if a handles is missing.
        let mut result: HashMap<String, Vec<u8>> = HashMap::new();
        let request = self.create_read_request(self.create_read_request_list(var_list)?)?;
        let response = self.create_response_channel(invoke_id)?;
        self.request(request, invoke_id);
        let response: ReadWriteResponse = response.recv()??.try_into()?;
        Client::check_ads_error(&response.result)?;
        let mut read_values = SumupReadResponse::read_from(&mut response.data.as_slice())?;

        for (n, var) in var_list.iter().enumerate() {
            self.is_handle_invalid(&read_values.read_responses[n].result); //remove handles from list if invalid            
            result.insert(var.name.clone(), read_values.read_responses[n].data.clone());
            //ToDo find a way without clone for data.
        }
        Ok(result)
    }

    fn handles_available(&self, var_list: &[Var]) -> ClientResult<()> {
        //check if handles available
        for var in var_list {
            if !self.sym_handle.contains_key(&var.name) {
                return Err(anyhow!("Symhandle for {:?} missing", var.name));
            }
        }
        Ok(())
    }

    ///Creates a vec of read requests
    fn create_read_request_list(&self, var_list: &[Var]) -> ClientResult<Vec<ReadRequest>> {
        let mut result: Vec<ReadRequest> = Vec::new();
        for var in var_list {
            if let Some(handle) = self.sym_handle.get(&var.name) {
                result.push(ReadRequest::new(
                    READ_WRITE_SYMVAL_BY_HANDLE.index_group,
                    *handle,
                    var.plc_type.size() as u32,
                ));
            } else {
                return Err(anyhow!("Symhandle for {:?} missing", var.name));
            }
        }
        Ok(result)
    }

    ///Create a read request from a vec of ReadRequests as data (sumup)
    fn create_read_request(&self, requests: Vec<ReadRequest>) -> ClientResult<Request> {
        let mut buf: Vec<u8> = Vec::new();
        let sumup = SumupReadRequest::new(requests);
        sumup.write_to(&mut buf)?;
        let read_request = Request::ReadWrite(ReadWriteRequest::new(
            ADSIGRP_SUMUP_READEX.index_group,
            sumup.request_count(),
            sumup.expected_response_len(),
            buf,
        ));
        Ok(read_request)
    }

    pub fn read_device_info(&mut self, invoke_id: u32) -> ClientResult<ReadDeviceInfoResponse> {
        let rx = self.create_response_channel(invoke_id)?;
        self.request(
            Request::ReadDeviceInfo(ReadDeviceInfoRequest::new()),
            invoke_id,
        );
        let response: ReadDeviceInfoResponse = rx.recv()??.try_into()?;
        Client::check_ads_error(&response.result)?;
        Ok(response)
    }

    pub fn read_state(&mut self, invoke_id: u32) -> ClientResult<ReadStateResponse> {
        let rx = self.create_response_channel(invoke_id)?;
        self.request(Request::ReadState(ReadStateRequest::new()), invoke_id);
        let response: ReadStateResponse = rx.recv()??.try_into()?;
        Client::check_ads_error(&response.result)?;
        Ok(response)
    }

    ///Write data to a single var
    pub fn write_by_name(&mut self, var: &Var, invoke_id: u32, data: Vec<u8>) -> ClientResult<()> {
        if !self.sym_handle.contains_key(&var.name) {
            return Err(anyhow!("Symhandle for {:?} missing", var.name));
        }

        if let Some(handle) = self.sym_handle.get(&var.name) {
            let request = Request::Write(WriteRequest::new(
                READ_WRITE_SYMVAL_BY_HANDLE.index_group,
                *handle,
                data,
            ));
            let rx = self.create_response_channel(invoke_id)?;
            self.request(request, invoke_id)?;
            let response: WriteResponse = rx.recv()??.try_into()?;
            //Delete handles if AdsError::AdsErrDeviceSymbolVersionInvalid in response data
            match Client::check_ads_error(&response.result) {
                Ok(()) => Ok(()),
                Err(e) => {
                    self.is_handle_invalid(&e);
                    Err(anyhow!(e))
                }
            }
        } else {
            return Err(anyhow!("Symhandle for {:?} missing", var.name));
        }
    }

    ///write data to multiple variables in a single call
    pub fn sumup_write_by_name(
        &mut self,
        var_list: &[Var],
        invoke_id: u32,
    ) -> ClientResult<HashMap<String, AdsError>> {
        let mut result: HashMap<String, AdsError> = HashMap::new();
        self.handles_available(var_list)?;
        let request = self.create_write_request(self.create_write_request_list(var_list)?)?;
        let rx = self.create_response_channel(invoke_id)?;
        self.request(request, invoke_id);
        let response: ReadWriteResponse = rx.recv()??.try_into()?;
        Client::check_ads_error(&response.result)?;
        let mut response = SumupWriteResponse::read_from(&mut response.data.as_slice())?;

        for (n, var) in var_list.iter().enumerate() {
            result.insert(var.name.clone(), response.write_responses[n].result.clone());
            //ToDo find a way without clone for data.
        }
        Ok(result)
    }

    ///Creates a vec with write requests
    fn create_write_request_list(&self, var_list: &[Var]) -> ClientResult<Vec<WriteRequest>> {
        let mut result: Vec<WriteRequest> = Vec::new();
        for var in var_list {
            if let Some(handle) = self.sym_handle.get(&var.name) {
                result.push(WriteRequest::new(
                    READ_WRITE_SYMVAL_BY_HANDLE.index_group,
                    *handle,
                    var.data.clone(),
                ));
            } else {
                return Err(anyhow!("Symhandle for {:?} missing", var.name));
            }
        }
        Ok(result)
    }

    ///Creates a request (sumup) from a Vec<WreiteRequest>
    fn create_write_request(&self, requests: Vec<WriteRequest>) -> ClientResult<Request> {
        let mut buf: Vec<u8> = Vec::new();
        let sumup = SumupWriteRequest::new(requests);
        sumup.write_to(&mut buf)?;
        let read_request = Request::ReadWrite(ReadWriteRequest::new(
            ADSIGRP_SUMUP_WRITE.index_group,
            sumup.request_count(),
            sumup.expected_response_len(),
            buf,
        ));
        Ok(read_request)
    }

    ///Writre an AdsState/DeviceState to the remote device
    ///For example stop the Ads device
    pub fn write_control(
        &mut self,
        new_ads_state: AdsState,
        device_state: u16,
        invoke_id: u32,
    ) -> ClientResult<()> {
        self.request(
            Request::WriteControl(WriteControlRequest::new(
                new_ads_state,
                device_state,
                0,
                Vec::with_capacity(0),
            )),
            invoke_id,
        );
        let rx = self.create_response_channel(invoke_id)?;
        let response: WriteControlResponse = rx.recv()??.try_into()?;
        Client::check_ads_error(&response.result)?;
        Ok(())
    }

    /// Get notifications from the remote device. Define with with AdsTransMode the behaviour
    /// Returns the receiver part of a mpsc channel which can be polled to reviced the notifications.
    pub fn add_device_notification(
        &mut self,
        var: &Var,
        trans_mode: AdsTransMode,
        max_delay: u32,
        cycle_time: u32,
        invoke_id: u32,
    ) -> ClientResult<Receiver<Result<AdsNotificationStream, AdsError>>> {
        let mut handle_val = self.get_symhandle(var, invoke_id)?;
        let rx = self.create_response_channel(invoke_id)?;
        self.request(
            Request::AddDeviceNotification(AddDeviceNotificationRequest::new(
                READ_WRITE_SYMVAL_BY_HANDLE.index_group,
                handle_val,
                var.plc_type.size() as u32,
                trans_mode,
                max_delay,
                cycle_time,
            )),
            invoke_id,
        );

        let response: AddDeviceNotificationResponse = rx.recv()??.try_into()?;
        Client::check_ads_error(&response.result)?;
        let rx = self.create_device_notification_channel(response.notification_handle)?;
        self.notification_handles
            .insert(var.name.clone(), response.notification_handle);
        Ok(rx)
    }

    ///Stop receiving device notifications
    pub fn delete_device_notification(&mut self, var: &Var, invoke_id: u32) -> ClientResult<()> {
        let rx = self.create_response_channel(invoke_id)?;
        let mut handle = 0;
        if let Some(handle_val) = self.notification_handles.get(&var.name) {
            handle = *handle_val;
            self.request(
                Request::DeleteDeviceNotification(DeleteDeviceNotificationRequest::new(handle)),
                invoke_id,
            );
        } else {
            anyhow!("No handle for var {:?}", var);
        }

        let response: DeleteDeviceNotificationResponse = rx.recv()??.try_into()?;
        Client::check_ads_error(&response.result)?;

        let mut channels = match self.device_notification_stream_channels.lock() {
            Ok(c) => c,
            Err(_) => panic!("Failed to get lock!"),
        };
        channels.remove(&handle);
        Ok(())
    }

    ///Check if the result contains an error
    fn check_ads_error(ads_error: &AdsError) -> Result<(), AdsError> {
        if ads_error != &AdsError::ErrNoError {
            return Err(ads_error.clone());
        }
        Ok(())
    }
}
