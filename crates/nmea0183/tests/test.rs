use nmea0183::Sentence;
use std::path::PathBuf;

fn load_test_data() -> Result<String,std::io::Error> {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/test.txt");
    std::fs::read_to_string(p)
}

#[test]
fn parse_lines() {
    let data = load_test_data().expect("Failed to open test data file");

    for line in data.lines() {
	let nms: Sentence = line.parse().unwrap();
    }
}
