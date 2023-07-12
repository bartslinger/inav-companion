pub const RAW_GPS: u16 = 106;
pub const ALTITUDE: u16 = 109;
pub const INAV_ANALOG: u16 = 0x2002;
pub const INAV_MISC2: u16 = 0x203A;

#[derive(Debug)]
pub(crate) enum MspV2Request {
    Request(u16),
}

#[derive(Debug, PartialEq)]
pub(crate) enum MspV2Response {
    RawGps(RawGpsMessage),
    Altitude(AltitudeMessage),
    InavAnalog(InavAnalogMessage),
    InavMisc2(InavMisc2Message),
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct RawGpsMessage {
    pub(crate) fix_type: u8,
    pub(crate) num_sat: u8,
    pub(crate) lat: i32,
    pub(crate) lon: i32,
    pub(crate) alt: i16,
    pub(crate) ground_speed: i16,
    pub(crate) ground_course: i16,
    pub(crate) hdop: u16,
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct AltitudeMessage {
    pub(crate) estimated_z_position: i32,
    pub(crate) estimated_z_velocity: i16,
    pub(crate) baro_altitude: i32,
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct InavAnalogMessage {
    pub(crate) battery_flags: u8,
    pub(crate) battery_voltage: u16,
    pub(crate) amperage: i16,
    pub(crate) power: i32,
    pub(crate) mah_drawn: i32,
    pub(crate) mwh_drawn: i32,
    pub(crate) battery_remaining_capacity: u32,
    pub(crate) battery_percentage: u8,
    pub(crate) rssi: u16,
}

#[derive(Debug, PartialEq, serde::Deserialize)]
pub(crate) struct InavMisc2Message {
    pub(crate) on_time: u32,
    pub(crate) flight_time: u32,
    pub(crate) throttle_percent: u8,
    pub(crate) auto_throttle_flag: u8,
}
