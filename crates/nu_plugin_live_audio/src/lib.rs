mod commands;

use std::sync::Mutex;
use mush_core::conductor::Conductor;
use mush_core::zmq_client::ZmqClient;
use nu_plugin::{Plugin, PluginCommand};
use tokio::runtime::Runtime;

pub struct AudioPlugin {
    pub(crate) client: Mutex<ZmqClient>,
    pub(crate) rt:     Runtime,
}

impl AudioPlugin {
    pub fn new(client: ZmqClient, rt: Runtime) -> Self {
        Self { client: Mutex::new(client), rt }
    }

    pub async fn setup(cmd_addr: &str, pub_addr: &str) -> anyhow::Result<ZmqClient> {
        let (conductor, _midi_queue, _audio_tx, _midi_tx) =
            Conductor::new(cmd_addr, pub_addr, 44_100)?;
        let ctx = conductor.context().clone();
        tokio::spawn(async move { conductor.run().await; });
        tokio::task::yield_now().await;
        Ok(ZmqClient::connect(&ctx, cmd_addr, pub_addr)?)
    }
}

impl Plugin for AudioPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(commands::Devices),
            Box::new(commands::DevicesRefresh),
            Box::new(commands::Hosts),
            Box::new(commands::Host),
            Box::new(commands::Device),
            Box::new(commands::Play),
            Box::new(commands::Stop),
            Box::new(commands::Record),
        ]
    }
}