use assert_cmd::prelude::*;
use predicates::prelude::*;

use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dotman")?;
    cmd.arg("-f").arg("test/file/doesnt/exist").arg("link");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("could not read file"));

    Ok(())
}

#[test]
fn tag_single_matches_single() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dotman")?;
    cmd.arg("-f")
        .arg("tests/dotman_conf.toml")
        .arg("-t")
        .arg("group1")
        .arg("show");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("cake"))
        .stdout(predicate::str::contains("maccaroon"));

    Ok(())
}

#[test]
fn tag_single_matches_multi_tag_item() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dotman")?;
    cmd.arg("-f")
        .arg("tests/dotman_conf.toml")
        .arg("-t")
        .arg("red")
        .arg("show");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("maccaroon"))
        .stdout(predicate::str::is_match("cake").unwrap().not());

    Ok(())
}

#[test]
fn tag_comma_requires_all_tags() -> Result<(), Box<dyn std::error::Error>> {
    // --tag group1,red should only match items with BOTH tags
    let mut cmd = Command::cargo_bin("dotman")?;
    cmd.arg("-f")
        .arg("tests/dotman_conf.toml")
        .arg("-t")
        .arg("group1,red")
        .arg("show");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("maccaroon"))
        .stdout(predicate::str::is_match("cake").unwrap().not())
        .stdout(predicate::str::is_match("biscuit").unwrap().not());

    Ok(())
}

#[test]
fn tag_multiple_flags_or_logic() -> Result<(), Box<dyn std::error::Error>> {
    // --tag group1 --tag blue should match items with EITHER tag
    let mut cmd = Command::cargo_bin("dotman")?;
    cmd.arg("-f")
        .arg("tests/dotman_conf.toml")
        .arg("-t")
        .arg("group1")
        .arg("-t")
        .arg("blue")
        .arg("show");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("cake"))
        .stdout(predicate::str::contains("maccaroon"))
        .stdout(predicate::str::contains("biscuit"));

    Ok(())
}

#[test]
fn tag_no_match_exits_with_warning() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dotman")?;
    cmd.arg("-f")
        .arg("tests/dotman_conf.toml")
        .arg("-t")
        .arg("nonexistent")
        .arg("show");
    cmd.assert().success();

    Ok(())
}
