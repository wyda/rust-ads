use crate::error::AmsAddressError;
use crate::proto::request::WriteTo;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{self, Error, Write};

pub struct AmsAddress {
    ams_net_id: AmsNetId,
    port: u16,
}

impl AmsAddress {
    fn new(ams_net_id: AmsNetId, port: u16) -> Self {
        AmsAddress { ams_net_id, port }
    }
}

impl WriteTo for AmsAddress {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        self.ams_net_id.write_to(&mut wtr);
        wtr.write_u16::<LittleEndian>(self.port)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AmsNetId {
    net_id: [u8; 6],
}

impl WriteTo for AmsNetId {
    fn write_to<W: Write>(&self, mut wtr: W) -> io::Result<()> {
        wtr.write_all(&self.net_id[..])?;
        Ok(())
    }
}

impl From<[u8; 6]> for AmsNetId {
    fn from(net_id: [u8; 6]) -> AmsNetId {
        AmsNetId { net_id }
    }
}

impl AmsNetId {
    /// create a new AmsNetId from a str input
    pub fn parse(net_id: &str) -> Result<AmsNetId, AmsAddressError> {
        let parts: Vec<&str> = net_id.split('.').collect();
        if parts.len() != 6 {
            return Err(AmsAddressError::InvalidAddressLength {
                length: parts.len(),
            });
        }
        let mut net_id = [0; 6];
        for (i, p) in parts.iter().enumerate() {
            match p.parse::<u8>() {
                Ok(v) => net_id[i] = v,
                Err(e) => return Err(AmsAddressError::ParseError { source: e }),
            }
        }

        Ok(AmsNetId { net_id })
    }

    #[allow(clippy::many_single_char_names)]
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> AmsNetId {
        AmsNetId {
            net_id: [a, b, c, d, e, f],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ams_net_id_new_test() {
        let ams_net_id = AmsNetId::new(192, 168, 1, 1, 1, 1);
        assert_eq!(ams_net_id.net_id, [192, 168, 1, 1, 1, 1]);
    }

    #[test]
    fn ams_net_id_from_test() {
        let ams_net_id = AmsNetId::from([192, 168, 1, 1, 1, 1]);
        assert_eq!(ams_net_id.net_id, [192, 168, 1, 1, 1, 1]);
    }

    #[test]
    fn ams_net_id_parse_test() {
        let ams_net_id = AmsNetId::parse("192.168.1.1.1.1").unwrap();
        assert_eq!(ams_net_id.net_id, [192, 168, 1, 1, 1, 1]);

        let ams_parse_error = AmsNetId::parse("192.168.1.1.1.1.1").unwrap_err();
        assert_eq!(
            ams_parse_error,
            AmsAddressError::InvalidAddressLength { length: 7 }
        );

        let ams_parse_error = AmsNetId::parse("999.168.1.1.1.1").unwrap_err();
        //assert_eq!(ams_parse_error, AmsAddressError::ParseError{kind: std::num::IntErrorKind::PosOverflow});
    }

    #[test]
    fn ams_address_new_test() {
        let ams_net_id = AmsNetId::parse("192.168.1.1.1.1").unwrap();
        let port = 30000;
        let ams_address = AmsAddress::new(ams_net_id.clone(), port);

        assert_eq!(ams_address.port, port);
        assert_eq!(ams_address.ams_net_id.net_id, ams_net_id.net_id);
    }

    #[test]
    fn ams_address_write_to_test() {
        let ams_net_id = AmsNetId::parse("192.168.1.1.1.1").unwrap();
        let port = 30000;
        let ams_address = AmsAddress::new(ams_net_id.clone(), port);

        let mut buffer: Vec<u8> = Vec::new();
        ams_address.write_to(&mut buffer);

        assert_eq!(buffer, [192, 168, 1, 1, 1, 1, 48, 117]);
    }
}
