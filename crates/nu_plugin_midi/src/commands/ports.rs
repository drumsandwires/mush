use crate::MidiPlugin;
use mish_midi_core::midi::engine::MidiResponse;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, Type, Value};

pub struct Ports;

impl SimplePluginCommand for Ports {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str {
        "mish ports"
    }

    fn description(&self) -> &str {
        "List available MIDI ports"
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
        Ok(match plugin.engine.ports() {
            MidiResponse::Output(s) => Value::string(s, Span::unknown()),
            _ => Value::nothing(Span::unknown()),
        })
    }
}
