use anyhow::anyhow;
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::result;

use crate::ads_services::system_services::*;
use crate::client::plc_types::Var;
use crate::error::AdsError;
use crate::proto::ads_state::*;
use crate::proto::ams_address::{AmsAddress, AmsNetId};
use crate::proto::ams_header::*;
use crate::proto::command_id::CommandID;
use crate::proto::proto_traits::*;
use crate::proto::response::*;
use crate::proto::state_flags::*;
use std::convert::TryInto;

pub const AMS_HEADER_SIZE: usize = 38;

pub type ClientResult<T> = result::Result<T, anyhow::Error>;

pub struct AdsReader {
    pub stream: TcpStream,
}

impl AdsReader {
    pub fn new(stream: TcpStream) -> Self {
        AdsReader { stream }
    }

    pub fn read_response(&mut self) -> ClientResult<AmsTcpHeader> {
        let mut buf = vec![0; AMS_HEADER_SIZE];
        let mut reader = BufReader::new(&self.stream);
        reader.read_exact(&mut buf)?;
        let mut ams_tcp_header = AmsTcpHeader::read_from(&mut buf.as_slice())?;
        if ams_tcp_header.response_data_len() > 0 {
            let mut buf = vec![0; ams_tcp_header.response_data_len() as usize];
            reader.read_exact(&mut buf)?;
            ams_tcp_header.update_response_data(buf);
            return Ok(ams_tcp_header);
        }
        Ok(ams_tcp_header)
    }
}
