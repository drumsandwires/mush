use nu_plugin::{MsgPackSerializer, serve_plugin};
use nu_plugin_live_audio::AudioPlugin;
use mush_core::conductor::Conductor;
use mush_core::zmq_client::ZmqClient;

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let rt_handle = rt.handle().clone();

    let (conductor, midi_queue, audio_tx, midi_tx, ctx, sub_handle) =
        rt_handle.block_on(async {
            let (mut conductor, midi_queue, audio_tx, midi_tx) = Conductor::new(
                mush_core::addrs::CMD_INPROC,
                mush_core::addrs::PUB_INPROC,
                44_100,
            )?;
            let sub_handle = conductor.subscribe();
            let ctx = conductor.context().clone();
            Ok::<_, anyhow::Error>((conductor, midi_queue, audio_tx, midi_tx, ctx, sub_handle))
        })?;

    let client = rt_handle.block_on(async {
        Ok::<_, anyhow::Error>(ZmqClient::connect(
            &ctx,
            mush_core::addrs::CMD_INPROC,
            mush_core::addrs::PUB_INPROC,
        )?)
    })?;

    rt_handle.spawn(async move { conductor.run().await });
    let _rt_guard = std::thread::spawn(move || {
        rt.block_on(std::future::pending::<()>());
    });

    let _keep = (midi_queue, audio_tx, midi_tx);

    let plugin = AudioPlugin::new(client, sub_handle, rt_handle);
    serve_plugin(&plugin, MsgPackSerializer);
    Ok(())
}