use expectrl::{Expect, Regex, spawn};
use std::time::Duration;

fn repl_bin() -> String {
    std::env::var("CARGO_BIN_EXE_repl").unwrap_or_else(|_| "../target/debug/repl".to_string())
}

#[test]
fn test_hello_world() {
    let mut s = spawn(repl_bin()).expect("failed to spawn");
    s.set_expect_timeout(Some(Duration::from_secs(10)));

    s.expect("mash> ").unwrap();
    s.send_line("hello world").unwrap();
    s.expect("You said: hello world").unwrap();

    s.expect("mash> ").unwrap();
    s.send_line("quit").unwrap();
    s.expect("Goodbye!").unwrap();
    s.expect(expectrl::Eof).unwrap();
}

#[test]
fn test_version() {
    let mut s = spawn(repl_bin()).expect("failed to spawn");
    s.set_expect_timeout(Some(Duration::from_secs(10)));

    s.expect("mash> ").unwrap();
    s.send_line("version").unwrap();
    s.expect(Regex(r"v\d+\.\d+\.\d+")).unwrap();

    s.expect("mash> ").unwrap();
    s.send_line("quit").unwrap();
    s.expect(expectrl::Eof).unwrap();
}

#[test]
fn test_blank_line_ignored() {
    let mut s = spawn(repl_bin()).expect("failed to spawn");
    s.set_expect_timeout(Some(Duration::from_secs(10)));

    s.expect("mash> ").unwrap();
    s.send_line("").unwrap();
    s.expect("mash> ").unwrap(); // confirm re-prompt, no output

    s.send_line("quit").unwrap();
    s.expect(expectrl::Eof).unwrap();
}
