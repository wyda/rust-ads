use byteorder::{LittleEndian, WriteBytesExt};
use num_traits::{FromPrimitive,ToPrimitive};
use std::convert::Into;
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream, UdpSocket};
/// Implementation of BECKHOFF's ADS protocol
use std::result;

pub const MAXDATALEN: usize = 8192;

/// 48898 ADS-Protocol port
pub const ADS_TCP_SERVER_PORT: u16 = 0xBF02;

pub type VirtualConnection = (u16, AmsAddress);

pub trait SizedData {
    fn data_len(&self) -> u32;

    fn data(&self) -> &[u8];

    fn data_info(&self) -> (u32, &[u8]) {
        (self.data_len(), self.data())
    }

    fn read_write_len(&self) -> Option<(u32, u32)> {
        None
    }
}

pub trait AmsProxy {
    // TODO
    fn delete_notification(&mut self);
}

#[derive(Debug, PartialEq, Clone)]
pub enum State {
    OPEN,
    FAILED,
    CLOSED,
    UNKNOWN,
}

/// All possible Ads Errors
//TODO refactor
#[derive(Debug, PartialEq)]
pub enum AdsError {
    // TODO use official error codes
    InvalidAddress,
    ConnectionError,
    SyncError,
    PortAlreadyInUse(u16),
    IOError,
    TargetNotReachable,
    NotFound,
    BadStreamNotConnected,
    NoMemoryLeft,
    BadPort(u16),
    PortNotOpen(u16),
}

pub type Result<T> = result::Result<T, AdsError>;

/// TODO check whether need packing without offsets#[repr(packed)]
pub struct AdsTcpHeader {
    reserved: u16,
    length: u32,
}

/// The ADS Net ID
/// addresses the transmitter or receiver
/// The AMS Net ID is composed of the TCP/IP of the local computer plus the suffix ".1.1".
/// The AMS Net ID is based on the TCP/IP address, but the relationship is not entirely fixed.
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct AmsNetId {
    b: [u8; 6],
}

impl From<[u8; 6]> for AmsNetId {
    fn from(b: [u8; 6]) -> AmsNetId {
        AmsNetId { b }
    }
}

impl AmsNetId {
    /// create a new AmsNetId from a str input
    pub fn parse(s: &str) -> Result<AmsNetId> {
        let parts: Vec<&str> = s.split(".").collect();
        if parts.len() != 6 {
            return Err(AdsError::InvalidAddress);
        }
        let mut b = [0; 6];
        for (i, p) in parts.iter().enumerate() {
            match p.parse::<u8>() {
                Ok(v) => b[i] = v,
                Err(_) => return Err(AdsError::InvalidAddress),
            }
        }
        // // the AmsNetId should end with ".1.1"
        // // this is not mandatory
        // for i in 4..6 {
        //     if b[i] != 1 {
        //         return Err(AdsError::InvalidAddress);
        //     }
        // }
        Ok(AmsNetId { b })
    }

    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> AmsNetId {
        AmsNetId {
            b: [a, b, c, d, e, f],
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.b
    }
}

impl Into<Ipv4Addr> for AmsNetId {
    ///create a new Ipv4Addr based on an AmsNetId; just drop the last 2 bytes(1.1)
    fn into(self) -> Ipv4Addr {
        Ipv4Addr::from([self.b[0], self.b[1], self.b[2], self.b[3]])
    }
}

/// create a new AmsNetId based on an Ipv4Addr; adds two additional bytes ([1,1]) to the octects
impl Into<AmsNetId> for Ipv4Addr {
    fn into(self) -> AmsNetId {
        let o = self.octets();
        AmsNetId::new(o[0], o[1], o[2], o[3], 1, 1)
    }
}

pub trait ToAmsId {
    fn to_ams_id(&self) -> Result<AmsNetId>;
}

impl ToAmsId for str {
    fn to_ams_id(&self) -> Result<AmsNetId> {
        AmsNetId::parse(self)
    }
}

impl ToAmsId for String {
    fn to_ams_id(&self) -> Result<AmsNetId> {
        AmsNetId::parse(self.as_ref())
    }
}

// impl Into<Result<AmsNetId>> for &str {
//     fn into(self) -> Result<AmsNetId> {
//         AmsNetId::parse(self)
//     }
// }

///
#[derive(Debug, PartialEq, Clone)]
pub struct AmsHeader {
    target_id: AmsNetId,
    target_port: u16,
    source_id: AmsNetId,
    source_port: u16,
    command_id: AdsCommandId,
    state_flag: AdsStateFlag,
    /// the size of the data in the ADS packet in bytes
    data_length: u32,
    error_code: u32,
    invoke_id: u32,
}

impl AmsHeader {
    pub fn source_addr(&self) -> AmsAddress {
        AmsAddress::new(self.source_id.clone(), self.source_port)
    }

    pub fn target_addr(&self) -> AmsAddress {
        AmsAddress::new(self.target_id.clone(), self.target_port)
    }

    ///
    // TODO lets return a Result instead
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(32);
        wtr.extend(self.target_id.as_bytes());
        wtr.write_u16::<LittleEndian>(self.target_port).unwrap();
        wtr.extend(self.source_id.as_bytes());
        wtr.write_u16::<LittleEndian>(self.source_port).unwrap();
        wtr.write_u16::<LittleEndian>(self.command_id.to_u16().unwrap())
            .unwrap();
        wtr.write_u16::<LittleEndian>(self.state_flag.to_u16().unwrap())
            .unwrap();
        wtr.write_u32::<LittleEndian>(self.data_length).unwrap();
        wtr.write_u32::<LittleEndian>(self.error_code).unwrap();
        wtr.write_u32::<LittleEndian>(self.invoke_id).unwrap();
        wtr
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AmsRequestHeader {
    group: u32,
    offset: u32,
    length: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AmsResponseHeader {
    result: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AmsReadResponseHeader {
    result: u32,
    read_length: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AmsAddress {
    pub net_id: AmsNetId,
    pub port: u16, // the ads port number
}

impl AmsAddress {
    pub fn new(net_id: AmsNetId, port: u16) -> Self {
        AmsAddress { net_id, port }
    }
}

/// state flags
pub const SF_ADS_REQ_RESP: u32 = 0x0001;
pub const SF_ADS_COMMAND: u32 = 0x0004;

// TODO do we need this ads_data buffer? Can't we parse directly?
pub struct AdsPacket {
    // 6 bytes
    ads_tcp_header: AdsTcpHeader,
    // 32 bytes
    ams_header: AmsHeader,
    ads_data: [u8; MAXDATALEN], // contains the data
}

#[derive(Debug, PartialEq, Clone, FromPrimitive)]
pub enum IndexGroup {
    /// READ_M - WRITE_M
    Memorybyte = 0x4020,
    /// plc memory area (%M), offset means byte-offset
    /// READ_MX - WRITE_MX
    Memorybit = 0x4021,
    /// plc memory area (%MX), offset means the bit adress, calculatedb by bytenumber * 8 + bitnumber
    /// PLCADS_IGR_RMSIZE
    Memorysize = 0x4025,
    /// size of the memory area in bytes
    /// PLCADS_IGR_RWRB
    Retain = 0x4030,
    /// plc retain memory area, offset means byte-offset
    /// PLCADS_IGR_RRSIZE
    Retainsize = 0x4035,
    /// size of the retain area in bytes
    /// LCADS_IGR_RWDB
    Data = 0x4040,
    /// data area, offset means byte-offset
    /// PLCADS_IGR_RDSIZE
    Datasize = 0x4045, // size of the data area in bytes
}

///

#[derive(Debug, PartialEq, Clone, ToPrimitive)]
#[repr(u16)]
pub enum AdsStateFlag {
    AmsRequest = 0x0004,
    AmsResponse = 0x0005,
    AmsUdp = 0x0040,
    INVALID = 0x0000,
}
/// ADS Commands

/// Command ids
#[derive(Debug, PartialEq, Clone, ToPrimitive)]
#[repr(u16)]
pub enum AdsCommandId {
    AdsInvalid = 0x0000,
    AdsReadDeviceInfo = 0x0001,
    AdsRead = 0x0002,
    AdsWrite = 0x0003,
    AdsReadState = 0x0004,
    AdsWriteControl = 0x0005,
    AdsAddDeviceNotification = 0x0006,
    AdsDeleteDeviceNotification = 0x0007,
    AdsDeviceNotification = 0x0008,
    AdsReadWrite = 0x0009,
}

#[derive(Debug, PartialEq, Clone, FromPrimitive)]
pub enum AdsPortNumber {
    Logger = 100,
    EventLogger = 110,
    IO = 300,
    AdditionalTask1 = 301,
    AdditionalTask2 = 302,
    NC = 500,
    PlcRuntimeSystem1 = 801,
    PlcRuntimeSystem2 = 811,
    PlcRuntimeSystem3 = 821,
    PlcRuntimeSystem4 = 831,
    CamshaftController = 900,
    SystemService = 10000,
    Scope = 14000,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AdsState {
    Run = 5,
    Stop = 6,
}

/// ADS Device Notification
/// Request: `The data which are transfered at the Device Notification are multiple nested into one another.
/// The Notification Stream contains an array with elements of type AdsStampHeader.
/// This array again contains elements of type AdsNotificationSample.`
// TODO what's the difference between length and stamps?
#[derive(Debug, PartialEq)]
pub struct AdsNotificationStream {
    length: u32,
    /// amount of stamps in the
    stamps: u32,
    // number of AdsStampHeaders in the ads_stamp_header field
    ads_stamp_header: AdsStampHeader,
}

#[derive(Debug, PartialEq)]
pub struct AdsStampHeader {
    timestamp: u64,
    samples: u32,
    // number of AdsNotificationSamples in the ads_notification_filed
    ads_notification_filed: AdsNotificationSample,
}

#[derive(Debug, PartialEq)]
pub struct AdsNotificationSample {
    notification_handle: u32,
    sample_size: u32,
    data: Vec<u8>,
}

impl SizedData for AdsNotificationSample {
    fn data_len(&self) -> u32 {
        self.sample_size
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AdsVersion {
    version: u8,
    revision: u8,
    build: u16,
}

#[derive(Debug, PartialEq, Clone, FromPrimitive)]
pub enum AdsTransmode {
    AdsTransNotrans = 0,
    AdsTransClientcycle = 1,
    AdsTransClientoncha = 2,
    AdsTransServercycle = 3,
    AdsTransServeroncha = 4,
    AdsTransServercycle2 = 5,
    AdsTransServeroncha2 = 6,
    AdsTransClient1req = 10,
    AdsTransMaxmodes,
}

#[cfg(test)]
mod tests {
    use core::ads::*;
    use std::net::Ipv4Addr;
    #[test]
    fn parse_ams_net_id() {
        let mut id1 = AmsNetId::new(127, 0, 0, 1, 1, 1);
        let id2 = AmsNetId::parse("127.0.0.1.1.1");
        assert_eq!(Ok(id1), id2);
        id1 = AmsNetId::from([127, 0, 0, 1, 1, 1]);
        assert_eq!(Ok(id1), id2);
    }

    #[test]
    fn into_ams_net_id() {
        let id1 = "127.0.0.1.1.1".to_ams_id();
        let id2 = "127.0.0.1.1.1".to_string().to_ams_id();
        assert_eq!(id1, id2);
    }

    #[test]
    fn into_ipv4_and_ams_id() {
        let id1 = AmsNetId::new(127, 0, 0, 1, 1, 1);
        let ipv4 = Ipv4Addr::new(127, 0, 0, 1);
        assert_eq!(Into::<Ipv4Addr>::into(id1.clone()), ipv4);
        assert_eq!(Into::<AmsNetId>::into(ipv4), id1);
    }

    // #[test]
    // fn serde_ams_net_id() {
    //     let id1 = AmsNetId::new(127, 0, 0, 1, 1, 1);
    //     let encoded: Vec<u8> = serialize(&id1).unwrap();
    //     let v = vec![127, 0, 0, 1, 1, 1];
    //     assert_eq!(&v, &encoded);
    //     let id2: AmsNetId = deserialize(&v[..]).unwrap();
    //     assert_eq!(id1, id2);
    // }
}
