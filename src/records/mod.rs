use crate::channel::Channel;
use crate::parser::jsf;
use time::OffsetDateTime;
use serde::{Deserialize,Serialize};

#[derive(Debug,Deserialize,Serialize)]
pub enum SonarDataRecord<T> {
    Ping {
	source: String,
	timestamp: OffsetDateTime,
	frequency: f64,
	sampling_interval: f64,
	channel: Channel,
	data: Vec<T>	
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
	heading: Option<f64>
    },
    Course {
	source: String,
	timestamp: OffsetDateTime,
	speed: Option<f64>,
	heading: Option<f64>,	
    },
    Unknown
}

impl From<jsf::Message> for SonarDataRecord<u16> {
    fn from(msg: jsf::Message) -> Self {
	let md = jsf::message_data(&msg);
	match md {
	    jsf::MessageType::M80 {msg: mt} => SonarDataRecord::Ping {
		source: "unknown".to_string(),
		timestamp: mt.timestamp(),
		frequency: mt.mixer_frequency(),
		sampling_interval: mt.sampling_interval(),
		channel: jsf::channel(&msg),	
		data: mt.trace().to_vec()
	    },
	    jsf::MessageType::M2020 {msg: mt} => SonarDataRecord::Orientation {
		source: "unknown".to_string(),
		timestamp: mt.timestamp(),
		pitch: mt.pitch(),
		roll: mt.roll(),
		heading: mt.heading()
	    },
	    _ => SonarDataRecord::Unknown	    
	}
    }
}
