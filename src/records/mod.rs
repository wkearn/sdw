use crate::channel::Channel;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Deserialize, Serialize)]
pub enum SonarDataRecord<T> {
    Ping {
        source: String,
        timestamp: OffsetDateTime,
        frequency: f64,
        sampling_interval: f64,
        channel: Channel,
        data: Vec<T>,
    },
    Position {
        source: String,
        timestamp: OffsetDateTime,
        longitude: Option<f64>,
        latitude: Option<f64>,
        altitude: Option<f64>,
    },
    Orientation {
        source: String,
        timestamp: OffsetDateTime,
        pitch: Option<f64>,
        roll: Option<f64>,
        heading: Option<f64>,
    },
    Course {
        source: String,
        timestamp: OffsetDateTime,
        speed: Option<f64>,
        heading: Option<f64>,
    },
    Unknown,
}
