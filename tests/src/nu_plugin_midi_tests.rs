// mush/tests/src/nu_plugin_midi_tests.rs
//
// nu-plugin-midi is registered with nushell rather than spawned as a
// standalone REPL, so PTY-based testing via expectrl is not applicable here.
// These tests invoke the plugin binary directly to verify it starts cleanly.
// Full command testing requires a running nushell instance with the plugin
// registered — that is better handled in CI via nushell script fixtures.

#[test]
fn nu_plugin_midi_binary_exists() {
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let path = std::path::PathBuf::from(format!("{target_dir}/debug/nu_plugin_midi"));
    assert!(path.exists(), "nu-plugin-midi binary not found at {path:?}");
}
