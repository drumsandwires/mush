use nu_plugin::{MsgPackSerializer, serve_plugin};
use nu_plugin_live_audio::AudioPlugin;

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let client = rt.block_on(AudioPlugin::setup(
        mush_core::addrs::CMD_INPROC,
        mush_core::addrs::PUB_INPROC,
    ))?;
    let plugin = AudioPlugin::new(client, rt);
    serve_plugin(&plugin, MsgPackSerializer);
    Ok(())
}