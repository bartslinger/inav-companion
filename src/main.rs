mod messages;
mod mspv2;
mod webserver;

use crate::messages::InavMessage;
use crate::webserver::run_webserver;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use mspv2::{MspV2Codec, MspV2Request, MspV2Response};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let (websocket_broadcast_channel, _) = tokio::sync::broadcast::channel(100);
    let webserver_broadcast_channel = websocket_broadcast_channel.subscribe();
    tokio::select! {
        _ = run_webserver(webserver_broadcast_channel)=> {},
        _ = run_serial_link(websocket_broadcast_channel) => {},
    };
}

pub(crate) enum GetResponseError {
    Timeout,
    DecoderError,
    StreamDepleted,
}

async fn get_response(
    receiver: &mut SplitStream<Framed<SerialStream, MspV2Codec>>,
) -> Result<MspV2Response, GetResponseError> {
    match tokio::time::timeout(tokio::time::Duration::from_millis(100), receiver.next()).await {
        Ok(Some(Ok(v))) => {
            match &v {
                MspV2Response::RawGps(x) => {
                    println!("{:#?}", x);
                }
                MspV2Response::Altitude(x) => {
                    println!("{:#?}", x);
                }
                MspV2Response::InavAnalog(x) => {
                    println!("{:#?}", x);
                }
                MspV2Response::InavMisc2(x) => {
                    println!("{:#?}", x);
                }
            };
            Ok(v)
        }
        Ok(Some(Err(_))) => Err(GetResponseError::DecoderError),
        Ok(None) => Err(GetResponseError::StreamDepleted),
        Err(_) => {
            println!("timeout");
            Err(GetResponseError::Timeout)
        }
    }
}

async fn run_serial_link(broadcast_channel: tokio::sync::broadcast::Sender<InavMessage>) {
    // let port = "/dev/cu.usbserial-0001";
    let port = "/dev/serial0";
    // let port = "/dev/cu.usbserial-AB0JSZ6R";
    let serial = tokio_serial::new(port, 9600).open_native_async().unwrap();

    let codec = MspV2Codec {};
    let (mut sender, mut receiver) = Framed::new(serial, codec).split();

    loop {
        sender
            .send(MspV2Request::Request(mspv2::RAW_GPS))
            .await
            .ok();
        if let Ok(v) = get_response(&mut receiver).await {
            let _ = broadcast_channel.send(v.into());
        }
        sender
            .send(MspV2Request::Request(mspv2::ALTITUDE))
            .await
            .ok();
        if let Ok(v) = get_response(&mut receiver).await {
            let _ = broadcast_channel.send(v.into());
        }
        sender
            .send(MspV2Request::Request(mspv2::INAV_ANALOG))
            .await
            .ok();
        if let Ok(v) = get_response(&mut receiver).await {
            let _ = broadcast_channel.send(v.into());
        }
        sender
            .send(MspV2Request::Request(mspv2::INAV_MISC2))
            .await
            .ok();
        if let Ok(v) = get_response(&mut receiver).await {
            let _ = broadcast_channel.send(v.into());
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
