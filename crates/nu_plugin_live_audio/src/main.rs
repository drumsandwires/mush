// mush/crates/nu-plugin-audio/src/main.rs
use mash_audio_core::{AudioEngine, AudioResponse};
use nu_plugin::{
    EngineInterface,
    EvaluatedCall,
    MsgPackSerializer, // MediumInterface removed
    Plugin,
    PluginCommand,
    serve_plugin,
};
use nu_protocol::{Category, LabeledError, PipelineData, Signature, SyntaxShape, Value};

struct AudioPlugin {
    engine: AudioEngine,
}

impl Plugin for AudioPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(AudioCmd),
            Box::new(AudioListPorts),
            Box::new(AudioPlay),
            Box::new(AudioRecord),
            Box::new(AudioStop),
        ]
    }
}

// --- audio (generic passthrough) ---

struct AudioCmd;

impl PluginCommand for AudioCmd {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str {
        "audio"
    }

    fn description(&self) -> &str {
        "Run an Audio command"
    }

    fn signature(&self) -> Signature {
        Signature::build("audio")
            .required("command", SyntaxShape::String, "Audio command to run")
            .category(Category::Custom("audio".to_string()))
    }

    fn run(
        &self,
        plugin: &AudioPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let cmd: String = call.req(0)?;
        to_pipeline(plugin.engine.evaluate(&cmd), call)
    }
}

// --- audio ports ---

struct AudioListPorts;

impl PluginCommand for AudioListPorts {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str {
        "audio ports"
    }

    fn description(&self) -> &str {
        "List available Audio ports"
    }

    fn signature(&self) -> Signature {
        Signature::build("audio ports").category(Category::Custom("audio".to_string()))
    }

    fn run(
        &self,
        plugin: &AudioPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        to_pipeline(plugin.engine.evaluate("ports"), call)
    }
}

// --- audio play ---

struct AudioPlay;

impl PluginCommand for AudioPlay {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str {
        "audio play"
    }

    fn description(&self) -> &str {
        "Start Audio playback"
    }

    fn signature(&self) -> Signature {
        Signature::build("audio play").category(Category::Custom("audio".to_string()))
    }

    fn run(
        &self,
        plugin: &AudioPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        to_pipeline(plugin.engine.evaluate("play"), call)
    }
}

// --- audio record ---

struct AudioRecord;

impl PluginCommand for AudioRecord {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str {
        "audio record"
    }

    fn description(&self) -> &str {
        "Start Audio recording"
    }

    fn signature(&self) -> Signature {
        Signature::build("audio record").category(Category::Custom("audio".to_string()))
    }

    fn run(
        &self,
        plugin: &AudioPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        to_pipeline(plugin.engine.evaluate("record"), call)
    }
}

// --- audio stop ---

struct AudioStop;

impl PluginCommand for AudioStop {
    type Plugin = AudioPlugin;

    fn name(&self) -> &str {
        "audio stop"
    }

    fn description(&self) -> &str {
        "Stop Audio playback"
    }

    fn signature(&self) -> Signature {
        Signature::build("audio stop").category(Category::Custom("audio".to_string()))
    }

    fn run(
        &self,
        plugin: &AudioPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        to_pipeline(plugin.engine.evaluate("stop"), call)
    }
}

// --- shared response mapping ---

fn to_pipeline(
    response: AudioResponse,
    call: &EvaluatedCall,
) -> Result<PipelineData, LabeledError> {
    match response {
        AudioResponse::Output(s) => Ok(PipelineData::Value(Value::string(s, call.head), None)),
        AudioResponse::Empty => Ok(PipelineData::Empty),
        AudioResponse::Exit(s) => Ok(PipelineData::Value(Value::string(s, call.head), None)),
        AudioResponse::Unknown => Err(LabeledError::new("Unknown Audio command")
            .with_label("not a recognised Audio command", call.head)),
    }
}

fn main() {
    serve_plugin(
        &AudioPlugin {
            engine: AudioEngine::new(),
        },
        MsgPackSerializer, // was MediumInterface
    );
}
