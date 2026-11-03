use nu_plugin_midi::MidiPlugin;
use nu_plugin_test_support::PluginTest;
use nu_protocol::Value;

#[ignore = "requires live Conductor — run manually on a dev machine"]
#[test]
fn test_nu_midi_devices_returns_string() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;

    let client = rt.block_on(MidiPlugin::setup(
        mush_core::addrs::CMD_INPROC,
        mush_core::addrs::PUB_INPROC,
    ))?;

    let plugin = MidiPlugin::new(client, rt);

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
