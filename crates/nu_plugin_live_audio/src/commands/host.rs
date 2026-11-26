use crate::AudioPlugin;
use mush_core::transport::TransportCommand;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, SyntaxShape, Type, Value};

pub struct Host;

impl SimplePluginCommand for Host {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str {
        "audio host"
    }
    fn description(&self) -> &str {
        "Get or set the active audio host"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .optional(
                "name",
                SyntaxShape::String,
                "Host name to select; omit to query the current host",
            )
            .input_output_type(Type::Nothing, Type::String)
    }

    fn run(
        &self,
        plugin: &AudioPlugin,
        _engine: &nu_plugin::EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let _span = tracing::debug_span!("command", cmd = "host").entered();
        tracing::debug!("host: run");
        let name: Option<String> = call.opt(0)?;
        let mut client = plugin.client.lock().expect("client lock poisoned");
        let mut handle = plugin.handle.lock().expect("handle lock poisoned");
        plugin
            .rt
            .block_on(client.send_command(&TransportCommand::AudioHost(name)))
            .map_err(|e| LabeledError::new(e.to_string()))?;
        let update = plugin.rt.block_on(handle.recv());
        let s = update
            .and_then(|u| mush_core::format::format_update(&u))
            .unwrap_or_else(|| "no response".to_string());
        tracing::debug!(result = %s, "host: done");
        Ok(Value::string(s, Span::unknown()))
    }
}
