use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, Type, Value};

use crate::MidiPlugin;

pub struct Record;

impl SimplePluginCommand for Record {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str { "midi record" }
    fn description(&self) -> &str { "Start MIDI recording" }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_type(Type::Nothing, Type::String)
    }

    fn run(
        &self,
        _plugin: &MidiPlugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        Ok(Value::string("record: not yet implemented", Span::unknown()))
    }
}
