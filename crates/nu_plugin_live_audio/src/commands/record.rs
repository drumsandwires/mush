use crate::AudioPlugin;
use mash_audio_core::audio::engine::AudioResponse;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, Type, Value};

pub struct Record;

impl SimplePluginCommand for Record {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str {
        "audio record"
    }

    fn description(&self) -> &str {
        "Start audio recording"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name()).input_output_type(Type::Nothing, Type::String)
    }

    fn run(
        &self,
        plugin: &AudioPlugin,
        _engine: &nu_plugin::EngineInterface,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        Ok(match plugin.engine.record() {
            AudioResponse::Output(s) => Value::string(s, Span::unknown()),
            _ => Value::nothing(Span::unknown()),
        })
    }
}
