use crate::error::AdsError;
use crate::proto::ams_address::{AmsAddress, AmsNetId};
use crate::proto::command_id::CommandID;
use crate::proto::proto_traits::{ReadFrom, SendRecieve, WriteTo};
use crate::proto::request::*;
use crate::proto::response::*;
use crate::proto::state_flags::StateFlags;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

///Length of the fix part of the AMS Header in bytes
const FIX_AMS_HEADER_LEN: u32 = 32;

#[derive(Debug)]
pub struct AmsTcpHeader {
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

impl ReadFrom for AmsTcpHeader {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let reserved = read.read_u16::<LittleEndian>()?;
        let reserved = reserved.to_le_bytes();
        Ok(AmsTcpHeader {
            reserved,
            length: read.read_u32::<LittleEndian>()?,
            ams_header: AmsHeader::read_from(read)?,
        })
    }
}

impl AmsTcpHeader {
    pub fn command_id(&self) -> CommandID {
        self.ams_header.command_id
    }

    pub fn response(&mut self) -> io::Result<Response> {
        self.ams_header.response()
    }

    pub fn response_data_len(&self) -> u32 {
        self.ams_header.length
    }
    
    pub fn update_response_data(&mut self, buf: Vec<u8>) {
        self.ams_header.update_data(buf);
    }
}

impl From<AmsHeader> for AmsTcpHeader {
    fn from(ams_header: AmsHeader) -> Self {
        AmsTcpHeader {
            reserved: [0, 0],
            length: ams_header.header_len(),
            ams_header,
        }
    }
}

#[derive(Debug)]
pub struct AmsHeader {
    ams_address_targed: AmsAddress,
    ams_address_source: AmsAddress,
    command_id: CommandID,
    state_flags: StateFlags,
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
        self.state_flags.write_to(&mut wtr);
        wtr.write_u32::<LittleEndian>(self.length);
        wtr.write_u32::<LittleEndian>(self.ads_error.as_u32());
        wtr.write_u32::<LittleEndian>(self.invoke_id);
        wtr.write_all(&self.data);
        Ok(())
    }
}

impl ReadFrom for AmsHeader {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let ams_address_targed = AmsAddress::read_from(read)?;
        let ams_address_source = AmsAddress::read_from(read)?;
        let command_id = CommandID::read_from(read)?;
        let state_flags = StateFlags::read_from(read)?;
        let length = read.read_u32::<LittleEndian>()?;
        let ads_error = AdsError::from(read.read_u32::<LittleEndian>()?);
        let invoke_id = read.read_u32::<LittleEndian>()?;
        let mut data: Vec<u8> = Vec::new();
        read.read_to_end(&mut data);

        Ok(AmsHeader {
            ams_address_targed,
            ams_address_source,
            command_id,
            state_flags,
            length,
            ads_error,
            invoke_id,
            data,
        })
    }
}

impl AmsHeader {
    pub fn new(
        ams_address_targed: AmsAddress,
        ams_address_source: AmsAddress,
        state_flags: StateFlags,
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

    fn response(&mut self) -> io::Result<Response> {
        match self.command_id {
            CommandID::Invalid => Err(io::Error::new(
                io::ErrorKind::Other,
                AdsError::AdsErrDeviceInvalidData,
            )),
            CommandID::ReadDeviceInfo => Ok(Response::ReadDeviceInfo(
                ReadDeviceInfoResponse::read_from(&mut self.data.as_slice())?,
            )),
            CommandID::Read => Ok(Response::Read(ReadResponse::read_from(
                &mut self.data.as_slice(),
            )?)),
            CommandID::Write => Ok(Response::Write(WriteResponse::read_from(
                &mut self.data.as_slice(),
            )?)),
            CommandID::ReadState => Ok(Response::ReadState(ReadStateResponse::read_from(
                &mut self.data.as_slice(),
            )?)),
            CommandID::WriteControl => Ok(Response::WriteControl(WriteControlResponse::read_from(
                &mut self.data.as_slice(),
            )?)),
            CommandID::Write => Ok(Response::Write(WriteResponse::read_from(
                &mut self.data.as_slice(),
            )?)),
            CommandID::AddDeviceNotification => Ok(Response::AddDeviceNotification(
                AddDeviceNotificationResponse::read_from(&mut self.data.as_slice())?,
            )),
            CommandID::DeleteDeviceNotification => Ok(Response::DeleteDeviceNotification(
                DeleteDeviceNotificationResponse::read_from(&mut self.data.as_slice())?,
            )),
            CommandID::DeviceNotification => Ok(Response::DeviceNotification(
                AdsNotificationStream::read_from(&mut self.data.as_slice())?,
            )),
            CommandID::ReadWrite => Ok(Response::ReadWrite(ReadResponse::read_from(
                &mut self.data.as_slice(),
            )?)),
        }        
    }    

    fn header_len(&self) -> u32 {
        self.data.len() as u32 + FIX_AMS_HEADER_LEN
    }

    pub fn data_len(&self) -> u32 {
        self.length
    }    

    pub fn update_data(&mut self, buf: Vec<u8>) {
        self.data = buf;
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
            StateFlags::resp_default(),
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
            5, 0,                               
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
    fn ams_header_read_from_test() {
        #[rustfmt::skip]
        let mut data: Vec<u8> = vec![
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

        let ams_header = AmsHeader::read_from(&mut data.as_slice()).unwrap();
        assert_eq!(
            ams_header.ams_address_targed.ams_net_id.net_id(),
            [192, 168, 1, 1, 1, 1]
        );
        assert_eq!(
            ams_header.ams_address_source.ams_net_id.net_id(),
            [192, 168, 1, 1, 1, 2]
        );
        assert_eq!(ams_header.ams_address_targed.port, 30000);
        assert_eq!(ams_header.ams_address_source.port, 30000);
        assert_eq!(ams_header.command_id, CommandID::Read);
        assert_eq!(ams_header.state_flags.value(), 4);
        assert_eq!(ams_header.length, 12);
        assert_eq!(ams_header.ads_error, AdsError::ErrNoError);
        assert_eq!(ams_header.invoke_id, 111);
        assert_eq!(ams_header.data, [3, 1, 0, 0, 3, 1, 0, 0, 4, 0, 0, 0]);
    }

    #[test]
    fn ams_header_len_test() {
        let port = 30000;
        let ams_header = AmsHeader::new(
            AmsAddress::new(AmsNetId::parse("192.168.1.1.1.1").unwrap(), port),
            AmsAddress::new(AmsNetId::new(192, 168, 1, 1, 1, 2), port),
            StateFlags::req_default(),
            111,
            Request::Read(ReadRequest::new(259, 259, 4)),
        );

        assert_eq!(ams_header.header_len(), 44);
    }

    #[test]
    fn ams_tcp_header_write_to_test() {
        let mut buffer: Vec<u8> = Vec::new();

        let port = 30000;

        let ams_header = AmsHeader::new(
            AmsAddress::new(AmsNetId::parse("192.168.1.1.1.1").unwrap(), port),
            AmsAddress::new(AmsNetId::new(192, 168, 1, 1, 1, 2), port),
            StateFlags::req_default(),
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

    #[test]
    fn ams_tcp_header_read_from_test() {
        #[rustfmt::skip]
        let mut data: Vec<u8> = vec![
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

        let ams_tcp_header = AmsTcpHeader::read_from(&mut data.as_slice()).unwrap();
        assert_eq!(ams_tcp_header.reserved, [0, 0]);
        assert_eq!(ams_tcp_header.length, 44);
        assert_eq!(
            ams_tcp_header
                .ams_header
                .ams_address_targed
                .ams_net_id
                .net_id(),
            [192, 168, 1, 1, 1, 1]
        );
        assert_eq!(
            ams_tcp_header
                .ams_header
                .ams_address_source
                .ams_net_id
                .net_id(),
            [192, 168, 1, 1, 1, 2]
        );
        assert_eq!(ams_tcp_header.ams_header.ams_address_targed.port, 30000);
        assert_eq!(ams_tcp_header.ams_header.ams_address_source.port, 30000);
        assert_eq!(ams_tcp_header.ams_header.command_id, CommandID::Read);
        assert_eq!(ams_tcp_header.ams_header.state_flags.value(), 4);
        assert_eq!(ams_tcp_header.ams_header.length, 12);
        assert_eq!(ams_tcp_header.ams_header.ads_error, AdsError::ErrNoError);
        assert_eq!(ams_tcp_header.ams_header.invoke_id, 111);
        assert_eq!(
            ams_tcp_header.ams_header.data,
            [3, 1, 0, 0, 3, 1, 0, 0, 4, 0, 0, 0]
        );
    }
}
