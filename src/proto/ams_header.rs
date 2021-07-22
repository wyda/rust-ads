use crate::error::AdsError;
use crate::proto::ams_address::{AmsAddress, AmsNetId};
use crate::proto::command_id::CommandID;
use crate::proto::request::{ReadRequest, Request, WriteTo};

use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Write};

///Length of the fix part of the AMS Header in bytes
const FIX_HEADER_LEN: u32 = 32;

struct AmsTcpHeader {
    reserved: [u8; 2],
    length: u32,
    ams_header: AmsHeader,
}

impl WriteTo for AmsTcpHeader {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_all(&self.reserved);
        wtr.write_u32::<LittleEndian>(self.length);
        self.ams_header.write_to(&mut wtr);
        Ok(())
    }
}

impl From<AmsHeader> for AmsTcpHeader {
    fn from(ams_header: AmsHeader) -> Self {
        AmsTcpHeader {
            reserved: [0, 0],
            length: ams_header.len(),
            ams_header,
        }
    }
}

struct AmsHeader {
    ams_address_targed: AmsAddress,
    ams_address_source: AmsAddress,
    command_id: CommandID,
    state_flags: u16, //ToDo create enum or struct?
    length: u32,
    ads_error: AdsError,
    invoke_id: u32,
    data: Vec<u8>,
}

impl WriteTo for AmsHeader {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        self.ams_address_targed.write_to(&mut wtr);
        self.ams_address_source.write_to(&mut wtr);
        self.command_id.write_to(&mut wtr);
        wtr.write_u16::<LittleEndian>(self.state_flags);
        wtr.write_u32::<LittleEndian>(self.length);
        wtr.write_u32::<LittleEndian>(self.ads_error.as_u32());
        wtr.write_u32::<LittleEndian>(self.invoke_id);
        wtr.write_all(&self.data);
        Ok(())
    }
}

impl AmsHeader {
    pub fn new(
        ams_address_targed: AmsAddress,
        ams_address_source: AmsAddress,
        state_flags: u16,
        invoke_id: u32,
        request: Request,
    ) -> Self {
        let mut data: Vec<u8> = Vec::new();
        request.write_to(&mut data);

        AmsHeader {
            ams_address_targed,
            ams_address_source,
            command_id: request.command_id(),
            state_flags,
            length: data.len() as u32,
            ads_error: AdsError::ErrNoError,
            invoke_id,
            data,
        }
    }

    fn len(&self) -> u32 {
        self.data.len() as u32 + FIX_HEADER_LEN
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ams_header_write_to_test() {
        let mut buffer: Vec<u8> = Vec::new();

        let port = 30000;

        let ams_header = AmsHeader::new(
            AmsAddress::new(AmsNetId::parse("192.168.1.1.1.1").unwrap(), port),
            AmsAddress::new(AmsNetId::new(192, 168, 1, 1, 1, 2), port),
            4,
            111,
            Request::Read(ReadRequest::new(259, 259, 4)),
        );

        ams_header.write_to(&mut buffer);

        #[rustfmt::skip]
        let compare: Vec<u8> = vec![
            //target AmsAddress -> NetId/port (192.168.1.1.1.1, 30000)
            192, 168, 1, 1, 1, 1, 48, 117,      
            //Source AmsAddress -> NetId/port (192.168.1.1.1.2, 30000)
            192, 168, 1, 1, 1, 2, 48, 117,      
            //CommandID -> Read 
            2, 0,                               
            //state flag -> Request, Ads command, TCP (4)
            4, 0,                               
            //Lennth of data for read request (12 byte)
            12, 0, 0, 0,                        
            //Error code -> No error 
            0, 0, 0, 0,                         
            //Invoke ID -> 111 
            111, 0, 0, 0,                       
            //Data from read request -> see request.rs
            3, 1, 0, 0, 3, 1, 0, 0, 4, 0, 0, 0  
        ];
        assert_eq!(compare, buffer);
    }

    #[test]
    fn ams_header_len_test() {
        let port = 30000;
        let ams_header = AmsHeader::new(
            AmsAddress::new(AmsNetId::parse("192.168.1.1.1.1").unwrap(), port),
            AmsAddress::new(AmsNetId::new(192, 168, 1, 1, 1, 2), port),
            4,
            111,
            Request::Read(ReadRequest::new(259, 259, 4)),
        );

        assert_eq!(ams_header.len(), 44);
    }

    #[test]
    fn ams_tcp_header_write_to_test() {
        let mut buffer: Vec<u8> = Vec::new();

        let port = 30000;

        let ams_header = AmsHeader::new(
            AmsAddress::new(AmsNetId::parse("192.168.1.1.1.1").unwrap(), port),
            AmsAddress::new(AmsNetId::new(192, 168, 1, 1, 1, 2), port),
            4,
            111,
            Request::Read(ReadRequest::new(259, 259, 4)),
        );

        let ams_tcp_header = AmsTcpHeader::from(ams_header);
        ams_tcp_header.write_to(&mut buffer);

        #[rustfmt::skip]
        let compare: Vec<u8> = vec![
            //Reserved has to be 0
            0,0,
            //Length in bytes of AmsHeader
            44, 0, 0, 0,
            //target AmsAddress -> NetId/port (192.168.1.1.1.1, 30000)
            192, 168, 1, 1, 1, 1, 48, 117,      
            //Source AmsAddress -> NetId/port (192.168.1.1.1.2, 30000)
            192, 168, 1, 1, 1, 2, 48, 117,      
            //CommandID -> Read 
            2, 0,                               
            //state flag -> Request, Ads command, TCP (4)
            4, 0,                               
            //Lennth of data for read request (12 byte)
            12, 0, 0, 0,                        
            //Error code -> No error 
            0, 0, 0, 0,                         
            //Invoke ID -> 111 
            111, 0, 0, 0,                       
            //Data from read request -> see request.rs
            3, 1, 0, 0, 3, 1, 0, 0, 4, 0, 0, 0  
        ];
        assert_eq!(compare, buffer);
    }
}
