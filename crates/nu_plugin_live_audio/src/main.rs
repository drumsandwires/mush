mod commands;

use mash_audio_core::audio::engine::AudioEngine;
use nu_plugin::{MsgPackSerializer, Plugin, PluginCommand, serve_plugin};

pub struct AudioPlugin {
    pub engine: AudioEngine,
}

impl Plugin for AudioPlugin {
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

fn main() {
    serve_plugin(
        &AudioPlugin {
            engine: AudioEngine::new(),
        },
        MsgPackSerializer,
    );
}
