use crate::error::AdsError;
use crate::proto::ads_state::AdsState;
use crate::proto::proto_traits::{ReadFrom, SendRecieve, WriteTo};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Error, Read, Write};

#[derive(Debug)]
pub enum Response {
    ReadDeviceInfo(ReadDeviceInfoResponse),
    Read(ReadResponse),
    Write(WriteResponse),
    ReadState(ReadStateResponse),
    WriteControl(WriteControlResponse),
    AddDeviceNotification(AddDeviceNotificationResponse),
    DeleteDeviceNotification(DeleteDeviceNotificationResponse),
    DeviceNotification(AdsNotificationStream),
    ReadWrite(ReadResponse),
}

/*impl WriteTo for Response {
    fn write_to<W: Write>(&self, wtr: W) -> io::Result<()> {
        match self {
            Response::ReadDeviceInfo(r) => r.write_to(&mut wtr),
        }
    }
}*/

/// ADS Read Device Info
#[derive(Debug, PartialEq, Clone)]
pub struct ReadDeviceInfoResponse {
    result: AdsError,
    major_version: u8,
    minor_version: u8,
    version_build: u16,
    device_name: [u8; 16],
}

impl ReadFrom for ReadDeviceInfoResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let result = AdsError::from(read.read_u32::<LittleEndian>()?);
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

impl WriteTo for ReadDeviceInfoResponse {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.result.as_u32());
        wtr.write_u8(self.major_version);
        wtr.write_u8(self.minor_version);
        wtr.write_u16::<LittleEndian>(self.version_build);
        wtr.write_all(&self.device_name);
        Ok(())
    }
}

impl ReadDeviceInfoResponse {
    pub fn new(
        result: AdsError,
        major_version: u8,
        minor_version: u8,
        version_build: u16,
        device_name: [u8; 16],
    ) -> Self {
        ReadDeviceInfoResponse {
            result,
            major_version,
            minor_version,
            version_build,
            device_name,
        }
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
    ads_state: AdsState,
    device_state: u16,
}

impl ReadFrom for ReadStateResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        Ok(Self {
            result: read.read_u32::<LittleEndian>()?,
            ads_state: AdsState::from(read.read_u16::<LittleEndian>()?),
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
        Ok(Self {
            result: read.read_u32::<LittleEndian>()?,
        })
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
pub struct AdsStampHeader {
    time_stamp: u64,
    samples: u32,
    notification_samples: Vec<AdsNotificationSample>,
}

impl ReadFrom for AdsStampHeader {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let time_stamp = read.read_u64::<LittleEndian>()?;
        let samples = read.read_u32::<LittleEndian>()?;
        let mut notification_samples: Vec<AdsNotificationSample> =
            Vec::with_capacity(samples as usize);

        for n in 0..samples {
            let notification_handle = read.read_u32::<LittleEndian>()?;
            let sample_size = read.read_u32::<LittleEndian>()?;
            let mut data = vec![0; sample_size as usize];
            read.read_exact(&mut data)?;
            notification_samples.push(AdsNotificationSample {
                notification_handle,
                sample_size,
                data,
            });
        }

        Ok(Self {
            time_stamp,
            samples,
            notification_samples,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AdsNotificationStream {
    length: u32,
    stamps: u32,
    ads_stamp_headers: Vec<AdsStampHeader>,
}

impl ReadFrom for AdsNotificationStream {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let length = read.read_u32::<LittleEndian>()?;
        let stamps = read.read_u32::<LittleEndian>()?;
        let stamp_data_size = (length / stamps) as u64;
        let mut ads_stamp_headers: Vec<AdsStampHeader> = Vec::with_capacity(stamps as usize);
        let mut buffer: Vec<u8> = vec![0; stamp_data_size as usize];

        for n in 0..stamps {
            read.read_exact(&mut buffer.as_mut_slice());
            let stamp = AdsStampHeader::read_from(&mut buffer.as_slice())?;
            ads_stamp_headers.push(stamp);
        }

        Ok(Self {
            length,
            stamps,
            ads_stamp_headers,
        })
    }
}

//Asd Read response
#[derive(Debug)]
pub struct ReadResponse {
    result: u32,
    length: u32,
    data: Vec<u8>,
}

impl ReadFrom for ReadResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let result = read.read_u32::<LittleEndian>()?;
        let length = read.read_u32::<LittleEndian>()?;
        let mut data = Vec::with_capacity(length as usize);
        read.read_to_end(&mut data)?;
        Ok(Self {
            result,
            length,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn read_device_info_response_write_to_test() {
        let device_name = "MyDevice".as_bytes().try_into().unwrap();
        let device_info_response =
            ReadDeviceInfoResponse::new(AdsError::ErrNoError, 1, 2, 10, device_name);

        let mut response_data: Vec<u8> = vec![
            3, 1, 0, 0, 2, 14, 1, 1, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 0, 0, 0,
            0, 0,
        ];

        let read_device_info_response =
            ReadDeviceInfoResponse::read_from(&mut response_data.as_slice()).unwrap();

        let response = Response::ReadDeviceInfo(read_device_info_response.clone());

        assert_eq!(read_device_info_response.result, 259);
        assert_eq!(read_device_info_response.major_version, 2);
        assert_eq!(read_device_info_response.minor_version, 14);
        assert_eq!(read_device_info_response.version_build, 257);

        let expected_device_name: [u8; 16] = [
            72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 0, 0, 0, 0, 0,
        ]; //Hello World
        assert_eq!(read_device_info_response.device_name, expected_device_name);
    }

    #[test]
    fn read_device_info_response_test() {
        let mut response_data: Vec<u8> = vec![
            3, 1, 0, 0, 2, 14, 1, 1, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 0, 0, 0,
            0, 0,
        ];

        let read_device_info_response =
            ReadDeviceInfoResponse::read_from(&mut response_data.as_slice()).unwrap();

        let response = Response::ReadDeviceInfo(read_device_info_response.clone());

        assert_eq!(read_device_info_response.result, 259);
        assert_eq!(read_device_info_response.major_version, 2);
        assert_eq!(read_device_info_response.minor_version, 14);
        assert_eq!(read_device_info_response.version_build, 257);

        let expected_device_name: [u8; 16] = [
            72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 0, 0, 0, 0, 0,
        ]; //Hello World
        assert_eq!(read_device_info_response.device_name, expected_device_name);
    }

    #[test]
    fn read_response_test() {
        let mut response_data: Vec<u8> = vec![4, 0, 0, 0, 2, 0, 0, 0, 255, 2];

        let read_response = ReadResponse::read_from(&mut response_data.as_slice()).unwrap();

        assert_eq!(read_response.result, 4);
        assert_eq!(read_response.length, 2);
        assert_eq!(read_response.data, vec![255, 2]);
    }

    #[test]
    fn write_response_test() {
        let mut response_data: Vec<u8> = vec![4, 0, 0, 0];

        let write_response = WriteResponse::read_from(&mut response_data.as_slice()).unwrap();

        assert_eq!(write_response.result, 4);
    }

    #[test]
    fn read_state_response_test() {
        let mut response_data: Vec<u8> = vec![4, 0, 0, 0, 9, 0, 1, 1];

        let read_state_response =
            ReadStateResponse::read_from(&mut response_data.as_slice()).unwrap();

        assert_eq!(read_state_response.result, 4);
        assert_eq!(
            read_state_response.ads_state,
            AdsState::AdsStatePowerFailure
        );
        assert_eq!(read_state_response.device_state, 257);
    }

    #[test]
    fn write_control_response_test() {
        let mut response_data: Vec<u8> = vec![4, 0, 0, 0];

        let write_control_response =
            WriteControlResponse::read_from(&mut response_data.as_slice()).unwrap();

        assert_eq!(write_control_response.result, 4);
    }

    #[test]
    fn add_device_notification_response_test() {
        let mut response_data: Vec<u8> = vec![4, 0, 0, 0, 10, 0, 0, 0];

        let add_device_notification_response =
            AddDeviceNotificationResponse::read_from(&mut response_data.as_slice()).unwrap();

        assert_eq!(add_device_notification_response.result, 4);
        assert_eq!(add_device_notification_response.notification_handle, 10);
    }

    #[test]
    fn delete_device_notification_response_test() {
        let mut response_data: Vec<u8> = vec![4, 0, 0, 0];

        let delete_device_notification_response =
            DeleteDeviceNotificationResponse::read_from(&mut response_data.as_slice()).unwrap();

        assert_eq!(delete_device_notification_response.result, 4);
    }

    #[test]
    fn ads_notification_stream_test() {
        let mut notification_sample1: Vec<u8> = vec![4, 0, 0, 0, 2, 0, 0, 0, 6, 0];
        let mut notification_sample2: Vec<u8> = vec![4, 0, 0, 0, 4, 0, 0, 0, 9, 0, 0, 0];

        let mut stamp_header: Vec<u8> = vec![255, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0];
        stamp_header.extend(notification_sample1);
        stamp_header.extend(notification_sample2);

        let mut notification_stream: Vec<u8> = vec![68, 0, 0, 0, 2, 0, 0, 0];
        notification_stream.extend(stamp_header.clone());
        notification_stream.extend(stamp_header);

        let notification_data =
            AdsNotificationStream::read_from(&mut notification_stream.as_slice()).unwrap();

        assert_eq!(notification_data.length, 68, "Wrong data stream length");
        assert_eq!(notification_data.stamps, 2, "Wrong data stream stamp count");
        assert_eq!(
            notification_data.ads_stamp_headers.len(),
            2,
            "Wrong stamp header vec length"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[0]
                .notification_samples
                .len(),
            2,
            "Wrong notification sample vec len [0]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[0].samples, 2,
            "Wrong notification samples count [0]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[0].time_stamp, 255,
            "Wrong time stamp [0]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[0].notification_samples[0].notification_handle, 4,
            "Wrong notification handle [0][0]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[0].notification_samples[0].sample_size, 2,
            "Wrong sample size [0][0]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[0].notification_samples[0].data,
            vec![6, 0],
            "Wrong data [0][0]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[0].notification_samples[1].notification_handle, 4,
            "Wrong notification handle [0][1]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[0].notification_samples[1].sample_size, 4,
            "Wrong sample size [0][1]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[0].notification_samples[1].data,
            vec![9, 0, 0, 0],
            "Wrong data [0][1]"
        );

        assert_eq!(
            notification_data.ads_stamp_headers[1]
                .notification_samples
                .len(),
            2,
            "Wrong notification sample vec len [1]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[1].samples, 2,
            "Wrong notification samples count [1]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[1].time_stamp, 255,
            "Wrong time stamp [1]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[1].notification_samples[0].notification_handle, 4,
            "Wrong notification handle [1][0]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[1].notification_samples[0].sample_size, 2,
            "Wrong sample size [1][0]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[1].notification_samples[0].data,
            vec![6, 0],
            "Wrong data [1][0]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[1].notification_samples[1].notification_handle, 4,
            "Wrong notification handle [1][1]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[1].notification_samples[1].sample_size, 4,
            "Wrong sample size [1][1]"
        );
        assert_eq!(
            notification_data.ads_stamp_headers[1].notification_samples[1].data,
            vec![9, 0, 0, 0],
            "Wrong data [1][1]"
        );
    }
}
