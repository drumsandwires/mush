use std::path::PathBuf;
use mush_core::transport::TransportCommand;
use nu_plugin::{EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, SyntaxShape, Type, Value};
use crate::MidiPlugin;

pub struct Monitor;

impl SimplePluginCommand for Monitor {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str { "midi monitor" }
    fn description(&self) -> &str { "Stream engine updates to the terminal" }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .optional(
                "path",
                SyntaxShape::Filepath,
                "Optional autosave path for the pre-roll buffer",
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
        let _span = tracing::debug_span!("command", cmd = "monitor").entered();
        tracing::debug!("monitor: run");

        let path: Option<String> = call.opt(0)?;
        let path = path
            .map(|s| {
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
            .block_on(client.send_command(&TransportCommand::Monitor { path }))
            .map_err(|e| LabeledError::new(e.to_string()))?;

        let update = plugin.rt.block_on(handle.recv());
        let s = update
            .and_then(|u| mush_core::format::format_update(&u))
            .unwrap_or_else(|| "ok".to_string());

        tracing::debug!(result = %s, "monitor: done");
        Ok(Value::string(s, Span::unknown()))
    }
}