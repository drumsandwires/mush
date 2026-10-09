// crates/nu_plugin_live_audio/src/commands/devices_refresh.rs  (CREATE)
use crate::AudioPlugin;
use mash_audio_core::audio::engine::AudioResponse;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, Type, Value};

pub struct DevicesRefresh;

impl SimplePluginCommand for DevicesRefresh {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str {
        "audio devices-refresh"
    }

    fn description(&self) -> &str {
        "Refresh and list available audio hosts and devices"
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
        Ok(match plugin.engine.devices_refresh() {
            AudioResponse::Output(s) => Value::string(s, Span::unknown()),
            _ => Value::nothing(Span::unknown()),
        })
    }
}
