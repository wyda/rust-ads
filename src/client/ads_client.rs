use anyhow::{anyhow, Error};
use std::io::{Read, Write};
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
use crate::proto::state_flags::StateFlags;

/// UDO ADS-Protocol port dicovery
pub const ADS_UDP_SERVER_PORT: u16 = 48899;
/// TCP ADS-Protocol port not secured
pub const ADS_TCP_SERVER_PORT: u16 = 48898;
/// ADS-Protocol port secured
pub const ADS_SECURE_TCP_SERVER_PORT: u16 = 8016;

pub type Result<T> = result::Result<T, anyhow::Error>;

pub struct Connection {
    route: IpAddr,
    ams_targed_address: AmsAddress,
    ams_source_address: AmsAddress,
    stream: Option<TcpStream>,
}

impl Connection {
    pub fn new(
        route: Option<IpAddr>, //what else is needed to connect to a remote device?
        ams_targed_address: AmsAddress,
    ) -> Self {
        let ip = match route {
            Some(r) => r,
            None => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        };

        Connection {
            route: ip,
            ams_targed_address,
            ams_source_address: AmsAddress::new(AmsNetId::from([127, 0, 0, 1, 1, 1]), 0),
            stream: None,
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        if self.is_connected() {
            //Create separeate client error?
            return Err(anyhow!(AdsError::ErrPortAlreadyConnected));
        }

        let socket_addr = SocketAddr::from((self.route, ADS_TCP_SERVER_PORT));
        self.stream = Some(TcpStream::connect(socket_addr)?);
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

    pub fn read_response(&mut self) -> Result<Response> {
        let mut buffer = Vec::new();
        self.stream_read(&mut buffer)?;
        let ams_tcp_header = AmsTcpHeader::read_from(&mut buffer.as_slice())?;
        let command_id = ams_tcp_header.command_id();
        self.get_response(ams_tcp_header.command_id(), &buffer)
    }

    fn stream_read(&mut self, mut buffer: &mut Vec<u8>) -> Result<usize> {
        if let Some(s) = &mut self.stream {
            return Ok(s.read(buffer)?);
        }
        Err(anyhow!(AdsError::ErrPortNotConnected))
    }

    fn get_response(&self, command_id: CommandID, mut buffer: &[u8]) -> Result<Response> {
        match command_id {
            CommandID::Invalid => Err(anyhow!(AdsError::AdsErrDeviceInvalidData)),
            CommandID::ReadDeviceInfo => Ok(Response::ReadDeviceInfo(
                ReadDeviceInfoResponse::read_from(&mut buffer)?,
            )),
            CommandID::Read => Ok(Response::Read(ReadResponse::read_from(&mut buffer)?)),
            CommandID::Write => Ok(Response::Write(WriteResponse::read_from(&mut buffer)?)),
            CommandID::ReadState => Ok(Response::ReadState(ReadStateResponse::read_from(
                &mut buffer,
            )?)),
            CommandID::WriteControl => Ok(Response::WriteControl(WriteControlResponse::read_from(
                &mut buffer,
            )?)),
            CommandID::Write => Ok(Response::Write(WriteResponse::read_from(&mut buffer)?)),
            CommandID::AddDeviceNotification => Ok(Response::AddDeviceNotification(
                AddDeviceNotificationResponse::read_from(&mut buffer)?,
            )),
            CommandID::DeleteDeviceNotification => Ok(Response::DeleteDeviceNotification(
                DeleteDeviceNotificationResponse::read_from(&mut buffer)?,
            )),
            CommandID::DeviceNotification => Ok(Response::DeviceNotification(
                AdsNotificationStream::read_from(&mut buffer)?,
            )),
            CommandID::ReadWrite => Ok(Response::ReadWrite(ReadResponse::read_from(&mut buffer)?)),
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn connection_test() {
        let ams_targed_address = AmsAddress::new(AmsNetId::from([192, 168, 0, 150, 1, 1]), 851);
        let mut connection = Connection::new(None, ams_targed_address);
        connection.connect();
        let request = Request::ReadDeviceInfo(ReadDeviceInfoRequest::default());
        println!("{:?}", &connection.request(request, 1));
        let response = connection.read_response().unwrap();
    }
}
*/
