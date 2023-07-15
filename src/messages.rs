use crate::mspv2;
use crate::mspv2::MspV2Response;

#[derive(Clone, serde::Serialize, Debug)]
pub(crate) struct TimestampedInavMessage {
    pub(crate) ts: i64,
    #[serde(flatten)]
    pub(crate) msg: InavMessage,
}

impl From<MspV2Response> for TimestampedInavMessage {
    fn from(value: MspV2Response) -> Self {
        let now = chrono::Utc::now();
        Self {
            ts: now.timestamp_millis(),
            msg: value.into(),
        }
    }
}

#[derive(Clone, serde::Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum InavMessage {
    // Outgoing (to browser)
    RawGps(RawGpsMessage),
    Altitude(AltitudeMessage),
    InavAnalog(InavAnalogMessage),
    InavMisc2(InavMisc2Message),
}

impl From<MspV2Response> for InavMessage {
    fn from(value: MspV2Response) -> Self {
        match value {
            MspV2Response::RawGps(v) => InavMessage::RawGps(v.into()),
            MspV2Response::Altitude(v) => InavMessage::Altitude(v.into()),
            MspV2Response::InavAnalog(v) => InavMessage::InavAnalog(v.into()),
            MspV2Response::InavMisc2(v) => InavMessage::InavMisc2(v.into()),
        }
    }
}

#[derive(Clone, serde::Serialize, Debug)]
pub(crate) struct RawGpsMessage {
    fix_type: GpsFixType,
    satellites: u8,
    lat: f64,
    lon: f64,
    alt: f32,
    ground_speed: f32,
    ground_course: f32,
    hdop: f32,
}

#[derive(Clone, serde::Serialize, Debug)]
pub(crate) enum GpsFixType {
    #[serde(rename = "NO_FIX")]
    NoFix,
    #[serde(rename = "FIX_2D")]
    Fix2D,
    #[serde(rename = "FIX_3D")]
    Fix3D,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl From<mspv2::RawGpsMessage> for RawGpsMessage {
    fn from(value: mspv2::RawGpsMessage) -> Self {
        let fix_type = match value.fix_type {
            0 => GpsFixType::NoFix,
            1 => GpsFixType::Fix2D,
            2 => GpsFixType::Fix3D,
            _ => GpsFixType::Unknown,
        };
        Self {
            fix_type,
            satellites: value.num_sat,
            lat: f64::from(value.lat) / 10000000.0,
            lon: f64::from(value.lon) / 10000000.0,
            alt: f32::from(value.alt),
            ground_speed: f32::from(value.ground_speed) / 100.0,
            ground_course: f32::from(value.ground_course) / 10.0,
            hdop: f32::from(value.hdop) / 100.0,
        }
    }
}

#[derive(Clone, serde::Serialize, Debug)]
pub(crate) struct AltitudeMessage {
    estimated_z_position: f32,
    estimated_z_velocity: f32,
    baro_altitude: f32,
}
impl From<mspv2::AltitudeMessage> for AltitudeMessage {
    fn from(value: mspv2::AltitudeMessage) -> Self {
        Self {
            estimated_z_position: value.estimated_z_position as f32 / 100.0,
            estimated_z_velocity: f32::from(value.estimated_z_velocity) / 100.0,
            baro_altitude: value.baro_altitude as f32 / 100.0,
        }
    }
}

#[derive(Clone, serde::Serialize, Debug)]
pub(crate) struct InavAnalogMessage {
    battery_voltage: f32,
    amperage: f32,
    power: f32,
    battery_percentage: f32,
    rssi: f32,
}

impl From<mspv2::InavAnalogMessage> for InavAnalogMessage {
    fn from(value: mspv2::InavAnalogMessage) -> Self {
        Self {
            battery_voltage: f32::from(value.battery_voltage) / 100.0,
            amperage: f32::from(value.amperage) / 100.0,
            power: value.power as f32 / 100.0,
            battery_percentage: f32::from(value.battery_percentage),
            rssi: f32::from(value.rssi) * 100.0 / 1023.0,
        }
    }
}

#[derive(Clone, serde::Serialize, Debug)]
pub(crate) struct InavMisc2Message {
    on_time: u32,
    flight_time: u32,
    throttle_percent: u8,
    auto_throttle_flag: bool,
}

impl From<mspv2::InavMisc2Message> for InavMisc2Message {
    fn from(value: mspv2::InavMisc2Message) -> Self {
        Self {
            on_time: value.on_time,
            flight_time: value.flight_time,
            throttle_percent: value.throttle_percent,
            auto_throttle_flag: value.auto_throttle_flag == 1,
        }
    }
}

#[derive(Clone, serde::Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum IncomingWebsocketMessage {
    SetRawRc(SetRawRcMessage),
}

#[derive(Clone, serde::Deserialize, Debug)]
pub(crate) struct SetRawRcMessage {
    pub(crate) ts: i64,
    pub(crate) channels: [u16; 4],
}
