use crate::MidiPlugin;
use mish_midi_core::midi::engine::MidiResponse;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, Type, Value};

pub struct Stop;

impl SimplePluginCommand for Stop {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str {
        "midi stop"
    }

    fn description(&self) -> &str {
        "Stop MIDI playback or recording"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name()).input_output_type(Type::Nothing, Type::String)
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        Ok(match plugin.engine.stop() {
            MidiResponse::Output(s) => Value::string(s, Span::unknown()),
            _ => Value::nothing(Span::unknown()),
        })
    }
}
