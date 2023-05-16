use nmea0183::Sentence;
use std::path::PathBuf;

fn load_test_data() -> Result<String, std::io::Error> {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/test.txt");
    std::fs::read_to_string(p)
}

#[test]
fn count_types() {
    let data = load_test_data().expect("Failed to open test data file");

    let mut dict = std::collections::HashMap::new();

    for line in data.lines() {
        match line.parse().unwrap() {
            Sentence::GLL => {
                dict.entry("GLL")
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
            Sentence::VTG => {
                dict.entry("VTG")
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
            Sentence::RMC => {
                dict.entry("RMC")
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
            Sentence::GGA => {
                dict.entry("RMC")
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
            Sentence::ZDA => {
                dict.entry("RMC")
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
	    _ => {}
        }
    }

    assert_eq!(Some(&2077), dict.get("GLL"));
    assert_eq!(Some(&2078), dict.get("VTG"));
    assert_eq!(Some(&2077), dict.get("RMC"));
    assert_eq!(Some(&2078), dict.get("GGA"));
    assert_eq!(Some(&2078), dict.get("ZDA"));
}
