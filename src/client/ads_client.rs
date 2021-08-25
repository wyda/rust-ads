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

use std::convert::TryInto;

/// UDO ADS-Protocol port dicovery
pub const ADS_UDP_SERVER_PORT: u16 = 48899;
/// TCP ADS-Protocol port not secured
pub const ADS_TCP_SERVER_PORT: u16 = 48898;
/// ADS-Protocol port secured
pub const ADS_SECURE_TCP_SERVER_PORT: u16 = 8016;

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
            //Create separeate client error?
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
        let mut buffer = vec![0; 38]; //AmsTcpHeader size without response data.....depend on expected response?! What if no response data? read_to_end?
        self.stream_read(&mut buffer)?;
        Ok(AmsTcpHeader::read_from(&mut buffer.as_slice())?)
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
        let ams_targed_address = AmsAddress::new(AmsNetId::from([10, 2, 129, 32, 1, 1]), 851);
        //let route = Some(Ipv4Addr::new(192, 168, 0, 150));
        //let mut connection = Connection::new(route, ams_targed_address);
        let mut connection = Connection::new(None, ams_targed_address);
        connection.connect();
        let request = Request::Read(ReadRequest::new(16416, 0, 4));
        //let request = Request::ReadDeviceInfo(ReadDeviceInfoRequest::default());
        connection.request(request, 1);
        let mut ams_tcp_header = connection.read_response().unwrap();
        let response_data = ams_tcp_header.response().unwrap();

        let read_response: ReadResponse;
        let read_device_info_response: ReadDeviceInfoResponse;
        let read_state_response: ReadStateResponse;
        let write_response: WriteResponse;
        let write_control_response: WriteControlResponse;
        let add_device_notification_response: AddDeviceNotificationResponse;
        let delete_device_notification_response: DeleteDeviceNotificationResponse;
        let notification_response: AdsNotificationStream;

        match ams_tcp_header.command_id() {
            CommandID::Read => read_response = response_data.try_into().unwrap(),
            CommandID::ReadDeviceInfo => {
                read_device_info_response = response_data.try_into().unwrap()
            }
            CommandID::ReadState => read_state_response = response_data.try_into().unwrap(),
            CommandID::ReadWrite => read_response = response_data.try_into().unwrap(),
            CommandID::Write => write_response = response_data.try_into().unwrap(),
            CommandID::WriteControl => write_control_response = response_data.try_into().unwrap(),
            CommandID::AddDeviceNotification => {
                add_device_notification_response = response_data.try_into().unwrap()
            }
            CommandID::DeleteDeviceNotification => {
                delete_device_notification_response = response_data.try_into().unwrap()
            }
            CommandID::DeviceNotification => {
                notification_response = response_data.try_into().unwrap()
            }
            CommandID::Invalid => (),
        }
    }
}
