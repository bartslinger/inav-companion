mod mspv2;
use futures::{SinkExt, StreamExt};
use mspv2::{MspV2Codec, MspV2Request};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    run_serial_link().await;
}

async fn run_serial_link() {
    let port = "/dev/cu.usbserial-0001";
    // let port = "/dev/cu.usbserial-AB0JSZ6R";
    let serial = tokio_serial::new(port, 9600).open_native_async().unwrap();

    let codec = MspV2Codec {};
    let (mut sender, mut receiver) = Framed::new(serial, codec).split();

    loop {
        let item = MspV2Request::Request(mspv2::INAV_ANALOG);
        tokio::select! {
            _write_result = async {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                sender.send(item).await
            } => {},
            item = receiver.next() => {
                println!("Item: {:#?}", item);
            }
        }
    }

    // loop {
    //     // Very much MVP, I just want to get some state
    //     // Get analog stuff (includes RSSI!)
    //     let flag = 0;
    //     let function = 0x2002;
    //     let size = 0;

    //     let mut data: Vec<u8> = Vec::new();
    //     WriteBytesExt::write_u8(&mut data, flag).unwrap();
    //     WriteBytesExt::write_u16::<LittleEndian>(&mut data, function).unwrap();
    //     WriteBytesExt::write_u16::<LittleEndian>(&mut data, size).unwrap();

    //     let mut crc = CRC::crc8dvb_s2();
    //     crc.digest(&data);
    //     let crc_result = u8::try_from(crc.get_crc()).unwrap();

    //     println!("Hoi {:x}", crc_result);

    //     // Prepare the message
    //     let mut message = vec![];
    //     message.extend_from_slice(b"$X<");
    //     message.append(&mut data);
    //     WriteBytesExt::write_u8(&mut message, crc_result).unwrap();

    //     println!("Message: {:x?}", message);
    //     let write_result = port.write(&message).await;
    //     println!("{:?}", write_result);

    //     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    //     let mut buf = [0; 33];
    //     let read_result = tokio::time::timeout(
    //         tokio::time::Duration::from_millis(200),
    //         port.read(&mut buf[..]),
    //     )
    //     .await;
    //     println!("Read result: {:?}", read_result);
    //     println!("Buffer: {:x?}", buf);
    //     let rssi = byteorder::LittleEndian::read_u16(&buf[30..32]);
    //     println!("RSSI: {}", rssi);

    //     //tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    // }
}
