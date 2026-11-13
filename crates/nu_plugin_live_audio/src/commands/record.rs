use std::path::PathBuf;
use crate::AudioPlugin;
use mush_core::transport::TransportCommand;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, SyntaxShape, Type, Value};

pub struct Record;

impl SimplePluginCommand for Record {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str { "audio record" }
    fn description(&self) -> &str { "Start audio recording" }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .optional(
                "path",
                SyntaxShape::Filepath,
                "Optional path to save the recording when stopped",
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
        let path: Option<String> = call.opt(0)?;
        let path = path.map(PathBuf::from);

        let mut client = plugin.client.lock().expect("client lock poisoned");
        plugin.rt
            .block_on(client.send_command(&TransportCommand::Record { start: true, path }))
            .map_err(|e| LabeledError::new(e.to_string()))?;

        let update = plugin.rt.block_on(client.recv_update());
        let s = update
            .and_then(|u| mush_core::format::format_update(&u))
            .unwrap_or_else(|| "ok".to_string());

        Ok(Value::string(s, Span::unknown()))
    }
}