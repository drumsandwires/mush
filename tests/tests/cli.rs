// #[test]
// fn cli_tests() {
//     trycmd::TestCases::new().case("tests/cmd/*.toml");
// }

// use assert_cmd::Command;
// use predicates::str::contains;
// use predicates::prelude::*;

#[test]
fn test_banner() {
    trycmd::TestCases::new().case("cmd/mash-banner.toml");
}

#[test]
fn test_cmd_devices() {
    assert_cmd::Command::new("../target/debug/mash")
        .write_stdin("devices\nquit\n")
        .assert()
        .success()
        .stderr(predicates::str::contains("ALSA lib"))
        .stdout(predicates::str::contains("Host:"))
        .stdout(predicates::str::contains("Bye!"));
}
