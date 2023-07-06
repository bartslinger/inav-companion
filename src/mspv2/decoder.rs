use bytes::BytesMut;
use tokio_util::codec::Decoder;

use super::{messages::MspV2Response, MspV2Codec};

impl Decoder for MspV2Codec {
    type Item = MspV2Response;
    type Error = std::io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Only support V2
        // As a minimum, we need 3+1+2+2+1 = 9 bytes
        if src.len() <= 9 {
            return Ok(None);
        }
        println!("src: {:02x}\n\n", src);
        // println!("src: {}", src.len());
        Ok(None)
    }
}
