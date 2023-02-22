//! Parsing Edgetech JSF files
use crate::records::{Channel, SonarDataRecord};
use binrw::io;
use binrw::{binread, BinRead, BinResult};

use time::{Duration, OffsetDateTime};

/// A struct representing a message in a JSF file
#[binread]
#[br(little, magic = b"\x01\x16")]
#[derive(Debug, PartialEq)]
pub struct Message {
    protocol: u8,
    session_identifier: u8,
    message_type: u16,
    command_type: u8,
    subsystem_number: u8,
    channel_number: u8,
    #[br(pad_after = 2)]
    sequence_number: u8,
    message_size: i32,
    #[br(args {message_size, message_type})]
    data: MessageType,
}

impl Message {
    fn channel(&self) -> Channel {
	match self.channel_number {
	    0 => Channel::Port,
	    1 => Channel::Starboard,
	    _ => Channel::Other
	}
    }
}

#[binread]
#[br(little,import {message_size:i32})]
#[derive(Debug, PartialEq)]
pub struct UnknownMessage {
    #[br(count=message_size)]
    data: Vec<u8>,
}

#[binread]
#[br(little,import {message_size:i32})]
#[derive(Debug, PartialEq)]
pub struct SystemInformation {
    system_type: i32,
    low_rate_io: i32,
    version_number: i32,
    n_subsystems: i32,
    n_serial_ports: i32,
    #[br(pad_after=message_size-24)]
    serial_number: i32,
}

#[binread]
#[br(little)]
#[derive(Debug, PartialEq)]
pub struct NavigationOffsets {
    x: f32,
    y: f32,
    latitude: f32,
    longitude: f32,
    aft: f32,
    starboard: f32,
    depth: f32,
    altitude: f32,
    heading: f32,
    pitch: f32,
    roll: f32,
    yaw: f32,
    #[br(pad_after = 12)]
    tow_point_elevation: f32,
}

#[binread]
#[br(little,import {message_size:i32})]
#[derive(Debug, PartialEq)]
pub struct SonarData {
    time: i32,
    starting_depth: u32,
    #[br(pad_after = 4)]
    ping_number: u32,
    msbs: u16,
    lsb1: u16,
    #[br(pad_after = 6)]
    lsb2: u16,
    id_code: i16,
    #[br(pad_after = 2)]
    validity_flag: u16,
    data_format: i16,
    aft_antenna_distance: i16,
    #[br(pad_after = 4)]
    starboard_antenna_distance: i16,
    km_pipe: f32,
    #[br(pad_after = 24)]
    heave: f32,
    gap_filler_lateral_offset: f32,
    x_position: i32,
    y_position: i32,
    coordinate_units: i16,
    #[br(count = 24)]
    annotation_string: Vec<u8>,
    samples: u16,
    sampling_interval: u32,
    adc_gain_factor: u16,
    #[br(pad_after = 2)]
    transmit_level: i16,
    start_frequency: u16,
    end_frequency: u16,
    sweep_length: u16,
    pressure: i32,
    depth: i32,
    sample_frequency: u16,
    outgoing_pulse_identifier: u16,
    altitude: i32,
    sound_speed: f32,
    mixer_frequency: f32,
    year: i16,
    day: i16,
    hour: i16,
    minute: i16,
    second: i16,
    time_basis: i16,
    weighting_factor: i16,
    number_of_pulses: i16,
    heading: u16,
    pitch: i16,
    #[br(pad_after = 2)]
    roll: i16,
    #[br(pad_before = 2)]
    trigger_source: i16,
    mark_number: i16,
    position_fix_hour: i16,
    position_fix_minutes: i16,
    position_fix_seconds: i16,
    course: i16,
    speed: i16,
    position_fix_day: i16,
    position_fix_year: i16,
    milliseconds_today: u32,
    #[br(pad_after = 4)]
    max_adc_value: u16,
    #[br(count = 6)]
    software_version_number: Vec<u8>,
    spherical_correction_factor: i32,
    packet_number: u16,
    #[br(pad_after = 2)]
    adc_decimation: i16,
    temperature: i16,
    #[br(pad_after = 4)]
    layback: f32,
    #[br(pad_after = 2)]
    cable_out: u16,
    #[br(count=(message_size-240)>>1)]
    trace: Vec<u16>,
}

impl SonarData {
    pub fn timestamp(&self) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(i64::from(self.time)).unwrap()
            + Duration::milliseconds(i64::from(self.milliseconds_today % 1000))
    }

    pub fn mixer_frequency(&self) -> f64 {
        f64::from(self.mixer_frequency)
    }

    pub fn sampling_interval(&self) -> f64 {
        1e-9 * f64::from(self.sampling_interval)
    }

    pub fn trace(&self) -> &Vec<u16> {
        &self.trace
    }
}

#[binread]
#[br(little,import {message_size:i32})]
#[derive(Debug, PartialEq)]
pub struct NMEAString {
    time: i32,
    milliseconds: i32,
    #[br(pad_after = 3)]
    source: u8,
    #[br(count=message_size-12)]
    data: Vec<u8>,
}

#[binread]
#[br(little)]
#[derive(Debug, PartialEq)]
pub struct PitchRollData {
    time: i32,
    #[br(pad_after = 4)]
    milliseconds: i32,
    acceleration_x: i16,
    acceleration_y: i16,
    acceleration_z: i16,
    gyro_rate_x: i16,
    gyro_rate_y: i16,
    gyro_rate_z: i16,
    pitch: i16,
    roll: i16,
    temperature: i16,
    device_info: u16,
    heave: i16,
    heading: u16,
    validity_flag: i32,
    #[br(pad_after = 2)]
    yaw: i16,
}

impl PitchRollData {
    pub fn timestamp(&self) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(i64::from(self.time)).unwrap()
            + Duration::milliseconds(i64::from(self.milliseconds % 1000))
    }

    fn is_acceleration_x_valid(&self) -> bool {
        self.validity_flag & 0x0001 == 1
    }

    fn is_acceleration_y_valid(&self) -> bool {
        ((self.validity_flag & 0x0002) >> 1) == 1
    }

    fn is_acceleration_z_valid(&self) -> bool {
        ((self.validity_flag & 0x0004) >> 2) == 1
    }

    fn is_gyro_rate_x_valid(&self) -> bool {
        ((self.validity_flag & 0x0008) >> 3) == 1
    }

    fn is_gyro_rate_y_valid(&self) -> bool {
        ((self.validity_flag & 0x0010) >> 4) == 1
    }

    fn is_gyro_rate_z_valid(&self) -> bool {
        ((self.validity_flag & 0x0020) >> 5) == 1
    }

    fn is_pitch_valid(&self) -> bool {
        ((self.validity_flag & 0x0040) >> 6) == 1
    }

    fn is_roll_valid(&self) -> bool {
        ((self.validity_flag & 0x0080) >> 7) == 1
    }

    fn is_heave_valid(&self) -> bool {
        ((self.validity_flag & 0x0100) >> 8) == 1
    }

    fn is_heading_valid(&self) -> bool {
        ((self.validity_flag & 0x0200) >> 9) == 1
    }

    fn is_temperature_valid(&self) -> bool {
        ((self.validity_flag & 0x0400) >> 10) == 1
    }

    fn is_device_info_valid(&self) -> bool {
        ((self.validity_flag & 0x0800) >> 11) == 1
    }

    fn is_yaw_valid(&self) -> bool {
        ((self.validity_flag & 0x1000) >> 12) == 1
    }

    pub fn acceleration(&self) -> Option<(f64, f64, f64)> {
        if self.is_acceleration_x_valid()
            && self.is_acceleration_y_valid()
            && self.is_acceleration_z_valid()
        {
            Some((
                f64::from(self.acceleration_x) * 20.0 * 1.5 / 32768.0,
                f64::from(self.acceleration_y) * 20.0 * 1.5 / 32768.0,
                f64::from(self.acceleration_z) * 20.0 * 1.5 / 32768.0,
            ))
        } else {
            None
        }
    }

    pub fn gyro_rate(&self) -> Option<(f64, f64, f64)> {
        if self.is_gyro_rate_x_valid() && self.is_gyro_rate_y_valid() && self.is_gyro_rate_z_valid()
        {
            Some((
                f64::from(self.gyro_rate_x) * 500.0 * 1.5 / 32768.0,
                f64::from(self.gyro_rate_y) * 500.0 * 1.5 / 32768.0,
                f64::from(self.gyro_rate_z) * 500.0 * 1.5 / 32768.0,
            ))
        } else {
            None
        }
    }

    pub fn pitch(&self) -> Option<f64> {
        if self.is_pitch_valid() {
            Some(f64::from(self.pitch) * 180.0 / 32768.0)
        } else {
            None
        }
    }
    pub fn roll(&self) -> Option<f64> {
        if self.is_roll_valid() {
            Some(f64::from(self.roll) * 180.0 / 32768.0)
        } else {
            None
        }
    }

    pub fn temperature(&self) -> Option<f64> {
        if self.is_temperature_valid() {
            Some(f64::from(self.temperature) * 0.1)
        } else {
            None
        }
    }

    pub fn device_info(&self) -> Option<u16> {
        if self.is_device_info_valid() {
            Some(self.device_info)
        } else {
            None
        }
    }

    pub fn heading(&self) -> Option<f64> {
        if self.is_heading_valid() {
            Some(f64::from(self.heading) * 0.01)
        } else {
            None
        }
    }

    pub fn heave(&self) -> Option<f64> {
        if self.is_heave_valid() {
            Some(f64::from(self.heave) / 1000.0)
        } else {
            None
        }
    }

    pub fn yaw(&self) -> Option<f64> {
        if self.is_yaw_valid() {
            Some(f64::from(self.yaw) * 0.01)
        } else {
            None
        }
    }
}

#[binread]
#[br(import {message_type:u16,
	     message_size:i32})]
#[derive(Debug, PartialEq)]
pub enum MessageType {
    #[br(pre_assert(message_type==80))]
    M80 {
        #[br(args {message_size})]
        msg: SonarData,
    },
    #[br(pre_assert(message_type==2020))]
    M2020 { msg: PitchRollData },
    #[br(pre_assert(message_type==181))]
    M181 { msg: NavigationOffsets },
    #[br(pre_assert(message_type==182))]
    M182 {
        #[br(args {message_size})]
        msg: SystemInformation,
    },
    #[br(pre_assert(message_type==2002))]
    M2002 {
        #[br(args {message_size})]
        msg: NMEAString,
    },
    M0 {
        #[br(args {message_size})]
        msg: UnknownMessage,
    },
}

pub struct JSFFile<T: io::Read + io::Seek> {
    pub reader: T,
}

impl<T: io::Read + io::Seek> Iterator for JSFFile<T> {
    type Item = BinResult<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = Message::read(&mut self.reader);
        match res {
            Ok(msg) => Some(Ok(msg)),
            Err(e) => {
                if e.is_eof() {
                    None
                } else {
                    Some(Err(e))
                }
            }
        }
    }
}

// SonarDataRecord interface
impl From<Message> for SonarDataRecord<u16> {
    fn from(msg: Message) -> Self {
        let md = &msg.data;
        match md {
            MessageType::M80 { msg: mt } => SonarDataRecord::Ping(crate::records::Ping::new(
                "unknown".to_string(),
                mt.timestamp(),
                mt.mixer_frequency(),
                mt.sampling_interval(),
                msg.channel(),
                mt.trace().to_vec(),
            )),
            MessageType::M2020 { msg: mt } => {
                SonarDataRecord::Orientation(crate::records::Orientation::new(
                    "unknown".to_string(),
                    mt.timestamp(),
                    mt.pitch(),
                    mt.roll(),
                    mt.heading(),
                ))
            }
            _ => SonarDataRecord::Unknown,
        }
    }
}
