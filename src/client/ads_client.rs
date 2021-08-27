use anyhow::{anyhow, Error};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::result;

use crate::ads_services::system_services::*;
use crate::client::plc_types::{PlcTypes, SymHandle, Var};
use crate::error::AdsError;
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

pub type Result<T> = result::Result<T, anyhow::Error>;

pub struct Connection<'a> {
    route: Ipv4Addr,
    ams_targed_address: AmsAddress,
    ams_source_address: AmsAddress,
    stream: Option<TcpStream>,
    sym_handle: HashMap<&'a str, SymHandle>,
}

impl<'a> Connection<'a> {
    pub fn new(
        route: Option<Ipv4Addr>, //what else is needed to connect to a remote device?
        ams_targed_address: AmsAddress,
    ) -> Self {
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
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        if self.is_connected() {
            return Ok(());
        }

        let socket_addr = SocketAddr::from((self.route, ADS_TCP_SERVER_PORT));
        self.stream = Some(TcpStream::connect(socket_addr)?);
        if let Some(s) = &self.stream {
            self.ams_source_address
                .update_from_socket_addr(s.local_addr()?.to_string().as_str());
        }
        Ok(())
    }

    pub fn connect_secure(&mut self) -> Result<()> {
        unimplemented!()
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub fn request(&mut self, request: Request, invoke_id: u32) -> Result<usize> {
        let mut buffer = Vec::new();
        self.create_payload(request, StateFlags::req_default(), invoke_id, &mut buffer);
        self.stream_write(&mut buffer)
    }

    fn create_payload(
        &mut self,
        request: Request,
        state_flag: StateFlags,
        invoke_id: u32,
        mut buffer: &mut Vec<u8>,
    ) {
        let ams_header = AmsHeader::new(
            self.ams_targed_address.clone(),
            self.ams_source_address.clone(),
            state_flag,
            invoke_id,
            request,
        );
        let ams_tcp_header = AmsTcpHeader::from(ams_header);
        ams_tcp_header.write_to(&mut buffer);
    }

    fn stream_write(&mut self, mut buffer: &mut Vec<u8>) -> Result<usize> {
        if let Some(s) = &mut self.stream {
            return Ok(s.write(buffer)?);
        }
        Err(anyhow!(AdsError::ErrPortNotConnected))
    }

    pub fn read_response(&mut self) -> Result<AmsTcpHeader> {
        let mut buf = vec![0; AMS_HEADER_SIZE];
        self.stream_read(&mut buf)?;
        let mut ams_tcp_header = AmsTcpHeader::read_from(&mut buf.as_slice())?;

        if ams_tcp_header.ads_error() != &AdsError::ErrNoError {
            if ams_tcp_header.ads_error() == &AdsError::AdsErrDeviceNotifyHndInvalid {
                self.sym_handle.clear();
            }
            //return Err(anyhow!(ams_tcp_header.ads_error()))
        }

        if ams_tcp_header.response_data_len() > 0 {
            let mut buf = vec![0; ams_tcp_header.response_data_len() as usize];
            self.stream_read(&mut buf)?;
            ams_tcp_header.update_response_data(buf);
            return Ok(ams_tcp_header);
        }
        Ok(ams_tcp_header)
    }

    fn stream_read(&mut self, mut buffer: &mut Vec<u8>) -> Result<()> {
        if let Some(s) = &mut self.stream {
            let mut reader = BufReader::new(s);
            return Ok(reader.read_exact(&mut buffer)?);
        }
        Err(anyhow!(AdsError::ErrPortNotConnected))
    }

    //trial
    pub fn get_symhandle(&mut self, var: Var<'a>) -> Result<u32> {
        if self.sym_handle.contains_key(var.name) {
            if let Some(handle) = self.sym_handle.get(var.name) {
                return Ok(handle.handle);
            }
        }

        let request = Request::ReadWrite(ReadWriteRequest::new(
            GET_SYMHANDLE_BY_NAME.index_group,
            GET_SYMHANDLE_BY_NAME.index_offset_start,
            var.name.len() as u32,
            4, //allways u32 for get_symhandle
            var.name.as_bytes().to_vec(),
        ));

        self.request(request, 0)?;
        let mut response: ReadResponse = self.read_response()?.response()?.try_into()?;
        let raw_handle = response.data.as_slice().read_u32::<LittleEndian>()?;
        let handle = SymHandle::new(raw_handle, var.plc_type);
        self.sym_handle.insert(var.name, handle);
        Ok(raw_handle)
    }

    //trial
    pub fn read_by_name(&mut self, var: Var<'a>, invoke_id: u32) -> Result<Vec<u8>> {
        if !self.sym_handle.contains_key(&var.name) {
            self.get_symhandle(var.clone())?;
        }

        if let Some(handle) = self.sym_handle.get(&var.name) {
            let request = Request::Read(ReadRequest::new(
                READ_WRITE_SYMVAL_BY_HANDLE.index_group,
                handle.handle,
                1,
            ));
            self.request(request, invoke_id)?;
            let response: ReadResponse = self.read_response()?.response()?.try_into()?;
            Ok(response.data)
        } else {
            Err(anyhow!("No symHandle"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn connection_test() {
        let ams_targed_address = AmsAddress::new(AmsNetId::from([10, 2, 129, 32, 1, 1]), 851);
        let route = Some(Ipv4Addr::new(192, 168, 0, 150));
        let mut connection = Connection::new(route, ams_targed_address);
        connection.connect().unwrap();

        let var = Var::new("test", PlcTypes::Bool);
        let handle = connection.get_symhandle(var.clone());
        println!("handle :{:?}", handle);
        println!("handle list : {:?}", connection.sym_handle.values());

        let data = connection.read_by_name(var, 1234).unwrap(); //read by type and retrun accordingly?
        println!("read result : {:?}", data);
        let value = data.as_slice().read_u8().unwrap() != 0; //Improve!
        println!("read result : {:?}", value);
    }
}
