use byteorder::{LittleEndian, WriteBytesExt};
use bytes::{BufMut, BytesMut};
use crc_any::CRC;
// use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_util::codec::{Decoder, Encoder};
// use crc_any::CRC;

pub(crate) enum MspV2Message {
    InavAnalog,
}

pub(crate) struct MspV2Codec {}

impl Encoder<MspV2Message> for MspV2Codec {
    type Error = std::io::Error;

    fn encode(&mut self, item: MspV2Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut data: Vec<u8> = vec![];
        data.write_u8(0).unwrap(); // flag
        match item {
            MspV2Message::InavAnalog => {
                data.write_u16::<LittleEndian>(0x2002).unwrap();
                data.write_u16::<LittleEndian>(0).unwrap();
            }
        }
        // calculate crc
        let mut crc = CRC::crc8dvb_s2();
        crc.digest(&data);
        let crc_result = crc.get_crc() as u8;
        dst.reserve(data.len() + 4); // 3 start bytes and CRC
        dst.extend_from_slice(b"$X<");
        dst.extend_from_slice(&data);
        dst.put_u8(crc_result);
        println!("Dst: {:x?}", dst);

        Ok(())
    }
}

impl Decoder for MspV2Codec {
    type Item = MspV2Message;
    type Error = std::io::Error;
    fn decode(&mut self, _src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Ok(None)
    }
}
