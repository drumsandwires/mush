use nu_plugin_midi::MidiPlugin;
use nu_plugin_test_support::PluginTest;
use nu_protocol::Value;

#[test]
fn test_midi_devices_returns_string() -> Result<(), Box<dyn std::error::Error>> {
    let plugin = MidiPlugin::default();

    let result = PluginTest::new("nu_plugin_midi", plugin.into())?
        .eval("midi devices")?
        .into_value(nu_protocol::Span::test_data())
        .expect("command should succeed");

    assert!(
        matches!(result, Value::String { .. }),
        "expected a string, got: {result:?}"
    );

    Ok(())
}
