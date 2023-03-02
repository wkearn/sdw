use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn count_records() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("sdw")?;

    cmd.arg("count").arg("assets/HE501_Hydro3_025.001.jsf");
    cmd.assert().success().stdout(
        predicate::str::contains("640\tPing")
            .and(predicate::str::contains("168\tOrientation"))
            .and(predicate::str::contains("97\tUnknown")),
    );

    Ok(())
}

#[test]
fn count_records_no_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("sdw")?;

    cmd.arg("count").arg("assets/HE501_Hydro3_025.002.jsf");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));

    Ok(())
}
