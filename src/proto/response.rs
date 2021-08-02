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

impl WriteTo for Response {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        match self {
            Response::ReadDeviceInfo(w) => w.write_to(&mut wtr),
            Response::Read(w) => w.write_to(&mut wtr),
            Response::Write(w) => w.write_to(&mut wtr),
            Response::ReadState(w) => w.write_to(&mut wtr),
            Response::WriteControl(w) => w.write_to(&mut wtr),
            Response::AddDeviceNotification(w) => w.write_to(&mut wtr),
            Response::DeleteDeviceNotification(w) => w.write_to(&mut wtr),
            Response::DeviceNotification(w) => w.write_to(&mut wtr),
            Response::ReadWrite(w) => w.write_to(&mut wtr),
        }
    }
}

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
    result: AdsError,
}

impl ReadFrom for WriteResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let result = AdsError::from(read.read_u32::<LittleEndian>()?);
        Ok(Self { result })
    }
}

impl WriteTo for WriteResponse {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.result.as_u32());
        Ok(())
    }
}

impl WriteResponse {
    pub fn new(result: AdsError) -> Self {
        WriteResponse { result }
    }
}

/// ADS Read State
#[derive(Debug, PartialEq, Clone)]
pub struct ReadStateResponse {
    result: AdsError,
    ads_state: AdsState,
    device_state: u16,
}

impl ReadFrom for ReadStateResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        Ok(Self {
            result: AdsError::from(read.read_u32::<LittleEndian>()?),
            ads_state: AdsState::read_from(read)?,
            device_state: read.read_u16::<LittleEndian>()?,
        })
    }
}

impl WriteTo for ReadStateResponse {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.result.as_u32());
        self.ads_state.write_to(&mut wtr);
        wtr.write_u16::<LittleEndian>(self.device_state);
        Ok(())
    }
}

impl ReadStateResponse {
    pub fn new(result: AdsError, ads_state: AdsState, device_state: u16) -> Self {
        ReadStateResponse {
            result,
            ads_state,
            device_state,
        }
    }
}

///Write control
#[derive(Debug, PartialEq, Clone)]
pub struct WriteControlResponse {
    result: AdsError,
}

impl ReadFrom for WriteControlResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        Ok(Self {
            result: AdsError::from(read.read_u32::<LittleEndian>()?),
        })
    }
}

impl WriteTo for WriteControlResponse {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.result.as_u32());
        Ok(())
    }
}

impl WriteControlResponse {
    pub fn new(result: AdsError) -> Self {
        WriteControlResponse { result }
    }
}

/// ADS Add Device Notification
#[derive(Debug, PartialEq, Clone)]
pub struct AddDeviceNotificationResponse {
    result: AdsError,
    notification_handle: u32,
}
impl ReadFrom for AddDeviceNotificationResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        Ok(Self {
            result: AdsError::from(read.read_u32::<LittleEndian>()?),
            notification_handle: read.read_u32::<LittleEndian>()?,
        })
    }
}

impl WriteTo for AddDeviceNotificationResponse {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.result.as_u32());
        wtr.write_u32::<LittleEndian>(self.notification_handle);
        Ok(())
    }
}

impl AddDeviceNotificationResponse {
    pub fn new(result: AdsError, notification_handle: u32) -> Self {
        AddDeviceNotificationResponse {
            result,
            notification_handle,
        }
    }
}

/// ADS Delete Device Notification
#[derive(Debug, PartialEq, Clone)]
pub struct DeleteDeviceNotificationResponse {
    result: AdsError,
}
impl ReadFrom for DeleteDeviceNotificationResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        Ok(Self {
            result: AdsError::from(read.read_u32::<LittleEndian>()?),
        })
    }
}

impl WriteTo for DeleteDeviceNotificationResponse {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.result.as_u32());
        Ok(())
    }
}

impl DeleteDeviceNotificationResponse {
    pub fn new(result: AdsError) -> Self {
        DeleteDeviceNotificationResponse { result }
    }
}

//ADS Device Notification Response
#[derive(Debug, PartialEq, Clone)]
pub struct AdsNotificationSample {
    notification_handle: u32,
    sample_size: u32,
    data: Vec<u8>,
}

impl AdsNotificationSample {
    pub fn new(notification_handle: u32, data: Vec<u8>) -> Self {
        AdsNotificationSample {
            notification_handle,
            sample_size: data.len() as u32,
            data,
        }
    }
    pub fn sample_len(&self) -> usize {
        //plus fixed byte length (notification_handle, sample_size)
        self.data.len() + 8
    }
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

impl WriteTo for AdsStampHeader {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u64::<LittleEndian>(self.time_stamp);
        wtr.write_u32::<LittleEndian>(self.samples);

        for sample in &self.notification_samples {
            wtr.write_u32::<LittleEndian>(sample.notification_handle);
            wtr.write_u32::<LittleEndian>(sample.sample_size);
            wtr.write_all(sample.data.as_slice());
        }
        Ok(())
    }
}

impl AdsStampHeader {
    pub fn new(
        time_stamp: u64,
        samples: u32,
        notification_samples: Vec<AdsNotificationSample>,
    ) -> Self {
        AdsStampHeader {
            time_stamp,
            samples,
            notification_samples,
        }
    }

    pub fn stamp_len(&self) -> usize {
        let mut len: usize = 0;
        for sample in &self.notification_samples {
            len += sample.sample_len();
        }
        //plus fixed byte size (time_stamp, samples)
        len + 12
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

impl WriteTo for AdsNotificationStream {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.length);
        wtr.write_u32::<LittleEndian>(self.stamps);

        for stamp_header in &self.ads_stamp_headers {
            stamp_header.write_to(&mut wtr);
        }
        Ok(())
    }
}

impl AdsNotificationStream {
    pub fn new(length: u32, stamps: u32, ads_stamp_headers: Vec<AdsStampHeader>) -> Self {
        AdsNotificationStream {
            length,
            stamps,
            ads_stamp_headers,
        }
    }

    pub fn stream_len(&self) -> usize {
        let mut len: usize = 0;
        for stamp in &self.ads_stamp_headers {
            len += stamp.stamp_len();
        }
        //plus fixed byte size (length, stamps)
        len + 8
    }
}

//Asd Read response
#[derive(Debug)]
pub struct ReadResponse {
    result: AdsError,
    length: u32,
    data: Vec<u8>,
}

impl ReadFrom for ReadResponse {
    fn read_from<R: Read>(read: &mut R) -> io::Result<Self> {
        let result = AdsError::from(read.read_u32::<LittleEndian>()?);
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

impl WriteTo for ReadResponse {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_u32::<LittleEndian>(self.result.as_u32())?;
        wtr.write_u32::<LittleEndian>(self.length)?;
        wtr.write_all(self.data.as_slice());
        Ok(())
    }
}

impl ReadResponse {
    pub fn new(result: AdsError, data: Vec<u8>) -> Self {
        ReadResponse {
            result,
            length: data.len() as u32,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn read_device_info_response_write_to_test() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut device_name: [u8; 16] = [0; 16];

        for (n, b) in "Device".as_bytes().iter().enumerate() {
            device_name[n] = *b;
        }

        let device_info_response =
            ReadDeviceInfoResponse::new(AdsError::ErrAccessDenied, 1, 2, 10, device_name);

        let mut response_data: Vec<u8> = vec![
            30, 0, 0, 0, 1, 2, 10, 0, 68, 101, 118, 105, 99, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        device_info_response.write_to(&mut buffer);

        assert_eq!(buffer, response_data);
    }

    #[test]
    fn read_device_info_response_test() {
        let mut response_data: Vec<u8> = vec![
            30, 0, 0, 0, 2, 14, 1, 1, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 0, 0, 0,
            0, 0,
        ];

        let read_device_info_response =
            ReadDeviceInfoResponse::read_from(&mut response_data.as_slice()).unwrap();

        let response = Response::ReadDeviceInfo(read_device_info_response.clone());

        assert_eq!(read_device_info_response.result, AdsError::ErrAccessDenied);
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

        assert_eq!(read_response.result, AdsError::ErrInsertMailBox);
        assert_eq!(read_response.length, 2);
        assert_eq!(read_response.data, vec![255, 2]);
    }

    #[test]
    fn read_response_write_to_test() {
        let mut buffer: Vec<u8> = Vec::new();
        let data: u32 = 90000;
        let mut read_response =
            ReadResponse::new(AdsError::ErrAccessDenied, data.to_le_bytes().to_vec());
        read_response.write_to(&mut buffer);
        assert_eq!(buffer, [30, 0, 0, 0, 4, 0, 0, 0, 144, 95, 1, 0]);
    }

    #[test]
    fn write_response_test() {
        let mut response_data: Vec<u8> = vec![4, 0, 0, 0];

        let write_response = WriteResponse::read_from(&mut response_data.as_slice()).unwrap();

        assert_eq!(write_response.result, AdsError::from(4));
    }

    #[test]
    fn write_response_write_to_test() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut write_response = WriteResponse::new(AdsError::ErrAccessDenied);
        write_response.write_to(&mut buffer);
        assert_eq!(buffer, [30, 0, 0, 0]);
    }

    #[test]
    fn read_state_response_test() {
        let mut response_data: Vec<u8> = vec![4, 0, 0, 0, 9, 0, 1, 1];

        let read_state_response =
            ReadStateResponse::read_from(&mut response_data.as_slice()).unwrap();

        assert_eq!(read_state_response.result, AdsError::ErrInsertMailBox);
        assert_eq!(
            read_state_response.ads_state,
            AdsState::AdsStatePowerFailure
        );
        assert_eq!(read_state_response.device_state, 257);
    }

    #[test]
    fn read_state_response_write_to_test() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut read_state_response =
            ReadStateResponse::new(AdsError::ErrAccessDenied, AdsState::AdsStateConfig, 4);
        read_state_response.write_to(&mut buffer);
        assert_eq!(buffer, [30, 0, 0, 0, 15, 0, 4, 0]);
    }

    #[test]
    fn write_control_response_test() {
        let mut response_data: Vec<u8> = vec![30, 0, 0, 0];

        let write_control_response =
            WriteControlResponse::read_from(&mut response_data.as_slice()).unwrap();

        assert_eq!(write_control_response.result, AdsError::ErrAccessDenied);
    }

    #[test]
    fn write_control_response_write_to_test() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut write_control_response = WriteControlResponse::new(AdsError::ErrAccessDenied);
        write_control_response.write_to(&mut buffer);
        assert_eq!(buffer, [30, 0, 0, 0]);
    }

    #[test]
    fn add_device_notification_response_test() {
        let mut response_data: Vec<u8> = vec![4, 0, 0, 0, 10, 0, 0, 0];

        let add_device_notification_response =
            AddDeviceNotificationResponse::read_from(&mut response_data.as_slice()).unwrap();

        assert_eq!(
            add_device_notification_response.result,
            AdsError::ErrInsertMailBox
        );
        assert_eq!(add_device_notification_response.notification_handle, 10);
    }

    #[test]
    fn add_device_notification_response_write_to_test() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut add_device_notification_response =
            AddDeviceNotificationResponse::new(AdsError::ErrInsertMailBox, 10);
        add_device_notification_response.write_to(&mut buffer);
        assert_eq!(buffer, [4, 0, 0, 0, 10, 0, 0, 0]);
    }

    #[test]
    fn delete_device_notification_response_test() {
        let mut response_data: Vec<u8> = vec![4, 0, 0, 0];

        let delete_device_notification_response =
            DeleteDeviceNotificationResponse::read_from(&mut response_data.as_slice()).unwrap();

        assert_eq!(
            delete_device_notification_response.result,
            AdsError::ErrInsertMailBox
        );
    }

    #[test]
    fn delete_device_notification_response_write_to_test() {
        let mut buffer: Vec<u8> = Vec::new();
        let mut delete_device_notification_response =
            DeleteDeviceNotificationResponse::new(AdsError::ErrAccessDenied);
        delete_device_notification_response.write_to(&mut buffer);
        assert_eq!(buffer, [30, 0, 0, 0]);
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

#[test]
fn ads_notification_stream_write_to_test() {
    //4+4+4=12byte
    let sample_data1: u32 = 1000;
    let notification_sample1 = AdsNotificationSample {
        notification_handle: 10,
        sample_size: 4,
        data: sample_data1.to_le_bytes().to_vec(),
    };

    //4+4+2=10byte
    let sample_data2: u16 = 2000;
    let notification_sample2 = AdsNotificationSample {
        notification_handle: 20,
        sample_size: 2,
        data: sample_data2.to_le_bytes().to_vec(),
    };

    //4+4+8=16byte
    let sample_data3: u64 = 3000;
    let notification_sample3 = AdsNotificationSample {
        notification_handle: 30,
        sample_size: 8,
        data: sample_data3.to_le_bytes().to_vec(),
    };

    //8+4+12+10=34byte
    let mut notification_samples = Vec::new();
    notification_samples.push(notification_sample1);
    notification_samples.push(notification_sample2);
    let stamp_header1 = AdsStampHeader::new(1234567890, 2, notification_samples);

    //8+4+16=28byte
    let mut notification_samples = Vec::new();
    notification_samples.push(notification_sample3);
    let stamp_header2 = AdsStampHeader::new(1234567890, 1, notification_samples);

    let mut stamp_headers = Vec::new();
    stamp_headers.push(stamp_header1);
    stamp_headers.push(stamp_header2);

    let mut len: usize = 0;
    for header in &stamp_headers {
        len += header.stamp_len();
    }

    let expected_len: usize = 62;
    assert_eq!(&len, &expected_len, "Wrong number of bytes");

    //4+4+34+28=70byte
    let ads_notification_stream =
        AdsNotificationStream::new(len as u32, stamp_headers.len() as u32, stamp_headers);

    let expected_len: usize = 70;
    assert_eq!(
        &ads_notification_stream.stream_len(),
        &expected_len,
        "Wrong number of bytes"
    );

    let mut buffer: Vec<u8> = Vec::new();

    ads_notification_stream.write_to(&mut buffer);

    #[rustfmt::skip]
    let expected_data = [
        //Notification stream Length
        62, 0, 0, 0,
        ////Notification stream number of stamps
        2, 0, 0, 0, 
        //Stamp header1 time_stamp
        210, 2, 150, 73, 0, 0, 0, 0, 
        //Stamp header1 number of samples
        2, 0, 0, 0, 
        //Notification sample 1 notification handle
        10, 0, 0, 0, 
        //Notification sample 1 sample size
        4, 0, 0, 0,
        //Notification sample 1 data
        232, 3, 0, 0, 
        //Notification sample 2 notification handle
        20, 0, 0, 0, 
        //Notification sample 2 sample size
        2, 0, 0, 0, 
        //Notification sample 2 data
        208, 7, 
        //Stamp header2 time_stamp
        210, 2, 150, 73, 0, 0, 0, 0, 
        //Stamp header2 number of samples
        1, 0, 0, 0, 
        //Notification sample 3 notification handle
        30, 0, 0, 0,
        //Notification sample 3 sample size
        8, 0, 0, 0, 
        //Notification sample 3 data
        184, 11, 0, 0, 0, 0, 0, 0,
    ];

    assert_eq!(buffer, expected_data, "Data in buffer is not as expected");
}
