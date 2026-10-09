use crate::AudioPlugin;
use mash_audio_core::audio::engine::AudioResponse;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, Type, Value};

pub struct Stop;

impl SimplePluginCommand for Stop {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str {
        "mash stop"
    }

    fn description(&self) -> &str {
        "Stop audio playback or recording"
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
        Ok(match plugin.engine.stop() {
            AudioResponse::Output(s) => Value::string(s, Span::unknown()),
            _ => Value::nothing(Span::unknown()),
        })
    }
}
