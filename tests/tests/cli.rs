// #[test]
// fn cli_tests() {
//     trycmd::TestCases::new().case("tests/cmd/*.toml");
// }

#[test]
fn cli_tests() {
    // Print the current working directory so we know where globs resolve from
    // let cwd = std::env::current_dir().unwrap();
    // eprintln!("cwd: {}", cwd.display());
    //
    // // Check if the file exists relative to cwd
    // eprintln!(
    //     "exists: {}",
    //     std::path::Path::new("cmd/hello.toml").exists()
    // )
    //
    trycmd::TestCases::new().case("cmd/*.toml");
}
