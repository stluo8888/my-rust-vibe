use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn skip_license_env_allows_startup() {
    let mut cmd = Command::cargo_bin("rpa-bin").unwrap();
    cmd.env("RPA_SKIP_LICENSE", "1");
    cmd.assert()
        .success()
        .stdout(contains("RPA_SKIP_LICENSE set; skipping license check"));
}
