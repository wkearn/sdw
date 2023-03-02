use binrw::io::BufReader;
use sdw::parser::xtf;

#[test]
fn read_file() -> Result<(), Box<dyn std::error::Error>> {
    let reader = BufReader::new(std::fs::File::open("assets/15CCT03_SSS_150528172600.xtf")?);

    let f = xtf::File::new(reader);

    assert_eq!(26802, f.count());

    Ok(())
}
