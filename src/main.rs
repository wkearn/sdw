use binrw::io::BufReader;
use clap::Parser;

use sdw::cli::Args;
use sdw::parser::jsf;

fn main() -> Result<(), binrw::Error> {
    
    let args = Args::parse();

    let f = std::fs::File::open(args.path)?;
    let reader = BufReader::new(f);
    let jsf = jsf::JSFFile { reader };

    Ok(())
}
