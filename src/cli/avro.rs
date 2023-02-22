//! Converting sonar data files to Avro files
use crate::parser::jsf;
use crate::records::SonarDataRecord;

use apache_avro::{Schema, Writer};

use binrw::io::BufReader;

/// Convert a JSF file to the Avro format
///
/// Currently this writes only the Pings to a single Avro file.
/// This is meant to be used from the command line interface:
/// ```console
/// $ sdw avro <input> <output>
/// ```
pub fn avro(path: &std::path::PathBuf, output: &std::path::PathBuf) -> std::io::Result<()> {
    let raw_schema = r#"{"type": "record","namespace": "sdw","name": "ping","fields": [{"name": "source", "type": "string"},{"name": "timestamp", "type": "long"},{"name": "frequency", "type" : "double"},{"name": "sampling_interval", "type" : "double"},{"name": "channel", "type": "enum", "symbols":["Port","Starboard","Other"],"default":"Other"},{"name": "data", "type":"array","items": "int","default":[]}]}"#;
    let ping_schema = Schema::parse_str(raw_schema).unwrap();

    let f = std::fs::File::open(path)?;
    let reader = BufReader::new(f);
    let jsf = jsf::JSFFile { reader };
    let sds = jsf.filter_map(|msg| {
        if let SonarDataRecord::Ping(ping) = SonarDataRecord::from(msg.unwrap()) {
            Some(ping)
        } else {
            None
        }
    });

    let g = std::fs::File::create(output)?;
    let mut writer = Writer::new(&ping_schema, g);

    writer.extend_ser(sds).unwrap();
    Ok(())
}
