// #[test]
// fn cli_tests() {
//     trycmd::TestCases::new().case("tests/cmd/*.toml");
// }

// use assert_cmd::Command;
// use predicates::str::contains;
// use predicates::prelude::*;

#[ignore]
#[test]
fn test_mash_banner() {
    trycmd::TestCases::new().case("cmd/mash-banner.toml");
}

#[ignore]
#[test]
fn test_mish_banner() {
    trycmd::TestCases::new().case("cmd/mish-banner.toml");
}

#[ignore]
#[test]
fn test_mush_banner() {
    trycmd::TestCases::new().case("cmd/mush-banner.toml");
}

#[ignore]
#[test]
fn test_mash_cmd_devices() {
    assert_cmd::Command::new("../target/debug/mash")
        .write_stdin("devices\nquit\n")
        .assert()
        .success()
        .stderr(predicates::str::contains("ALSA lib"))
        .stdout(predicates::str::contains("Host:"))
        .stdout(predicates::str::contains("Bye!"));
}
