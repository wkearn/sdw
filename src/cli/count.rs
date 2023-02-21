//! Count SonarDataRecords in a given file
use crate::parser::jsf;
use crate::records::SonarDataRecord;
use binrw::io::BufReader;
use std::io::{stdout, Write};

fn count_records<T>(file: jsf::JSFFile<T>) -> std::collections::HashMap<String, i64>
where
    T: std::io::Seek + std::io::Read,
{
    let mut msg_counts = std::collections::HashMap::new();

    file.fold(&mut msg_counts, |counts, msg| {
        let rec = SonarDataRecord::from(msg.unwrap());
        let mt = match rec {
            SonarDataRecord::Ping(_) => "Ping".to_string(),
            SonarDataRecord::Position(_) => "Position".to_string(),
            SonarDataRecord::Orientation(_) => "Orientation".to_string(),
            SonarDataRecord::Course(_) => "Course".to_string(),
            SonarDataRecord::Unknown => "Unknown".to_string(),
        };
        let num = counts.entry(mt).or_insert(0);
        *num += 1;
        counts
    });

    msg_counts
}

pub fn count(path: std::path::PathBuf, output: Option<std::path::PathBuf>) -> std::io::Result<()> {
    let f = std::fs::File::open(path)?;
    let reader = BufReader::new(f);
    let jsf = jsf::JSFFile { reader };
    let counts = count_records(jsf);

    match output {
        Some(path) => {
            let mut writer = std::fs::File::create(path)?;
            for (key, value) in &counts {
                writeln!(writer, "{}\t{}", value, key)?;
            }
        }
        None => {
            let mut writer = stdout().lock();
            for (key, value) in &counts {
                writeln!(writer, "{}\t{}", value, key)?;
            }
        }
    };
    Ok(())
}
