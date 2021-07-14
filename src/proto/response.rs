use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Error, Read, Write};

#[derive(Debug)]
pub enum Response {
    ReadDeviceInfo(ReadDeviceInfoResponse),
    Read(DataMessage),
    Write(WriteResponse),
    ReadState(ReadStateResponse),
    WriteControl(WriteControlResponse),
    AddDeviceNotification(AddDeviceNotificationResponse),
    DeleteDeviceNotification(DeleteDeviceNotificationResponse),
    DeviceNotification(DeviceNotificationResponse),
    ReadWrite(DataMessage),
}

/// ADS Read Device Info
#[derive(Debug, PartialEq, Clone)]
pub struct ReadDeviceInfoResponse {
    result: u32,
    major_version: u8,
    minor_version: u8,
    version_build: u16,
    device_name: [u8; 16],
}

impl ReadFrom for ReadDeviceInfoResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let result = read.read_u32::<LittleEndian>()?;
        let major_version = read.read_u8()?;
        let minor_version = read.read_u8()?;
        let version_build = read.read_u16::<LittleEndian>()?;
        let mut device_name = [0; 16];
        read.read_exact(&mut device_name)?;
        Ok(Self {
            result,
            major_version,
            minor_version,
            version_build,
            device_name,
        })
    }
}

///Ads Write
#[derive(Debug, PartialEq, Clone)]
pub struct WriteResponse {
    result: u32,
}

impl ReadFrom for WriteResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let result = read.read_u32::<LittleEndian>()?;
        Ok(Self { result })
    }
}

/// ADS Read State
#[derive(Debug, PartialEq, Clone)]
pub struct ReadStateResponse {
    result: u32,
    ads_state: u16,
    device_state: u16,
}

impl ReadFrom for ReadStateResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        Ok(Self {
            result: read.read_u32::<LittleEndian>()?,
            ads_state: read.read_u16::<LittleEndian>()?,
            device_state: read.read_u16::<LittleEndian>()?,
        })
    }
}

///Write control
#[derive(Debug, PartialEq, Clone)]
pub struct WriteControlResponse {
    result: u32,
}

impl ReadFrom for WriteControlResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let result = read.read_u32::<LittleEndian>()?;
        Ok(Self { result })
    }
}

/// ADS Add Device Notification
#[derive(Debug, PartialEq, Clone)]
pub struct AddDeviceNotificationResponse {
    result: u32,
    notification_handle: u32,
}
impl ReadFrom for AddDeviceNotificationResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        Ok(Self {
            result: read.read_u32::<LittleEndian>()?,
            notification_handle: read.read_u32::<LittleEndian>()?,
        })
    }
}

/// ADS Add Delete Device Notification
#[derive(Debug, PartialEq, Clone)]
pub struct DeleteDeviceNotificationResponse {
    result: u32,
}
impl ReadFrom for DeleteDeviceNotificationResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        Ok(Self {
            result: read.read_u32::<LittleEndian>()?,
        })
    }
}

//ADS Device Notification Response
#[derive(Debug, PartialEq, Clone)]
pub struct AdsNotificationSample {
    notification_handle: u32,
    sample_size: u32,
    data: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StampHeader {
    time_stamp: u64,
    samples: u32,
    notification_samples: AdsNotificationSample,
}

impl ReadFrom for StampHeader {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let time_stamp = read.read_u32::<LittleEndian>()?;
        let samples = read.read_u32::<LittleEndian>()?;
        let notification_handle = read.read_u32::<LittleEndian>()?;
        let sample_size = read.read_u32::<LittleEndian>()?;
        let mut data: Vec<u8> = Vec::new();
        read.read_to_end(&mut data)?;

        Ok(Self {
            time_stamp: read.read_u64::<LittleEndian>()?,
            samples: read.read_u32::<LittleEndian>()?,
            notification_samples: AdsNotificationSample {
                notification_handle,
                sample_size,
                data,
            },
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct DeviceNotificationResponse {
    length: u32,
    stamps: u32,
    ads_stamp_headers: Vec<StampHeader>,
}

impl ReadFrom for DeviceNotificationResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let length = read.read_u32::<LittleEndian>()?;
        let stamps = read.read_u32::<LittleEndian>()?;
        let stamp_data_size = (length / stamps) as u64;
        let mut ads_stamp_headers: Vec<StampHeader> = Vec::new();

        for n in 0..stamps - 1 {
            let stamp = StampHeader::read_from(&mut read.take(stamp_data_size))?;
            ads_stamp_headers.push(stamp);
        }

        Ok(Self {
            length,
            stamps,
            ads_stamp_headers,
        })
    }
}

#[derive(Debug)]
pub struct DataMessage {
    result: u32,
    length: u32,
    data: Vec<u8>,
}

impl ReadFrom for DataMessage {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let result = read.read_u32::<LittleEndian>()?;
        let length = read.read_u32::<LittleEndian>()?;
        let mut data = Vec::with_capacity(length as usize);
        read.read_exact(data.as_mut_slice())?;
        Ok(Self {
            result,
            length,
            data,
        })
    }
}

pub trait ReadFrom: Sized {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn exploration() {
        assert_eq!(2 + 2, 4);
    }
}
