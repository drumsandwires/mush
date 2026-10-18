mod commands;

use mish_midi_core::midi::engine::MidiEngine;
use nu_plugin::{Plugin, PluginCommand};

#[derive(Default)]
pub struct MidiPlugin {
    pub engine: MidiEngine,
}

impl Plugin for MidiPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(commands::Devices),
            Box::new(commands::DevicesRefresh),
            Box::new(commands::Play),
            Box::new(commands::Stop),
            Box::new(commands::Record),
        ]
    }
}
