mod messages;
mod mspv2;
mod webserver;

use crate::messages::{InavMessage, SetRawRcMessage, TimestampedInavMessage};
use crate::webserver::run_webserver;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use mspv2::{MspV2Codec, MspV2Request, MspV2Response};
use tokio::time::MissedTickBehavior;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let (websocket_broadcast_channel, _) = tokio::sync::broadcast::channel(100);
    let webserver_broadcast_channel = websocket_broadcast_channel.subscribe();
    let (raw_rc_mpsc_tx, mut raw_rc_mpsc_rx) = tokio::sync::mpsc::channel(10);
    let (raw_rc_watch_tx, raw_rc_watch_rx) = tokio::sync::watch::channel(SetRawRcMessage {
        ts: 0,
        channels: [1500, 1500, 1000, 1500, 1500],
    });

    // This aggregator takes values from multiple websocket connections and presents the latest
    // value on the watch channel
    let raw_rc_aggregator = async move {
        while let Some(v) = raw_rc_mpsc_rx.recv().await {
            match raw_rc_watch_tx.send(v) {
                Ok(()) => {}
                Err(_) => break,
            }
        }
    };

    tokio::select! {
        _ = run_webserver(webserver_broadcast_channel, raw_rc_mpsc_tx)=> {},
        _ = run_serial_link(websocket_broadcast_channel, raw_rc_watch_rx) => {},
        _ = raw_rc_aggregator => {}
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
                MspV2Response::SetRawRcAck => {
                    // println!("SetRawRc ACK");
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

async fn run_serial_link(
    broadcast_channel: tokio::sync::broadcast::Sender<TimestampedInavMessage>,
    mut raw_rc_channel_rx: tokio::sync::watch::Receiver<SetRawRcMessage>,
) {
    // let port = "/dev/cu.usbserial-0001";
    let port = "/dev/serial0";
    // let port = "/dev/cu.usbserial-AB0JSZ6R";
    let serial = tokio_serial::new(port, 9600).open_native_async().unwrap();

    let codec = MspV2Codec {};
    let (mut sender, mut receiver) = Framed::new(serial, codec).split();

    let telemetry = [
        mspv2::RAW_GPS,
        mspv2::ALTITUDE,
        mspv2::INAV_ANALOG,
        mspv2::INAV_MISC2,
    ];
    let mut x: usize = 0;
    let mut telemetry_interval = tokio::time::interval(tokio::time::Duration::from_millis(1000));
    telemetry_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        let mut raw_rc_changed_trigger = false;
        if x == 0 {
            tokio::select! {
                _ = raw_rc_channel_rx.changed() => {
                    raw_rc_changed_trigger = true;
                    // println!("x changed");
                },
                _ = telemetry_interval.tick() => {
                    // println!("+ tick");
                }
            }
        }

        if raw_rc_changed_trigger || raw_rc_channel_rx.has_changed().ok() == Some(true) {
            // println!("* rc changed");
            let current_value = raw_rc_channel_rx.borrow_and_update().clone();
            // Make sure the timestamp is not too old
            let now = chrono::Utc::now().timestamp_millis();
            let age = now - current_value.ts;
            if !(-500..=2000).contains(&age) {
                println!("raw rc age out of range {}", age);
            } else {
                sender
                    .send(MspV2Request::SetRawRc([
                        current_value.channels[0],
                        current_value.channels[1],
                        current_value.channels[2],
                        current_value.channels[3],
                        current_value.channels[4],
                    ]))
                    .await
                    .ok();
                let _ = get_response(&mut receiver).await;
            }
        } else {
            // Do 1 non-RC transaction
            if let Some(request) = telemetry.get(x) {
                // println!("# {}", x);
                sender.send(MspV2Request::Request(*request)).await.ok();
            }
            if let Ok(v) = get_response(&mut receiver).await {
                let inav_message: TimestampedInavMessage = v.into();
                match inav_message.msg {
                    InavMessage::Ack => {}
                    _ => {
                        let _send_result = broadcast_channel.send(inav_message);
                    }
                }
            }

            x += 1;
        }

        if x >= telemetry.len() {
            x = 0;
        }
    }
}
