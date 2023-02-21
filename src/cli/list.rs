use crate::parser::jsf;
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
                let mt = jsf::message_type_string(jsf::message_data(&msg.unwrap()));
                writeln!(writer, "{}", mt).unwrap();
            }
        }
        None => {
            let mut writer = stdout().lock();
            for msg in jsf {
                let mt = jsf::message_type_string(jsf::message_data(&msg.unwrap()));
                writeln!(writer, "{}", mt).unwrap();
            }
        }
    };

    Ok(())
}
