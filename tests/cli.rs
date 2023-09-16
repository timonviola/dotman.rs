use assert_cmd::prelude::*;

use std::process::Command;
use assert_fs::prelude::*;

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dotman")?;
    cmd.arg("-f")
        .arg("test/file/doesnt/exist")
        .arg("link");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("could not read file"));
    
    Ok(())
}

