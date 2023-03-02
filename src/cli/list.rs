//! List SonarDataRecords in a given file
use crate::model::SonarDataRecord;
use crate::parser::jsf;
use binrw::io::BufReader;
use std::io::{stdout, Write};

fn write_record<T, W: Write>(mut writer: W, rec: SonarDataRecord<T>) -> std::io::Result<()> {
    let datatype = match rec {
        SonarDataRecord::Ping(_) => "Ping".to_string(),
        SonarDataRecord::Position(_) => "Position".to_string(),
        SonarDataRecord::Orientation(_) => "Orientation".to_string(),
        SonarDataRecord::Course(_) => "Course".to_string(),
        SonarDataRecord::Unknown => "Unknown".to_string(),
    };
    writeln!(writer, "{}", datatype)?;
    Ok(())
}

/// List SonarDataRecords in a file
pub fn list(path: &std::path::PathBuf, output: &Option<std::path::PathBuf>) -> std::io::Result<()> {
    let f = std::fs::File::open(path)?;
    let reader = BufReader::new(f);
    let jsf = jsf::File::new(reader);
    match output {
        Some(path) => {
            let mut writer = std::fs::File::create(path)?;
            for msg in jsf {
                //let mt = jsf::message_type_string(jsf::message_data(&msg.unwrap()));
                let rec = SonarDataRecord::from(msg.unwrap());
                write_record(&mut writer, rec)?;
            }
        }
        None => {
            let mut writer = stdout().lock();
            for msg in jsf {
                let rec = SonarDataRecord::from(msg.unwrap());
                write_record(&mut writer, rec)?;
            }
        }
    };

    Ok(())
}
