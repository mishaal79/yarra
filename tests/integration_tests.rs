use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_block_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());

    let mut cmd = Command::cargo_bin("yarra")?;
    cmd.arg("block").arg("--site").arg("youtube.com");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Blocked youtube.com"));

    let config = std::fs::read_to_string(temp_dir.path().join("yarra/config.toml"))?;
    assert!(config.contains("youtube.com"));

    Ok(())
}

#[test]
fn test_stats_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    std::env::set_var("XDG_DATA_HOME", temp_dir.path());

    let mut cmd = Command::cargo_bin("yarra")?;
    cmd.arg("stats");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Total focus time: 0 minutes"));

    Ok(())
}