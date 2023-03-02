use binrw::io::BufReader;
use sdw::parser::jsf;

#[test]
fn read_xtf() -> Result<(), Box<dyn std::error::Error>> {
    let reader = BufReader::new(std::fs::File::open("assets/HE501_Hydro3_025.001.jsf")?);

    let f = jsf::File::new(reader);

    assert_eq!(905, f.count());

    Ok(())
}
