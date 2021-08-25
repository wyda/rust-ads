use anyhow::{anyhow, Error};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::result;

use crate::error::AdsError;
use crate::proto::ams_address::{AmsAddress, AmsNetId};
use crate::proto::ams_header::*;
use crate::proto::command_id::CommandID;
use crate::proto::proto_traits::*;
use crate::proto::request::*;
use crate::proto::response::*;
use crate::proto::state_flags::*;

/// UDP ADS-Protocol port dicovery
pub const ADS_UDP_SERVER_PORT: u16 = 48899;
/// TCP ADS-Protocol port not secured
pub const ADS_TCP_SERVER_PORT: u16 = 48898;
/// ADS-Protocol port secured
pub const ADS_SECURE_TCP_SERVER_PORT: u16 = 8016;
//Tcp Header size without response data
pub const AMS_HEADER_SIZE: usize = 38;

pub type Result<T> = result::Result<T, anyhow::Error>;

pub struct Connection {
    route: Ipv4Addr,
    ams_targed_address: AmsAddress,
    ams_source_address: AmsAddress,
    stream: Option<TcpStream>,
}

impl Connection {
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
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        if self.is_connected() {           
            return Err(anyhow!(AdsError::ErrPortAlreadyConnected));
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

        if ams_tcp_header.response_data_len() > 0 {        
            let mut buf = vec![0; ams_tcp_header.response_data_len() as usize];
            self.stream_read(&mut buf)?; 
            ams_tcp_header.update_response_data(buf);
            return Ok(ams_tcp_header)    
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn connection_test() {                
        let ams_targed_address = AmsAddress::new(AmsNetId::from([192, 168, 0, 150, 1, 1]), 851);
        //let route = Some(Ipv4Addr::new(192, 168, 0, 150));
        //let mut connection = Connection::new(route, ams_targed_address);        
        let mut connection = Connection::new(None, ams_targed_address);        
        connection.connect();
        //let request = Request::Read(ReadRequest::new(16416, 0, 4));
        let request = Request::ReadDeviceInfo(ReadDeviceInfoRequest::default());
        connection.request(request, 1);
        let mut ams_tcp_header = connection.read_response().unwrap();
        let response_data = ams_tcp_header.response();

        println!("{:?}", ams_tcp_header); //ErrPortDisabled if loop back.
        println!("{:?}", response_data);

        //println!("{:?}", response_data.unwrap());
    }
}
