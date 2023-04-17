//! Print info about a sonar file

use crate::model::{Channel, Course, Orientation, Ping, Position, SonarDataRecord};
use crate::parser::jsf;
use binrw::io::BufReader;
use std::collections::{HashMap, HashSet};
use std::io::{stdout, Write};
use time::OffsetDateTime;

/// Print info about a sonar file
pub fn info<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<()> {
    let f = std::fs::File::open(path.as_ref())?;
    let reader = BufReader::new(f);
    let format = "JSF".to_string();
    let jsf = jsf::File::new(reader);

    let mut port_channel_count = 0;
    let mut starboard_channel_count = 0;
    let mut other_channel_count = 0;
    let mut data_lengths = HashSet::new();
    let mut sampling_intervals = Vec::new();
    let mut start_date = OffsetDateTime::now_utc();
    let mut end_date = OffsetDateTime::UNIX_EPOCH;

    for msg in jsf {
        let rec = SonarDataRecord::from(msg.unwrap());
        match rec {
            SonarDataRecord::Ping(Ping {
                timestamp,
                channel,
                data,
                sampling_interval,
                ..
            }) => {
                match channel {
                    Channel::Port => port_channel_count += 1,
                    Channel::Starboard => starboard_channel_count += 1,
                    Channel::Other => other_channel_count += 1,
                }

                if timestamp < start_date {
                    start_date = timestamp;
                } else if timestamp > end_date {
                    end_date = timestamp;
                }

                data_lengths.insert(data.len());
                sampling_intervals.push(sampling_interval);
            }
            SonarDataRecord::Position(Position {
                timestamp,
                longitude: _,
                latitude: _,
                altitude: _,
                ..
            }) => {
                if timestamp < start_date {
                    start_date = timestamp;
                } else if timestamp > end_date {
                    end_date = timestamp;
                }
            }
            SonarDataRecord::Orientation(Orientation {
                timestamp,
                pitch: _,
                roll: _,
                heading: _,
                ..
            }) => {
                if timestamp < start_date {
                    start_date = timestamp;
                } else if timestamp > end_date {
                    end_date = timestamp;
                }
            }
            SonarDataRecord::Course(Course {
                timestamp,
                speed: _,
                heading: _,
                ..
            }) => {
                if timestamp < start_date {
                    start_date = timestamp;
                } else if timestamp > end_date {
                    end_date = timestamp;
                }
            }
            SonarDataRecord::Unknown => {}
        };
    }

    println!("File: {}", path.as_ref().display());
    println!("Format: {}", format);
    println!("Start date: {}", start_date);
    println!("End date: {}", end_date);
    println!("Number of port channel pings: {}", port_channel_count);
    println!(
        "Number of starboard channel pings: {}",
        starboard_channel_count
    );
    println!("Number of other channel pings: {}", other_channel_count);
    println!("Unique lengths of pings:");
    for length in &data_lengths {
        println!("\t{}", length);
    }

    println!("Unique sampling intervals:");
    sampling_intervals.sort_by(|a, b| a.total_cmp(&b));
    sampling_intervals.dedup();
    for interval in sampling_intervals {
        println!("\t{}", interval);
    }

    Ok(())
}
