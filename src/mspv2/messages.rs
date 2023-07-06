pub const INAV_ANALOG: u16 = 0x2002;

#[derive(Debug)]
pub(crate) enum MspV2Request {
    Request(u16),
}

#[derive(Debug)]
pub(crate) enum MspV2Response {
    InavAnalog(InavAnalogMessage),
}

#[derive(Debug)]
pub(crate) struct InavAnalogMessage {
    pub(crate) battery_flags: u8,
    pub(crate) battery_voltage: u16,
    pub(crate) amperage: u16,
    pub(crate) power: u32,
    pub(crate) mah_drawn: u32,
    pub(crate) mwh_drawn: u32,
    pub(crate) battery_remaining_capacity: u32,
    pub(crate) battery_percentage: u8,
    pub(crate) rssi: u16,
}
