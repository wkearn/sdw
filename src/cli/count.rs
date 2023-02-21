use crate::parser::jsf;
use binrw::io::BufReader;
use std::io::{stdout, Write};

pub fn count(path: std::path::PathBuf, output: Option<std::path::PathBuf>) -> std::io::Result<()> {
    let f = std::fs::File::open(path)?;
    let reader = BufReader::new(f);
    let jsf = jsf::JSFFile { reader };
    let counts = jsf::count_jsf_messages(jsf);

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
