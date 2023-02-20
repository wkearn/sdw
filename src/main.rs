use binrw::io::BufReader;

use sdw::parser::jsf;

fn main() -> Result<(), binrw::Error> {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];

    let f = std::fs::File::open(path)?;
    let mut reader = BufReader::new(f);
    let jsf = jsf::JSFFile {
        reader: &mut reader,
    };

    let mut msg_counts = std::collections::HashMap::new();

    jsf.fold(&mut msg_counts, |counts, msg| {
        let num = counts.entry(jsf::message_type(&msg.unwrap())).or_insert(0);
        *num += 1;
        counts
    });

    println!("{:?}", msg_counts);

    Ok(())
}
