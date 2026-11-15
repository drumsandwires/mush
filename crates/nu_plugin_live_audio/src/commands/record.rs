use crate::AudioPlugin;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, SyntaxShape, Type, Value};

pub struct Record;

impl SimplePluginCommand for Record {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str { "audio record" }
    fn description(&self) -> &str { "Start audio recording (not yet implemented)" }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .optional(
                "path",
                SyntaxShape::Filepath,
                "Optional path to save the recording when stopped",
            )
            .input_output_type(Type::Nothing, Type::Nothing)
    }

    fn run(
        &self,
        _plugin: &AudioPlugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        Err(LabeledError::new("audio record is not yet implemented"))
    }
}