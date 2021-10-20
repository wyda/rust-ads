use anyhow::anyhow;
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

use crate::ads_services::system_services::*;
use crate::client::plc_types::{SymHandle, Var};
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

pub struct Connection<'a> {
    route: Ipv4Addr,
    ams_targed_address: AmsAddress,
    ams_source_address: AmsAddress,
    stream: Option<TcpStream>,
    sym_handle: HashMap<&'a str, SymHandle>,
    read_thread: Option<JoinHandle<ClientResult<()>>>,
    notification_channels: Arc<Mutex<HashMap<u32, Sender<Result<Response, AdsError>>>>>,
    device_notification_stream_channels:
        Arc<Mutex<HashMap<u32, Sender<Result<AdsNotificationStream, AdsError>>>>>,
    tx_thread_cancel: Option<Sender<bool>>,
    notification_handles: HashMap<&'a str, u32>,
}

impl<'a> Connection<'a> {
    pub fn new(route: Option<Ipv4Addr>, ams_targed_address: AmsAddress) -> Self {
        let ip = match route {
            Some(r) => r,
            None => Ipv4Addr::new(127, 0, 0, 1),
        };

        Connection {
            route: ip,
            ams_targed_address,
            ams_source_address: AmsAddress::new(AmsNetId::from([0, 0, 0, 0, 0, 0]), 0),
            stream: None,
            sym_handle: HashMap::new(),
            read_thread: None,
            notification_channels: Arc::new(Mutex::new(HashMap::new())),
            device_notification_stream_channels: Arc::new(Mutex::new(HashMap::new())),
            tx_thread_cancel: None,
            notification_handles: HashMap::new(),
        }
    }

    pub fn connect(&mut self) -> ClientResult<()> {
        if self.is_connected() {
            return Ok(());
        }

        let socket_addr = SocketAddr::from((self.route, ADS_TCP_SERVER_PORT));
        self.stream = Some(TcpStream::connect(socket_addr)?);
        if let Some(s) = &self.stream {
            self.ams_source_address
                .update_from_socket_addr(s.local_addr()?.to_string().as_str())?;
        }
        self.run_reader_thread()
    }

    pub fn connect_secure(&mut self) -> ClientResult<()> {
        unimplemented!()
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub fn request(&mut self, request: Request, invoke_id: u32) -> ClientResult<usize> {
        let mut buffer = Vec::new();
        self.create_payload(request, StateFlags::req_default(), invoke_id, &mut buffer)?;
        self.stream_write(&mut buffer)
    }

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
        let ams_tcp_header = AmsTcpHeader::from(ams_header);
        ams_tcp_header.write_to(&mut buffer)?;
        Ok(())
    }

    fn stream_write(&mut self, buffer: &mut [u8]) -> ClientResult<usize> {
        if let Some(s) = &mut self.stream {
            return Ok(s.write(buffer)?);
        }
        Err(anyhow!(AdsError::ErrPortNotConnected))
    }

    //test
    fn run_reader_thread(&mut self) -> ClientResult<()> {
        let (tx, rx) = channel::<bool>();
        self.tx_thread_cancel = Some(tx);
        let rx_thread_cancel = rx;
        let mut reader: AdsReader;

        if let Some(s) = &mut self.stream {
            let mut stream = s.try_clone()?;
            reader = AdsReader::new(stream);
        } else {
            panic!("No tcp stream available"); //ToDo
        }

        let notificatino_channels = Arc::clone(&self.notification_channels);
        let notification_stream_channels = Arc::clone(&self.device_notification_stream_channels);

        self.read_thread = Some(thread::spawn(move || {
            let mut cancel: bool = false;
            let mut buf: Vec<u8> = vec![0; AMS_HEADER_SIZE];
            let mut tcp_ams_header: AmsTcpHeader;
            while (!cancel) {
                tcp_ams_header = reader.read_response()?;
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
                            }
                        }

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
                                "No sender for invoke id {:?} found ....{:?}",
                                &tcp_ams_header.invoke_id(),
                                &tcp_ams_header.command_id()
                            )
                        }
                    }
                }

                if let Ok(c) = rx_thread_cancel.try_recv() {
                    cancel = c;
                }
            }
            Ok(())
        }));
        Ok(())
    }

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

    fn read_device_notification_response(
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

    //trial
    pub fn get_symhandle(&mut self, var: &Var<'a>, invoke_id: u32) -> ClientResult<u32> {
        if self.sym_handle.contains_key(var.name) {
            if let Some(handle) = self.sym_handle.get(var.name) {
                return Ok(handle.handle);
            }
        }

        let request = Request::ReadWrite(ReadWriteRequest::new(
            GET_SYMHANDLE_BY_NAME.index_group,
            GET_SYMHANDLE_BY_NAME.index_offset_start,
            4, //allways u32 for get_symhandle
            var.name.len() as u32,
            var.name.as_bytes().to_vec(),
        ));
        self.request(request, invoke_id)?;
        let mut response: ReadWriteResponse = self
            .create_response_channel(invoke_id)?
            .recv()??
            .try_into()?; //blocking call to the channel rx
        Connection::check_ads_error(&response.result)?;
        let raw_handle = response.data.as_slice().read_u32::<LittleEndian>()?;
        let handle = SymHandle::new(raw_handle, var.plc_type.clone());
        self.sym_handle.insert(var.name, handle);
        Ok(raw_handle)
    }

    //trial
    pub fn read_by_name(&mut self, var: &Var<'a>, invoke_id: u32) -> ClientResult<Vec<u8>> {
        if !self.sym_handle.contains_key(&var.name) {
            self.get_symhandle(var, invoke_id)?;
        }

        if let Some(handle) = self.sym_handle.get(var.name) {
            let request = Request::Read(ReadRequest::new(
                READ_WRITE_SYMVAL_BY_HANDLE.index_group,
                handle.handle,
                var.plc_type.size() as u32,
            ));
            self.request(request, invoke_id)?;
            let mut response = self.create_response_channel(invoke_id)?.recv()?;
            let response: ReadResponse = response?.try_into()?;
            //Delete handles if AdsError::AdsErrDeviceSymbolVersionInvalid
            match Connection::check_ads_error(&response.result) {
                Ok(()) => Ok(response.data),
                Err(e) => {
                    if e == AdsError::AdsErrDeviceSymbolVersionInvalid {
                        self.sym_handle.clear();
                    }
                    Err(anyhow!(e))
                }
            }
        } else {
            Err(anyhow!("No symHandle"))
        }
    }

    pub fn read_device_info(&mut self, invoke_id: u32) -> ClientResult<ReadDeviceInfoResponse> {
        let rx = self.create_response_channel(invoke_id)?;
        self.request(
            Request::ReadDeviceInfo(ReadDeviceInfoRequest::new()),
            invoke_id,
        );
        let response: ReadDeviceInfoResponse = rx.recv()??.try_into()?;
        Connection::check_ads_error(&response.result)?;
        Ok(response)
    }

    pub fn read_state(&mut self, invoke_id: u32) -> ClientResult<ReadStateResponse> {
        let rx = self.create_response_channel(invoke_id)?;
        self.request(Request::ReadState(ReadStateRequest::new()), invoke_id);
        let response: ReadStateResponse = rx.recv()??.try_into()?;
        Connection::check_ads_error(&response.result)?;
        Ok(response)
    }

    pub fn write_by_name(
        &mut self,
        var: &Var<'a>,
        invoke_id: u32,
        data: Vec<u8>,
    ) -> ClientResult<()> {
        if !self.sym_handle.contains_key(&var.name) {
            self.get_symhandle(var, invoke_id)?;
        }

        if let Some(handle) = self.sym_handle.get(var.name) {
            let request = Request::Write(WriteRequest::new(
                READ_WRITE_SYMVAL_BY_HANDLE.index_group,
                handle.handle,
                var.plc_type.size() as u32,
                data,
            ));
            self.request(request, invoke_id)?;
            let response = self.create_response_channel(invoke_id)?.recv()?;
            let response: WriteResponse = response?.try_into()?;
            //Delete handles if AdsError::AdsErrDeviceSymbolVersionInvalid in response data
            match Connection::check_ads_error(&response.result) {
                Ok(()) => Ok(()),
                Err(e) => {
                    if e == AdsError::AdsErrDeviceSymbolVersionInvalid {
                        self.sym_handle.clear();
                    }
                    Err(anyhow!(e))
                }
            }
        } else {
            Err(anyhow!("No symHandle"))
        }
    }

    //trial
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
        let mut response = self.create_response_channel(invoke_id)?.recv()?;
        let response: WriteControlResponse = response?.try_into()?;
        Connection::check_ads_error(&response.result)?;
        Ok(())
    }

    pub fn add_device_notification(
        &mut self,
        var: &Var<'a>,
        trans_mode: AdsTransMode,
        max_delay: u32,
        cycle_time: u32,
        invoke_id: u32,
    ) -> ClientResult<Receiver<Result<AdsNotificationStream, AdsError>>> {
        let mut handle_val = self.get_symhandle(var, invoke_id)?;
        let mut response_rx = self.create_response_channel(invoke_id)?;

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

        let response: AddDeviceNotificationResponse = response_rx.recv()??.try_into()?;
        Connection::check_ads_error(&response.result)?;
        self.notification_handles
            .insert(var.name, response.notification_handle);

        let rx = self.read_device_notification_response(response.notification_handle)?;
        Ok(rx)
    }

    pub fn delete_device_notification(&mut self, var: &Var, invoke_id: u32) -> ClientResult<()> {
        let response_rx = self.create_response_channel(invoke_id)?;
        if let Some(handle_val) = self.notification_handles.get(var.name) {
            let handle = *handle_val;
            self.request(
                Request::DeleteDeviceNotification(DeleteDeviceNotificationRequest::new(handle)),
                invoke_id,
            );
        }
        let response: DeleteDeviceNotificationResponse = response_rx.recv()??.try_into()?;
        Connection::check_ads_error(&response.result)?;
        Ok(())
    }

    fn check_ads_error(ads_error: &AdsError) -> Result<(), AdsError> {
        if ads_error != &AdsError::ErrNoError {
            return Err(ads_error.clone());
        }
        Ok(())
    }
}
