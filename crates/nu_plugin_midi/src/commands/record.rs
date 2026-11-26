use crate::MidiPlugin;
use mush_core::transport::TransportCommand;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, SyntaxShape, Type, Value};
use std::path::PathBuf;

pub struct Record;

impl SimplePluginCommand for Record {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str {
        "midi record"
    }
    fn description(&self) -> &str {
        "Start MIDI recording"
    }

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
        plugin: &MidiPlugin,
        engine: &nu_plugin::EngineInterface,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let _span = tracing::debug_span!("command", cmd = "record").entered();
        tracing::debug!("record: run");

        let path: Option<String> = call.opt(0)?;
        let path = path
            .map(|s| -> Result<PathBuf, LabeledError> {
                let p = PathBuf::from(&s);
                if p.is_absolute() {
                    Ok(p)
                } else {
                    let cwd = engine
                        .get_current_dir()
                        .map_err(|e| LabeledError::new(e.to_string()))?;
                    Ok(PathBuf::from(cwd).join(p))
                }
            })
            .transpose()?;

        let mut client = plugin.client.lock().expect("client lock poisoned");
        let mut handle = plugin.handle.lock().expect("handle lock poisoned");
        plugin
            .rt
            .block_on(client.send_command(&TransportCommand::Record { start: true, path }))
            .map_err(|e| LabeledError::new(e.to_string()))?;

        let update = plugin.rt.block_on(handle.recv());
        let s = update
            .and_then(|u| mush_core::format::format_update(&u))
            .unwrap_or_else(|| "ok".to_string());

        tracing::debug!(result = %s, "record: done");
        Ok(Value::string(s, Span::unknown()))
    }
}
