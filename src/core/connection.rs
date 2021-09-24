use chrono::Duration;
/// TODO implement ADS Connection-> use simple TCP Listener
/// standard port should be 3000
/// TODO add async wrapper based on tokio or directly async impl?!
use std::io;
use std::net::SocketAddr;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, RwLock};

use core::ads::*;
use core::requests::*;
use core::responses::*;
use core::router::RouterState;
use num_traits::ToPrimitive;

// TODO see BytesMut for Buffer impl crate bytes
// TODO see Decode Encode traits for enc/dec the request/response data

/// is responsible for connecting the server with an ads client
#[derive(Debug)]
pub struct AmsConnection<'a> {
    router_state: Arc<RwLock<RouterState<'a>>>,
    // connection has an AmsRouter as its parent element
    dest_ip: Ipv4Addr,
    ams_id: AmsNetId,
    stream: Option<TcpStream>,
}

// TODO implement buffer with preallocated memory -> @see BytesMut

impl<'a> AmsConnection<'a> {
    /// create a new AmsConnection object
    pub fn new(
        router_state: Arc<RwLock<RouterState<'a>>>,
        dest_ip: Ipv4Addr,
        ams_id: AmsNetId,
    ) -> Self {
        AmsConnection {
            router_state,
            dest_ip,
            ams_id,
            stream: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    fn stream(&mut self) -> &mut TcpStream {
        self.stream.as_mut().unwrap()
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        match self.stream {
            Some(ref s) => s.local_addr().map_err(|_| AdsError::BadStreamNotConnected),
            _ => Err(AdsError::BadStreamNotConnected),
        }
    }

    /// connect the stream
    pub fn connect(&mut self) -> Result<()> {
        if self.is_connected() {
            // return error instead, don't fail hard
            panic!("Should not try to connect when already connected");
        }

        // TODO add listener to handle the request response mapping
        let stream = TcpStream::connect((self.dest_ip, ADS_TCP_SERVER_PORT))
            .map_err(|e| AdsError::BadStreamNotConnected)?;
        self.stream = Some(stream);

        Ok(())
    }

    // TODO how to return a trait as result datatype?!
    fn write<T: AdsCommandPayload>(
        &mut self,
        request: &AdsRequest<T>,
        src_addr: AmsAddress,
    ) -> Result<()> {
        let command_id = T::command_id().to_u16();

        //        let header = AmsHeader {
        //            target_id: request.dest_addr.net_id.clone(),
        //            target_port: request.dest_addr.port,
        //            source_id: src_addr.net_id.clone(),
        //            source_port: src_addr.port,
        //            command_id: T::command_id(),
        //            state_flag: AdsStateFlag::AmsRequest,
        //            data_length: request.payload.payload_legnth(),
        //            error_code: 0,
        //            invoke_id: self.invoke_id(),
        //        };

        // steps:
        // 1. create the AmsHeader
        // 2. create the amsTcpHeader
        // 3. form the payload [tcpheader, amsheader, [request_info(cmd_id, length), request_data]]
        // 4. write to tcpstream

        Err(AdsError::TargetNotReachable)
    }

    pub fn ads_request<T: AdsCommandPayload>(
        &mut self,
        request: &AdsRequest<T>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        let state = self.router_state.read().map_err(|_| AdsError::SyncError)?;

        //        let src_addr = state.local_ams_net_id

        unimplemented!()
    }

    pub fn dest_id(&self) -> &Ipv4Addr {
        &self.dest_ip
    }

    pub fn update_dest_ip(&mut self, dest_ip: Ipv4Addr) -> Result<()> {
        unimplemented!()
    }

    pub fn ams_id(&self) -> &AmsNetId {
        &self.ams_id
    }

    fn invoke_id(&mut self) -> u32 {
        unimplemented!()
    }
}

impl<'a> Drop for AmsConnection<'a> {
    fn drop(&mut self) {
        //TODO join any waiting recieves
    }
}

impl<'a> AmsProxy for AmsConnection<'a> {
    fn delete_notification(&mut self) {
        unimplemented!()
    }
}
