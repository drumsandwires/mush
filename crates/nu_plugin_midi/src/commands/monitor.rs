use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, Type, Value};

use crate::MidiPlugin;

pub struct Monitor;

impl SimplePluginCommand for Monitor {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str { "midi monitor" }
    fn description(&self) -> &str { "Stream engine updates to the terminal" }

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
        Ok(Value::string("monitor: not yet implemented", Span::unknown()))
    }
}
