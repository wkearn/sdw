use binrw::io::BufReader;
use sdw::parser::imagenex81b;

#[test]
fn read_file() -> Result<(), Box<dyn std::error::Error>> {
    let reader = BufReader::new(std::fs::File::open("assets/5197DB5B.81B")?);

    let f = imagenex81b::File::new(reader);

    assert_eq!(340, f.count());

    Ok(())
}
