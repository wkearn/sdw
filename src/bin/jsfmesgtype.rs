use binrw::io::BufReader;
use clap::Parser;

use sdw::cli::Args;
use sdw::parser::jsf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    let f = std::fs::File::open(args.path)?;
    let reader = BufReader::new(f);
    let jsf = jsf::JSFFile { reader };

    let mut msg_counts = std::collections::HashMap::new();

    jsf.fold(&mut msg_counts, |counts, msg| {
        let num = counts.entry(jsf::message_type(&msg.unwrap())).or_insert(0);
        *num += 1;
        counts
    });

    println!("{:?}", msg_counts);

    Ok(())
}
