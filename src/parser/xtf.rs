//! Parsing XTF files
use binrw::{binread, BinRead, BinResult};
use std::io;

/// The XTFFileHeader
#[binread]
#[br(little, magic = b"\x7b")]
#[derive(Debug, PartialEq)]
pub struct FileHeader {
    system_type: u8,
    #[br(count = 8,try_map = |x: Vec<u8>| String::from_utf8(x))]
    recording_program_name: String,
    #[br(count = 8,try_map = |x: Vec<u8>| String::from_utf8(x))]
    recording_program_version: String,
    #[br(count = 16,try_map = |x: Vec<u8>| String::from_utf8(x))]
    sonar_name: String,
    sensors_type: u16,
    #[br(count = 64,try_map = |x: Vec<u8>| String::from_utf8(x))]
    note_string: String,
    #[br(count = 64,try_map = |x: Vec<u8>| String::from_utf8(x))]
    file_name: String,
    nav_units: u16,
    number_of_sonar_channels: u16,
    number_of_bathy_channels: u16,
    number_of_snippet_channels: u8,
    number_of_forward_look_arrays: u8,
    number_of_echo_strength_channels: u16,
    #[br(pad_after = 3)]
    number_of_interferometry_channels: u8,
    reference_point_height: f32,
    #[br(count = 12)]
    projection_type: Vec<u8>,
    #[br(count = 10)]
    spheroid_type: Vec<u8>,
    navigation_latency: i32,
    origin_y: f32,
    origin_x: f32,
    nav_offset_y: f32,
    nav_offset_x: f32,
    nav_offset_z: f32,
    nav_offset_yaw: f32,
    mru_offset_y: f32,
    mru_offset_x: f32,
    mru_offset_z: f32,
    mru_offset_yaw: f32,
    mru_offset_pitch: f32,
    mru_offset_roll: f32,
    #[br(count = 6)]
    chaninfos: Vec<ChanInfo>,
}

/// The ChanInfo struct
#[binread]
#[br(little)]
#[derive(Debug, PartialEq)]
pub struct ChanInfo {
    type_of_channel: u8,
    sub_channel_number: u8,
    correction_flags: u16,
    unipolar: u16,
    #[br(pad_after = 4)]
    bytes_per_sample: u16,
    #[br(count = 16,try_map = |x: Vec<u8>| String::from_utf8(x))]
    channel_name: String,
    volt_scale: f32,
    frequency: f32,
    horizontal_beam_angle: f32,
    tilt_angle: f32,
    beam_width: f32,
    offset_x: f32,
    offset_y: f32,
    offset_z: f32,
    offset_yaw: f32,
    offset_pitch: f32,
    offset_roll: f32,
    beams_per_array: u16,
    #[br(pad_after = 53)]
    sample_format: u8,
}

/// A directory of packet types
#[binread]
#[br(little, import {header_type: u8, num_chans_to_follow: u16})]
#[derive(Debug, PartialEq)]
pub enum PacketType {
    /// A packet for sidescan sonar data
    #[br(pre_assert(header_type==0))]
    Sonar(#[br(args {num_chans_to_follow} )] PingHeader),
    /// An unknown packet type.
    ///
    /// This is used as a fallback if no other packet succeeds
    Unknown,
}

/// An XTF data packet
///
/// This assumes that all packets start with fields that
/// describe the header type, channel number, number of channels,
/// and number of bytes in the packet, which all of the documented
/// packet types do. Manufacturer-specific packets may not follow
/// this structure, and parsing will fail for such packets.
#[binread]
#[br(little, magic = 64206u16)]
#[derive(Debug, PartialEq)]
pub struct Packet {
    header_type: u8,
    sub_channel_number: u8,
    #[br(pad_after = 4)]
    num_chans_to_follow: u16,
    num_bytes_this_record: u32,
    #[br(args {header_type, num_chans_to_follow},pad_size_to=num_bytes_this_record-14)]
    header: PacketType,
}

impl Packet {
    /// Return the name of the packet type
    pub fn packet_name(&self) -> String {
        match self.header {
            PacketType::Sonar(_) => "Sonar".to_string(),
            PacketType::Unknown => "Unknown".to_string(),
        }
    }
}

/// A header describing ping-specific information
///
/// Timing and navigation information is contained here. The
/// data for the ping are stored in a Vec<PingChanHeader> with
/// one element for each channel.
#[binread]
#[br(little,import {num_chans_to_follow: u16})]
#[derive(Debug, PartialEq)]
pub struct PingHeader {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    hseconds: u8,
    julian_day: u16,
    event_number: u32,
    ping_number: u32,
    sound_velocity: f32,
    #[br(pad_after = 4)]
    ocean_tide: f32,
    conductivity_freq: f32,
    temperature_freq: f32,
    pressure_freq: f32,
    pressure_temp: f32,
    conductivity: f32,
    water_temperature: f32,
    pressure: f32,
    computed_sound_velocity: f32,
    mag_x: f32,
    mag_y: f32,
    mag_z: f32,
    aux_val1: f32,
    aux_val2: f32,
    #[br(pad_after = 12)]
    aux_val3: f32,
    speed_log: f32,
    turbidity: f32,
    ship_speed: f32,
    ship_gyro: f32,
    ship_y_coordinate: f64,
    ship_x_coordinate: f64,
    ship_altitude: u16,
    ship_depth: u16,
    fix_time_hour: u8,
    fix_time_minute: u8,
    fix_time_second: u8,
    fix_time_hsecond: u8,
    sensor_speed: f32,
    kilometers_pipe: f32,
    sensor_y_coordinate: f64,
    sensor_x_coordinate: f64,
    sonar_status: u16,
    range_to_fish: u16,
    bearing_to_fish: u16,
    cable_out: u16,
    layback: f32,
    cable_tension: f32,
    sensor_depth: f32,
    sensor_primary_altitude: f32,
    sensor_aux_altitude: f32,
    sensor_pitch: f32,
    sensor_roll: f32,
    sensor_heading: f32,
    heave: f32,
    yaw: f32,
    attitude_time_tag: u32,
    dot: f32,
    nav_fix_milliseconds: u32,
    computer_clock_hour: u8,
    computer_clock_minute: u8,
    computer_clock_second: u8,
    computer_clock_hsec: u8,
    fish_position_delta_x: i16,
    fish_position_delta_y: i16,
    fish_position_error_code: u8,
    optional_offset: u32,
    #[br(pad_after = 6)]
    cable_out_hundredths: u8,
    #[br(count=num_chans_to_follow)]
    channel_data: Vec<PingChanHeader>,
}

/// A header describing ping- and channel-specific information
///
/// The actual sonar return data are stored as a SonarData wrapper
/// in the data field.
#[binread]
#[br(little)]
#[derive(Debug, PartialEq)]
pub struct PingChanHeader {
    channel_number: u16,
    downsample_method: u16,
    slant_range: f32,
    ground_range: f32,
    time_delay: f32,
    time_duration: f32,
    seconds_per_ping: f32,
    processing_flags: u16,
    frequency: u16,
    initial_gain_code: u16,
    gain_code: u16,
    bandwidth: u16,
    contact_number: u32,
    contact_classification: u16,
    contact_sub_number: u8,
    contact_type: u8,
    num_samples: u32,
    millivolt_scale: u16,
    contact_time_off_track: f32,
    #[br(pad_after = 1)]
    contact_close_number: u8,
    fixed_vsop: f32,
    #[br(pad_after = 4)]
    weight: i16,
    #[br(args {bytes_per_sample: 2, num_samples})]
    data: SonarData,
}

#[binread]
#[br(little, import {bytes_per_sample: u16, num_samples: u32})]
#[derive(Debug, PartialEq)]
/// An enum to dispatch different sonar data types
///
/// Warning: the bytes_per_sample field is currently hardcoded to 2,
/// so the data will always be 16 bit.
pub enum SonarData {
    /// 8 bit sonar data
    #[br(pre_assert(bytes_per_sample==1))]
    U8(#[br(count=num_samples)] Vec<u8>),
    /// 16 bit sonar data
    #[br(pre_assert(bytes_per_sample==2))]
    U16(#[br(count=num_samples)] Vec<u16>),
    /// 32 bit sonar data
    #[br(pre_assert(bytes_per_sample==4))]
    U32(#[br(count=num_samples)] Vec<u32>),
}

/// A representation of an XTF file on disk
pub struct File<T>
where
    T: io::Read + io::Seek,
{
    header: FileHeader,
    reader: T,
}

impl<T> File<T>
where
    T: io::Read + io::Seek,
{
    /// Create an XTF file from a reader
    pub fn new(mut reader: T) -> Self {
        let header = FileHeader::read(&mut reader).expect("Unable to read XTF file header");
        File { header, reader }
    }

    /// Return a reference to the FileHeader
    pub fn header(&self) -> &FileHeader {
        &self.header
    }
}

impl<T: io::Read + io::Seek> Iterator for File<T> {
    type Item = BinResult<Packet>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = Packet::read(&mut self.reader);
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
