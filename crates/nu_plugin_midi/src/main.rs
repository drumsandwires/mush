use mish_midi_core::midi::engine::MidiEngine;
use nu_plugin::{MsgPackSerializer, serve_plugin};
use nu_plugin_midi::MidiPlugin;

fn main() {
    serve_plugin(
        &MidiPlugin {
            engine: MidiEngine::new(),
        },
        MsgPackSerializer,
    );
}
