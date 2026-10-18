mod commands;

use mish_midi_core::midi::engine::MidiEngine;
use nu_plugin::{MsgPackSerializer, Plugin, PluginCommand, serve_plugin};

fn main() {
    serve_plugin(
        &MidiPlugin {
            engine: MidiEngine::new(),
        },
        MsgPackSerializer,
    );
}
