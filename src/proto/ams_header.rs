use crate::error::AdsError;
use crate::proto::ams_address::AmsAddress;
use crate::proto::command_id::CommandID;
use crate::proto::request::Request;

use crate::proto::request::WriteTo;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Write};

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
}
