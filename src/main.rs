use binrw::io::BufReader;

use sdw::parser::jsf;
use sdw::records::SonarDataRecord;

fn main() -> Result<(), binrw::Error> {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];

    let f = std::fs::File::open(path)?;
    let mut reader = BufReader::new(f);
    let jsf = jsf::JSFFile {
        reader: &mut reader,
    };

    /*
    let mut msg_counts = std::collections::HashMap::new();


    jsf.fold(&mut msg_counts, |counts, msg| {
        let num = counts.entry(jsf::message_type(&msg.unwrap())).or_insert(0);
        *num += 1;
        counts
    });

    println!("{:?}", msg_counts);
     */

    for msg in jsf {
	let rec = SonarDataRecord::from(msg.unwrap());
	match rec {
	    SonarDataRecord::Ping {source: _,
				   timestamp,
				   frequency: _,
				   sampling_interval: _,
				   channel: _,
				   data: _} => {
		println!("{:?}",timestamp);
	    }
		  
	    _ => {}
	}
    }

    Ok(())
}
