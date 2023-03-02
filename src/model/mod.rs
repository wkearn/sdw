//! The SDW data model
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// The channel for multi-channel sensors
///
/// This distinguishes pings to the port side
/// from those to starboard. All other channels are
/// currently represented as the Other variant.
#[derive(Debug, Clone, Copy)]
#[derive(Deserialize, Serialize)]
pub enum Channel {
    /// A ping to port
    Port,
    /// A ping to starboard
    Starboard,
    /// Some other channel
    Other,
}

/// A representation of a sonar ping
///
#[derive(Debug)]
#[derive(Deserialize, Serialize)]
pub struct Ping<T> {
    /// The source of the sonar data
    pub source: String,
    /// The time at which the ping was acquired
    ///
    /// This should typically be the start of the ping
    #[serde(with = "time::serde::timestamp")]
    pub timestamp: OffsetDateTime,
    /// The frequency of the sonar system
    pub frequency: f64,
    /// The sampling interval of the data
    pub sampling_interval: f64,
    /// The channel (Port, Starboard, Other)
    pub channel: Channel,
    /// The ping data
    pub data: Vec<T>,
}

impl<T> Ping<T> {
    /// Create a new Ping from the given data
    pub fn new(
        source: String,
        timestamp: OffsetDateTime,
        frequency: f64,
        sampling_interval: f64,
        channel: Channel,
        data: Vec<T>,
    ) -> Ping<T> {
        Ping {
            source,
            timestamp,
            frequency,
            sampling_interval,
            channel,
            data,
        }
    }
}

/// The position of a sensor
#[derive(Debug)]
#[derive(Deserialize, Serialize)]
pub struct Position {
    /// The source of the position information
    pub source: String,
    /// The time at which the data were acquired
    #[serde(with = "time::serde::timestamp")]
    pub timestamp: OffsetDateTime,
    /// The longitude in degrees
    pub longitude: Option<f64>,
    /// The latitude in degrees
    pub latitude: Option<f64>,
    /// The altitude in meters relative to a chosen datum
    ///
    /// The vertical datum is not specified as part of the Position
    /// data type and must be handled by the user.
    pub altitude: Option<f64>,
}

impl Position {
    /// Create a new Position from the given data
    pub fn new(
        source: String,
        timestamp: OffsetDateTime,
        longitude: Option<f64>,
        latitude: Option<f64>,
        altitude: Option<f64>,
    ) -> Position {
        Position {
            source,
            timestamp,
            longitude,
            latitude,
            altitude,
        }
    }
}

/// The orientation of the sensor
#[derive(Debug)]
#[derive(Deserialize, Serialize)]
pub struct Orientation {
    /// The source of the orientation data
    pub source: String,
    /// The timestamp at which the data were acquired
    #[serde(with = "time::serde::timestamp")]
    pub timestamp: OffsetDateTime,
    /// The pitch of the sensor in degrees (bow up is positive)
    pub pitch: Option<f64>,
    /// The roll of the sensor in degrees (roll to starboard is positive)
    pub roll: Option<f64>,
    /// The heading of the sensor in degrees east of North
    pub heading: Option<f64>,
}

impl Orientation {
    /// Create a new Orientation from the given data
    pub fn new(
        source: String,
        timestamp: OffsetDateTime,
        pitch: Option<f64>,
        roll: Option<f64>,
        heading: Option<f64>,
    ) -> Orientation {
        Orientation {
            source,
            timestamp,
            pitch,
            roll,
            heading,
        }
    }
}

/// The course of the sensor
#[derive(Debug)]
#[derive(Deserialize, Serialize)]
pub struct Course {
    /// The source of the course information
    pub source: String,
    /// The time at which the data were acquired
    #[serde(with = "time::serde::timestamp")]
    pub timestamp: OffsetDateTime,
    /// The speed of the sensor in m/s
    pub speed: Option<f64>,
    /// The heading or track made good of the sensor in
    /// degrees east of North
    pub heading: Option<f64>,
}

/// A SonarDataRecord encapsulates the data available to SDW
#[derive(Debug)]
#[derive(Deserialize, Serialize)]
pub enum SonarDataRecord<T> {
    /// A wrapper for a Ping
    Ping(Ping<T>),
    /// A wrapper for a Position
    Position(Position),
    /// A wrapper for an Orientation
    Orientation(Orientation),
    /// A wrapper for a Course
    Course(Course),
    /// An unknown data type used as a catchall
    Unknown,
}
