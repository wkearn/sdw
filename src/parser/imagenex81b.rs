//! Parsing Imagenex 81b files
use crate::model::{Channel, SonarDataRecord};
use binrw::{binread, io, BinRead, BinResult, NullString};

use time::{OffsetDateTime, PrimitiveDateTime};

/// An Imagenex881B rotary sonar shot
#[binread]
#[br(big, magic = b"81B")]
#[derive(Debug, PartialEq)]
pub struct Shot {
    n_to_read_index: u8,
    total_bytes: u16,
    n_to_read: u16,
    #[br(pad_size_to = 12)]
    dd: NullString,
    #[br(pad_size_to = 9)]
    tt: NullString,
    #[br(pad_size_to = 4)]
    hh: NullString,
    sample_rate: u8,
    #[br(pad_after = 2)]
    extended_bytes: u8,
    dir: u8,
    start_gain: u8,
    sector_size: u8,
    train_angle: u8,
    range_offset: u8,
    absorption: u8,
    profile_grid: u8,
    pulse_length: u8,
    profile: u8,
    velocity: u16,
    #[br(pad_size_to = 32)]
    user_text: NullString,
    frequency: u16,
    #[br(pad_after = 7)]
    azimuth_drive_head: u16,
    #[br(pad_after = 7)]
    vertical_angle_offset: u16,
    #[br(magic = b"I")]
    ix: u8,
    #[br(magic = b"X")]
    head_id: u8,
    serial_status: u8,
    head_position: u16,
    range: u8,
    profile_range: u16,
    data_bytes_lo: u8,
    data_bytes_hi: u8,
    #[br(count=(u16::from(data_bytes_hi) << 7)| u16::from(data_bytes_lo))]
    echo_data: Vec<u8>,
    #[br(pad_after=if ix == 0x4d { 19 } else { 63 },assert(termination_byte==0xfc))]
    termination_byte: u8,
}

impl Shot {
    fn timestamp(&self) -> OffsetDateTime {
        let format = time::format_description::parse(
            "[day]-[month repr:short case_sensitive:false]-[year][hour]:[minute]:[second]",
        )
        .unwrap();
        let s = std::str::from_utf8(&self.dd)
            .unwrap()
            .trim_end_matches('\0')
            .to_owned()
            + std::str::from_utf8(&self.tt)
                .unwrap()
                .trim_end_matches('\0')
            + std::str::from_utf8(&self.hh)
                .unwrap()
                .trim_end_matches('\0');
        PrimitiveDateTime::parse(&s, &format).unwrap().assume_utc()
    }

    fn frequency(&self) -> f64 {
        1000.0 * f64::from(self.frequency)
    }

    fn sampling_interval(&self) -> f64 {
        0.0
    }
}

impl From<Shot> for SonarDataRecord<u8> {
    fn from(shot: Shot) -> Self {
        SonarDataRecord::Ping(crate::model::Ping::new(
            "unknown".to_string(),
            shot.timestamp(),
            shot.frequency(),
            shot.sampling_interval(),
            Channel::Other,
            shot.echo_data,
        ))
    }
}

/// An iterator interface to an Imagenex .81b file
pub struct File<T: io::Read + io::Seek> {
    /// The reader from which bytes are read and parsed
    reader: T,
}

impl<T> File<T>
where
    T: io::Read + io::Seek,
{
    /// Create an Imagenex 81b file from a reader
    pub fn new(reader: T) -> Self {
        File { reader }
    }
}

impl<T> io::Read for File<T>
where
    T: io::Read + io::Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl<T> io::Seek for File<T>
where
    T: io::Read + io::Seek,
{
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.reader.seek(pos)
    }
}

impl<T: io::Read + io::Seek> Iterator for File<T> {
    type Item = BinResult<Shot>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = Shot::read(&mut self.reader);
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
