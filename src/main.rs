use binrw::io::BufReader;
use clap::Parser;
use std::io::{stdout, Write};

use sdw::cli::{Action, Args};
use sdw::parser::jsf;

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
    }

    Ok(())
}
