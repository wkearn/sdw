//! List SonarDataRecords in a given file
use crate::parser::jsf;
use crate::records::SonarDataRecord;
use binrw::io::BufReader;
use std::io::{stdout, Write};

pub fn list(path: std::path::PathBuf, output: Option<std::path::PathBuf>) -> std::io::Result<()> {
    let f = std::fs::File::open(path)?;
    let reader = BufReader::new(f);
    let jsf = jsf::JSFFile { reader };
    match output {
        Some(path) => {
            let mut writer = std::fs::File::create(path)?;
            for msg in jsf {
                //let mt = jsf::message_type_string(jsf::message_data(&msg.unwrap()));
                let rec = SonarDataRecord::from(msg.unwrap());
                match rec {
                    SonarDataRecord::Ping(_) => writeln!(writer, "Ping").unwrap(),
                    SonarDataRecord::Position(_) => writeln!(writer, "Position").unwrap(),
                    SonarDataRecord::Orientation(_) => writeln!(writer, "Orientation").unwrap(),
                    SonarDataRecord::Course(_) => writeln!(writer, "Course").unwrap(),
                    SonarDataRecord::Unknown => writeln!(writer, "Unknown").unwrap(),
                };
            }
        }
        None => {
            let mut writer = stdout().lock();
            for msg in jsf {
                let rec = SonarDataRecord::from(msg.unwrap());
                match rec {
                    SonarDataRecord::Ping(_) => writeln!(writer, "Ping").unwrap(),
                    SonarDataRecord::Position(_) => writeln!(writer, "Position").unwrap(),
                    SonarDataRecord::Orientation(_) => writeln!(writer, "Orientation").unwrap(),
                    SonarDataRecord::Course(_) => writeln!(writer, "Course").unwrap(),
                    SonarDataRecord::Unknown => writeln!(writer, "Unknown").unwrap(),
                };
            }
        }
    };

    Ok(())
}
