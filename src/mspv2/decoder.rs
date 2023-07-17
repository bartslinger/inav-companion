use byteorder::{ByteOrder, LittleEndian};
use bytes::BytesMut;
use tokio_util::codec::Decoder;

use super::{
    messages::MspV2Response, MspV2Codec, ALTITUDE, INAV_ANALOG, INAV_MISC2, RAW_GPS, SET_RAW_RC,
};

impl Decoder for MspV2Codec {
    type Item = MspV2Response;
    type Error = std::io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Only support V2
        // As a minimum, we need 3+1+2+2+1 = 9 bytes
        if src.len() < 9 {
            return Ok(None);
        }

        // Search for "$X>"
        let pattern = "$X>".as_bytes();
        let start_index = match src.windows(3).position(|window| window == pattern) {
            None => return Ok(None),
            Some(v) => v,
        };

        // Truncate everything up to the start index
        let rejected = src.split_to(start_index);
        if !rejected.is_empty() {
            println!("Rejected {:x?}", rejected);
        }

        // Check if this is a valid message
        // Do we have sufficent bytes a message with 0 payload?
        if src.len() < 9 {
            return Ok(None);
        }

        let size = LittleEndian::read_u16(&src[6..8]) as usize;
        // Check if the we have all the bytes for a message of this size
        if src.len() < (9 + size) {
            // TODO: If the size number is really large by error, don't get stuck here
            // Instead, look for the next occurance of the header
            return Ok(None);
        }

        // Check the CRC
        let mut crc = crc_any::CRC::crc8dvb_s2();
        // crc includes flag, function, size and payload
        crc.digest(&src[3..(9 + size - 1)]);
        let crc_result = crc.get_crc() as u8;
        if crc_result != src[8 + size] {
            // TODO: If the size number is really large by error, don't get stuck here
            // Instead, look for the next occurance of the header
            println!("Invalid CRC");
            let _ = src.split_to(size + 9);
            return Ok(None);
        }

        let _flag = src[3];
        let function = LittleEndian::read_u16(&src[4..6]);

        // At this point, we have all the bytes that belong to the message
        // Take the data from the src
        let data = src.split_to(size + 9);
        // Create reference to the payload
        let payload = &data[8..];

        let cfg = bincode::config();
        match function {
            RAW_GPS => {
                let message: crate::mspv2::RawGpsMessage = cfg.deserialize(payload).unwrap();
                Ok(Some(MspV2Response::RawGps(message)))
            }
            ALTITUDE => {
                let message: crate::mspv2::AltitudeMessage = cfg.deserialize(payload).unwrap();
                Ok(Some(MspV2Response::Altitude(message)))
            }
            INAV_ANALOG => {
                let message: crate::mspv2::InavAnalogMessage = cfg.deserialize(payload).unwrap();
                Ok(Some(MspV2Response::InavAnalog(message)))
            }
            INAV_MISC2 => {
                let message: crate::mspv2::InavMisc2Message = cfg.deserialize(payload).unwrap();
                Ok(Some(MspV2Response::InavMisc2(message)))
            }
            SET_RAW_RC => Ok(Some(MspV2Response::SetRawRcAck)),
            _ => Ok(None),
        }
    }
}

fn format_bytes(bytes: &BytesMut) -> String {
    let mut s = String::new();
    s.push_str("&[");
    for (i, byte) in bytes.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&format!("0x{:02x}", byte));
    }
    s.push(']');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_analog() {
        let mut codec = MspV2Codec {};

        let mut src = BytesMut::new();
        src.extend_from_slice(&[
            0x24, 0x58, 0x3e, 0x00, 0x02, 0x20, 0x18, 0x00, 0x20, 0x2b, 0x03, 0x2d, 0x00, 0x6c,
            0x01, 0x00, 0x00, 0x47, 0x00, 0x00, 0x00, 0x42, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x53, 0x00, 0x00, 0x79,
        ]);
        let result = codec.decode(&mut src).unwrap();
        assert_eq!(
            result,
            Some(MspV2Response::InavAnalog(crate::mspv2::InavAnalogMessage {
                battery_flags: 0x20,
                battery_voltage: 0x032b,
                amperage: 0x002d,
                power: 0x0000016c,
                mah_drawn: 0x00000047,
                mwh_drawn: 0x00000242,
                battery_remaining_capacity: 0x00000000,
                battery_percentage: 0x53,
                rssi: 0x0000,
            }))
        );
    }
}
