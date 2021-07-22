use byteorder::{LittleEndian, WriteBytesExt};
use bytes::BufMut;
use std::io::{self, Error, Write};

use crate::proto::ads_state::AdsState;
use crate::proto::command_id::CommandID;

pub trait WriteTo {
    fn write_to<W: Write>(&self, wtr: W) -> io::Result<()>;
}

pub trait SendRecieve {
    // TODO add router as param that implements read to write
    fn send_receive(&self) -> io::Result<()>;
}

#[derive(Debug)]
pub enum Request {
    Invalid(InvalidRequest),
    ReadDeviceInfo(ReadDeviceInfoRequest),
    ReadState(ReadStateRequest),
    Read(ReadRequest),
    Write(WriteRequest),
    WriteControl(WriteControlRequest),
    AddDeviceNotification(AddDeviceNotificationRequest),
    DeleteDeviceNotification(DeleteDeviceNotificationRequest),
    DeviceNotification(DeviceNotificationRequest),
    ReadWrite(ReadWriteRequest),
}

impl WriteTo for Request {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        match self {
            Request::Invalid(_) => Ok(()),
            Request::ReadDeviceInfo(_) => Ok(()),
            Request::ReadState(_) => Ok(()),
            Request::Read(r) => r.write_to(wtr),
            Request::Write(r) => r.write_to(wtr),
            Request::ReadWrite(r) => r.write_to(wtr),
            Request::AddDeviceNotification(r) => r.write_to(wtr),
            Request::WriteControl(r) => r.write_to(wtr),
            Request::DeviceNotification(_) => Ok(()),
            Request::DeleteDeviceNotification(r) => r.write_to(wtr),
        }
    }
}

impl Request {
    pub fn command_id(&self) -> CommandID {
        match self {
            Request::Invalid(r) => r.command_id,
            Request::ReadDeviceInfo(r) => r.command_id,
            Request::ReadState(r) => r.command_id,
            Request::Read(r) => r.command_id,
            Request::Write(r) => r.command_id,
            Request::ReadWrite(r) => r.command_id,
            Request::AddDeviceNotification(r) => r.command_id,
            Request::WriteControl(r) => r.command_id,
            Request::DeviceNotification(r) => r.command_id,
            Request::DeleteDeviceNotification(r) => r.command_id,
        }
    }
}

/// ADS Invalid request
#[derive(Debug, PartialEq)]
pub struct InvalidRequest {
    command_id: CommandID,
}

impl InvalidRequest {
    pub fn new() -> Self {
        InvalidRequest {
            command_id: CommandID::Invalid,
        }
    }
}

impl Default for InvalidRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// ADS read device info request
#[derive(Debug, PartialEq)]
pub struct ReadDeviceInfoRequest {
    command_id: CommandID,
}

impl ReadDeviceInfoRequest {
    pub fn new() -> Self {
        ReadDeviceInfoRequest {
            command_id: CommandID::ReadDeviceInfo,
        }
    }
}

impl Default for ReadDeviceInfoRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// ADS read device info request
#[derive(Debug, PartialEq)]
pub struct ReadStateRequest {
    command_id: CommandID,
}

impl ReadStateRequest {
    pub fn new() -> Self {
        ReadStateRequest {
            command_id: CommandID::ReadState,
        }
    }
}

impl Default for ReadStateRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// ADS read device info request
#[derive(Debug, PartialEq)]
pub struct DeviceNotificationRequest {
    command_id: CommandID,
}

impl DeviceNotificationRequest {
    pub fn new() -> Self {
        DeviceNotificationRequest {
            command_id: CommandID::DeviceNotification,
        }
    }
}

impl Default for DeviceNotificationRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// ADS Read
#[derive(Debug, PartialEq)]
pub struct ReadRequest {
    index_group: u32,
    index_offset: u32,
    length: u32,
    command_id: CommandID,
}

impl ReadRequest {
    pub fn new(index_group: u32, index_offset: u32, length: u32) -> Self {
        ReadRequest {
            index_group,
            index_offset,
            length,
            command_id: CommandID::Read,
        }
    }
}

impl WriteTo for ReadRequest {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.index_group)?;
        wtr.write_u32::<LittleEndian>(self.index_offset)?;
        wtr.write_u32::<LittleEndian>(self.length);
        Ok(())
    }
}

///ADS Write
#[derive(Debug, PartialEq)]
pub struct WriteRequest {
    index_group: u32,
    index_offset: u32,
    length: u32,
    data: Vec<u8>,
    command_id: CommandID,
}

impl WriteRequest {
    pub fn new(index_group: u32, index_offset: u32, length: u32, data: Vec<u8>) -> Self {
        WriteRequest {
            index_group,
            index_offset,
            length,
            data,
            command_id: CommandID::Write,
        }
    }
}

impl WriteTo for WriteRequest {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.index_group)?;
        wtr.write_u32::<LittleEndian>(self.index_offset)?;
        wtr.write_u32::<LittleEndian>(self.length)?;
        wtr.write_all(self.data.as_slice())?;
        Ok(())
    }
}

/// ADS Write Control
#[derive(Debug, PartialEq)]
pub struct WriteControlRequest {
    ads_state: AdsState,
    device_state: u16,
    length: u32,
    data: Vec<u8>,
    command_id: CommandID,
}

impl WriteControlRequest {
    pub fn new(ads_state: AdsState, device_state: u16, length: u32, data: Vec<u8>) -> Self {
        WriteControlRequest {
            ads_state,
            device_state,
            length,
            data,
            command_id: CommandID::WriteControl,
        }
    }
}

impl WriteTo for WriteControlRequest {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        self.ads_state.write_to(&mut wtr)?;
        wtr.write_u16::<LittleEndian>(self.device_state)?;
        wtr.write_u32::<LittleEndian>(self.length)?;
        wtr.write_all(self.data.as_slice())?;
        Ok(())
    }
}

/// ADS Add Device Notification
#[derive(Debug, PartialEq, Clone)]
pub struct AddDeviceNotificationRequest {
    index_group: u32,
    index_offset: u32,
    length: u32,
    transmission_mode: u32,
    max_delay: u32,
    cycle_time: u32,
    reserved: [u8; 16],
    command_id: CommandID,
}

impl AddDeviceNotificationRequest {
    pub fn new(
        index_group: u32,
        index_offset: u32,
        length: u32,
        transmission_mode: AdsTransMode,
        max_delay: u32,
        cycle_time: u32,
    ) -> Self {
        AddDeviceNotificationRequest {
            index_group,
            index_offset,
            length,
            transmission_mode: AdsTransMode::get_value(transmission_mode),
            max_delay,
            cycle_time,
            reserved: [0; 16],
            command_id: CommandID::AddDeviceNotification,
        }
    }
}

/// ADS read device info request
#[derive(Debug, PartialEq)]
pub struct DeleteDeviceNotificationRequest {
    handle: u32,
    command_id: CommandID,
}

impl WriteTo for DeleteDeviceNotificationRequest {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.handle)?;
        Ok(())
    }
}

impl DeleteDeviceNotificationRequest {
    fn new(handle: u32) -> Self {
        DeleteDeviceNotificationRequest {
            handle,
            command_id: CommandID::DeleteDeviceNotification,
        }
    }
}

impl WriteTo for AddDeviceNotificationRequest {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.index_group)?;
        wtr.write_u32::<LittleEndian>(self.index_offset)?;
        wtr.write_u32::<LittleEndian>(self.length)?;
        wtr.write_u32::<LittleEndian>(self.transmission_mode)?;
        wtr.write_u32::<LittleEndian>(self.max_delay)?;
        wtr.write_u32::<LittleEndian>(self.cycle_time)?;
        wtr.write_all(&self.reserved);
        Ok(())
    }
}

pub enum AdsTransMode {
    None,
    ClientCylcle,
    ClientOnChange,
    Cyclic,
    OnChange,
    CyclicInContext,
    OnChangeInContext,
}

impl AdsTransMode {
    pub fn get_value(mode: AdsTransMode) -> u32 {
        match mode {
            AdsTransMode::None => 0,
            AdsTransMode::ClientCylcle => 1,
            AdsTransMode::ClientOnChange => 2,
            AdsTransMode::Cyclic => 3,
            AdsTransMode::OnChange => 4,
            AdsTransMode::CyclicInContext => 5,
            AdsTransMode::OnChangeInContext => 6,
        }
    }
}

/// ADS Read Write
#[derive(Debug, PartialEq)]
pub struct ReadWriteRequest {
    index_group: u32,
    index_offset: u32,
    read_length: u32,
    write_length: u32,
    data: Vec<u8>,
    command_id: CommandID,
}

impl ReadWriteRequest {
    pub fn new(
        index_group: u32,
        index_offset: u32,
        read_length: u32,
        write_length: u32,
        data: Vec<u8>,
    ) -> Self {
        ReadWriteRequest {
            index_group,
            index_offset,
            read_length,
            write_length,
            data,
            command_id: CommandID::ReadWrite,
        }
    }
}

impl WriteTo for ReadWriteRequest {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.index_group)?;
        wtr.write_u32::<LittleEndian>(self.index_offset)?;
        wtr.write_u32::<LittleEndian>(self.read_length)?;
        wtr.write_u32::<LittleEndian>(self.write_length)?;
        wtr.write_all(self.data.as_slice());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_request_test() {
        let mut buffer: Vec<u8> = Vec::new();
        Request::Read(ReadRequest::new(259, 259, 4)).write_to(&mut buffer);

        let compare: Vec<u8> = vec![3, 1, 0, 0, 3, 1, 0, 0, 4, 0, 0, 0];
        assert_eq!(compare, buffer);
    }

    #[test]
    fn write_uint_request_test() {
        let mut buffer: Vec<u8> = Vec::new();
        let data: u32 = 12000;
        Request::Write(WriteRequest::new(259, 259, 4, data.to_le_bytes().to_vec()))
            .write_to(&mut buffer);

        let compare: Vec<u8> = vec![3, 1, 0, 0, 3, 1, 0, 0, 4, 0, 0, 0, 224, 46, 0, 0];
        assert_eq!(compare, buffer);
    }

    #[test]
    fn write_float_request_test() {
        let mut buffer: Vec<u8> = Vec::new();
        let data: f32 = 12000.33;
        Request::Write(WriteRequest::new(259, 259, 4, data.to_le_bytes().to_vec()))
            .write_to(&mut buffer);

        let compare: Vec<u8> = vec![3, 1, 0, 0, 3, 1, 0, 0, 4, 0, 0, 0, 82, 129, 59, 70];
        assert_eq!(compare, buffer);
    }

    #[test]
    fn write_control_request_test() {
        let mut buffer: Vec<u8> = Vec::new();
        let data: u8 = 0;
        Request::WriteControl(WriteControlRequest::new(
            AdsState::AdsStateIdle,
            296,
            1,
            data.to_le_bytes().to_vec(),
        ))
        .write_to(&mut buffer);

        let compare: Vec<u8> = vec![1, 0, 40, 1, 1, 0, 0, 0, 0];
        assert_eq!(compare, buffer);
    }

    #[test]
    fn read_write_request_test() {
        let mut buffer: Vec<u8> = Vec::new();
        let data: u32 = 40000;
        let data: Vec<u8> = data.to_le_bytes().to_vec();
        Request::ReadWrite(ReadWriteRequest::new(259, 259, 4, 4, data)).write_to(&mut buffer);

        let compare: Vec<u8> = vec![
            3, 1, 0, 0, 3, 1, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 64, 156, 0, 0,
        ];
        assert_eq!(compare, buffer);
    }

    #[test]
    fn add_device_notification_request_test() {
        let mut buffer: Vec<u8> = Vec::new();
        Request::AddDeviceNotification(AddDeviceNotificationRequest::new(
            259,
            259,
            4,
            AdsTransMode::Cyclic,
            1,
            1,
        ))
        .write_to(&mut buffer);

        let compare: Vec<u8> = vec![
            3, 1, 0, 0, 3, 1, 0, 0, 4, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(compare, buffer);
    }

    #[test]
    fn delete_device_notification_request_test() {
        let mut buffer: Vec<u8> = Vec::new();
        let notification_handle = DeleteDeviceNotificationRequest::new(1234);
        Request::DeleteDeviceNotification(notification_handle).write_to(&mut buffer);

        let compare: Vec<u8> = vec![210, 4, 0, 0];
        assert_eq!(compare, buffer);
    }
}
