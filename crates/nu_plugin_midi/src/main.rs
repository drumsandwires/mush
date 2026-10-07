// mush/crates/nu-plugin-midi/src/main.rs
use mish_midi_core::{MidiEngine, MidiResponse};
use nu_plugin::{
    EngineInterface,
    EvaluatedCall,
    MsgPackSerializer, // MediumInterface removed
    Plugin,
    PluginCommand,
    serve_plugin,
};
use nu_protocol::{Category, LabeledError, PipelineData, Signature, SyntaxShape, Value};

struct MidiPlugin {
    engine: MidiEngine,
}

impl Plugin for MidiPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(MidiCmd),
            Box::new(MidiListPorts),
            Box::new(MidiPlay),
            Box::new(MidiListen),
            Box::new(MidiRecord),
            Box::new(MidiStop),
        ]
    }
}

// --- midi (generic passthrough) ---

struct MidiCmd;

impl PluginCommand for MidiCmd {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str {
        "midi"
    }

    fn description(&self) -> &str {
        "Run a MIDI command"
    }

    fn signature(&self) -> Signature {
        Signature::build("midi")
            .required("command", SyntaxShape::String, "MIDI command to run")
            .category(Category::Custom("midi".to_string()))
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let cmd: String = call.req(0)?;
        to_pipeline(plugin.engine.evaluate(&cmd), call)
    }
}

// --- midi ports ---

struct MidiListPorts;

impl PluginCommand for MidiListPorts {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str {
        "midi ports"
    }

    fn description(&self) -> &str {
        "List available MIDI ports"
    }

    fn signature(&self) -> Signature {
        Signature::build("midi ports").category(Category::Custom("midi".to_string()))
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        to_pipeline(plugin.engine.evaluate("ports"), call)
    }
}

// --- midi play ---

struct MidiPlay;

impl PluginCommand for MidiPlay {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str {
        "midi play"
    }

    fn description(&self) -> &str {
        "Start MIDI playback"
    }

    fn signature(&self) -> Signature {
        Signature::build("midi play").category(Category::Custom("midi".to_string()))
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        to_pipeline(plugin.engine.evaluate("play"), call)
    }
}

// --- midi listen ---

struct MidiListen;

impl PluginCommand for MidiListen {
    type Plugin = MidiPlugin;
    fn name(&self) -> &str {
        "midi listen"
    }

    fn description(&self) -> &str {
        "Start MIDI monitoring"
    }

    fn signature(&self) -> Signature {
        Signature::build("midi listen").category(Category::Custom("midi".to_string()))
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        to_pipeline(plugin.engine.evaluate("listen"), call)
    }
}

// --- midi record ---

struct MidiRecord;

impl PluginCommand for MidiRecord {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str {
        "midi record"
    }

    fn description(&self) -> &str {
        "Start MIDI recording"
    }

    fn signature(&self) -> Signature {
        Signature::build("midi record").category(Category::Custom("midi".to_string()))
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        to_pipeline(plugin.engine.evaluate("record"), call)
    }
}

// --- midi stop ---

struct MidiStop;

impl PluginCommand for MidiStop {
    type Plugin = MidiPlugin;

    fn name(&self) -> &str {
        "midi stop"
    }

    fn description(&self) -> &str {
        "Stop MIDI playback"
    }

    fn signature(&self) -> Signature {
        Signature::build("midi stop").category(Category::Custom("midi".to_string()))
    }

    fn run(
        &self,
        plugin: &MidiPlugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        to_pipeline(plugin.engine.evaluate("stop"), call)
    }
}

// --- shared response mapping ---

fn to_pipeline(response: MidiResponse, call: &EvaluatedCall) -> Result<PipelineData, LabeledError> {
    match response {
        MidiResponse::Output(s) => Ok(PipelineData::Value(Value::string(s, call.head), None)),
        MidiResponse::Empty => Ok(PipelineData::Empty),
        MidiResponse::Exit(s) => Ok(PipelineData::Value(Value::string(s, call.head), None)),
        MidiResponse::Unknown => Err(LabeledError::new("Unknown MIDI command")
            .with_label("not a recognised MIDI command", call.head)),
    }
}

fn main() {
    serve_plugin(
        &MidiPlugin {
            engine: MidiEngine::new(),
        },
        MsgPackSerializer, // was MediumInterface
    );
}
