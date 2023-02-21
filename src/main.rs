use binrw::io::BufReader;
use clap::Parser;
use std::io::{stdout, Write};

use sdw::cli::{Action, Args};
use sdw::parser::jsf;
use sdw::records::SonarDataRecord;

use apache_avro::{Schema, Writer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.cmd {
        Action::Count { path } => {
            let f = std::fs::File::open(path)?;
            let reader = BufReader::new(f);
            let jsf = jsf::JSFFile { reader };
            let counts = jsf::count_jsf_messages(jsf);
            for (key, value) in &counts {
                println!("{}\t{}", value, key);
            }
        }
        Action::List { path } => {
            let f = std::fs::File::open(path)?;
            let reader = BufReader::new(f);
            let jsf = jsf::JSFFile { reader };
            let mut lock = stdout().lock();
            for msg in jsf {
                let mt = jsf::message_type_string(jsf::message_data(&msg.unwrap()));
                writeln!(lock, "{}", mt).unwrap();
            }
        }
        Action::Avro { path, output } => {
            let raw_schema = r#"{"type": "record","namespace": "sdw","name": "ping","fields": [{"name": "source", "type": "string"},{"name": "timestamp", "type": "long"},{"name": "frequency", "type" : "double"},{"name": "sampling_interval", "type" : "double"},{"name": "channel", "type": "enum", "symbols":["Port","Starboard","Other"],"default":"Other"},{"name": "data", "type":"array","items": "int","default":[]}]}"#;
            let ping_schema = Schema::parse_str(raw_schema)?;

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
        }
    };

    Ok(())
}
