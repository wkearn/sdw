use binrw::io::BufReader;
use clap::Parser;

use sdw::cli::{Args,Action};
use sdw::parser::jsf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let args = Args::parse();

    match args.cmd {
	Action::Count {path} => {
	    let f = std::fs::File::open(path)?;
	    let reader = BufReader::new(f);
	    let jsf = jsf::JSFFile { reader };
	    jsf::count_jsf_messages(jsf);
	}
    }

    Ok(())
}
