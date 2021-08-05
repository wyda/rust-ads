use std::io::Error;
use std::net::TcpStream;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::result;

use crate::error::AdsError;
use crate::proto::ams_address::{AmsAddress, AmsNetId};

/// UDO ADS-Protocol port dicovery
pub const ADS_UDP_SERVER_PORT: u16 = 48899;
/// TCP ADS-Protocol port not secured
pub const ADS_TCP_SERVER_PORT: u16 = 48898;
/// ADS-Protocol port secured
pub const ADS_SECURE_TCP_SERVER_PORT: u16 = 8016;

pub type Result<T> = result::Result<T, Error>;

pub struct Connection {
    route: IpAddr,
    ams_address: AmsAddress,
    stream: Option<TcpStream>,
}

impl Connection {
    pub fn new(
        route: Option<IpAddr>, //what else is needed to connect to a remote device?
        ams_address: AmsAddress,
    ) -> Self {
        let ip = match route {
            Some(r) => r,
            None => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        };

        Connection {
            route: ip,
            ams_address,
            stream: None,
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        //TODO new error type -> like client error? AdsError does not realy fit..
        /* if self.is_connected() {
            return Err(AdsError::ErrPortAlreadyConnected)
        }
        */

        let socket_addr = SocketAddr::from((self.route, 8080));
        self.stream = Some(TcpStream::connect(socket_addr)?);
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
}
