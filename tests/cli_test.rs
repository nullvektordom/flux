use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_flux_help() {
    let mut cmd = Command::cargo_bin("flux").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AI-guided Git workflow assistant"));
}

#[test]
fn test_flux_version() {
    let mut cmd = Command::cargo_bin("flux").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_flux_init_requires_interaction() {
    let mut cmd = Command::cargo_bin("flux").unwrap();
    cmd.arg("init");
    // Init command requires interactive input, so it will fail in non-interactive mode
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not a terminal"));
}

#[test]
fn test_flux_profile_list_not_initialized() {
    let mut cmd = Command::cargo_bin("flux").unwrap();
    cmd.args(["profile", "list"]);
    // Without initialization, should show error
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn test_flux_commit_placeholder() {
    let mut cmd = Command::cargo_bin("flux").unwrap();
    cmd.arg("commit");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_flux_status_placeholder() {
    let mut cmd = Command::cargo_bin("flux").unwrap();
    cmd.arg("status");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}
