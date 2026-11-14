mod commands;

use mush_core::subscription::SubscriptionHandle;
use mush_core::zmq_client::ZmqClient;
use nu_plugin::{Plugin, PluginCommand};
use std::sync::Mutex;
use tokio::runtime::Handle;

pub struct MidiPlugin {
    pub(crate) client: Mutex<ZmqClient>,
    pub(crate) handle: Mutex<SubscriptionHandle>,
    pub(crate) rt: Handle,
}

impl MidiPlugin {
    pub fn new(client: ZmqClient, handle: SubscriptionHandle, rt: Handle) -> Self {
        Self {
            client: Mutex::new(client),
            handle: Mutex::new(handle),
            rt,
        }
    }
}

impl Plugin for MidiPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(commands::Load),
            Box::new(commands::Save),
            Box::new(commands::Ports),
            Box::new(commands::Port),
            Box::new(commands::Send),
            Box::new(commands::Play),
            Box::new(commands::Stop),
            Box::new(commands::Record),
            Box::new(commands::Monitor),
        ]
    }
}