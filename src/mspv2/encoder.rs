use crate::mspv2::SET_RAW_RC;
use byteorder::{LittleEndian, WriteBytesExt};
use bytes::{BufMut, BytesMut};
use tokio_util::codec::Encoder;

use super::{messages::MspV2Request, MspV2Codec};

impl Encoder<MspV2Request> for MspV2Codec {
    type Error = std::io::Error;

    fn encode(&mut self, item: MspV2Request, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut data: Vec<u8> = vec![];
        data.write_u8(0).unwrap(); // flag
        match item {
            MspV2Request::Request(function) => {
                data.write_u16::<LittleEndian>(function).unwrap();
                data.write_u16::<LittleEndian>(0).unwrap();
            }
            MspV2Request::SetRawRc(channels) => {
                data.write_u16::<LittleEndian>(SET_RAW_RC).unwrap();
                data.write_u16::<LittleEndian>(2 * channels.len() as u16)
                    .unwrap();
                channels.iter().for_each(|channel| {
                    data.write_u16::<LittleEndian>(*channel).unwrap();
                })
            }
        }

        let mut crc = crc_any::CRC::crc8dvb_s2();
        crc.digest(&data);
        let crc_result = crc.get_crc() as u8;

        dst.reserve(data.len() + 4); // 3 start bytes and CRC
        dst.extend_from_slice(b"$X<");
        dst.extend_from_slice(&data);
        dst.put_u8(crc_result);

        Ok(())
    }
}
