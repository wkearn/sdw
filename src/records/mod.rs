use crate::channel::Channel;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Deserialize, Serialize)]
pub struct Ping<T> {
    source: String,
    #[serde(with = "time::serde::timestamp")]
    timestamp: OffsetDateTime,
    frequency: f64,
    sampling_interval: f64,
    channel: Channel,
    data: Vec<T>,
}

impl<T> Ping<T> {
    pub fn new(source:String,timestamp:OffsetDateTime,frequency:f64,sampling_interval:f64,channel:Channel,data:Vec<T>) -> Ping<T> {
	Ping {source,timestamp,frequency,sampling_interval,channel,data}
    }

    pub fn timestamp(&self) -> OffsetDateTime {
	self.timestamp
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Position {
    source: String,
    #[serde(with="time::serde::timestamp")]
    timestamp: OffsetDateTime,
    longitude: Option<f64>,
    latitude: Option<f64>,
    altitude: Option<f64>,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Orientation {
    source: String,
    #[serde(with="time::serde::timestamp")]
    timestamp: OffsetDateTime,
    pitch: Option<f64>,
    roll: Option<f64>,
    heading: Option<f64>,
}

impl Orientation {
    pub fn new(source:String,timestamp:OffsetDateTime,pitch:Option<f64>,roll:Option<f64>,heading:Option<f64>) -> Orientation {
	Orientation {source,timestamp,pitch,roll,heading}
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Course {
    source: String,
    timestamp: OffsetDateTime,
    speed: Option<f64>,
    heading: Option<f64>
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SonarDataRecord<T> {
    Ping(Ping<T>),
    Position(Position),
    Orientation(Orientation),
    Course(Course),
    Unknown
}
