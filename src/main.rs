use binrw::io::BufReader;
use clap::Parser;

use sdw::cli::{Args,Action};
use sdw::parser::jsf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let args = Args::parse();

    let f = std::fs::File::open(args.path)?;
    let reader = BufReader::new(f);
    let jsf = jsf::JSFFile { reader };

    match args.cmd {
	Action::Count => {
	    jsf::count_jsf_messages(jsf);
	}
    }

    Ok(())
}
