# Project Mid-Point Review

Here are the revised MVP requirements at mid-point project review. 

## Architecture

```plantuml
@startuml mush_architecture

skinparam defaultFontName "JetBrains Mono"
skinparam defaultFontSize 12
top to bottom direction

package "mish-shell binary (MVP — same pattern for mash · mush · nu_plugin_midi)" {

    package "Control Layer" {
        component [Shell REPL\nReedline · or Nushell plugin interface] as SHELL
    }

    package "Transport Layer — ZMQ inproc" {
        component [ZmqClient\nDEALER · SUB\ninproc://mush-cmd · mush-pub] as ZMQC
        component [ZmqTransport\nROUTER · PUB\ninproc://mush-cmd · mush-pub] as ZMQS
    }

    package "Conductor Layer — Tokio Runtime" {
        component [Conductor\ntokio::select! loop] as TC
        component [Subscription · SubscriptionHandle\nmpsc — in-process] as SUB
    }

    package "Transport State — Arc<TransportState>" {
        component [playhead · is_playing\nbpm · timeline · next_event_idx] as TE
    }

    package "RT Layer" {
        component [Clock\ncpal callback · software fallback] as CLK
        component [MidiEngine\nhi-pri thread] as ME
        component [MidiEngineRt\nSCHED_FIFO] as MERT
        component [crossbeam::ArrayQueue\nMidiEvent] as LFQ
    }

    package "Hardware" {
        component [Audio Interface\ncpal] as AI
        component [MIDI Interface\nmidir] as MI
    }

    ' Command path — downward
    SHELL -d-> ZMQC : TransportCommand
    ZMQC  -d-> ZMQS : DEALER→ROUTER inproc

    ' Conductor receives and dispatches
    ZMQS -d-> TC  : TransportCommand
    TC   -d-> TE  : write atomics
    TC   -r-> ME  : LoadMidi · QueueEvent\nLoadMudl (reserved)
    TC   -l-> CLK : Play · Stop · Seek · SetBpm

    ' RT execution path
    TE   -d-> CLK  : read atomics
    CLK  -d-> LFQ  : push MidiEvent
    LFQ  -r-> MERT : pop MidiEvent
    MERT -u-> TE   : read playhead
    CLK  -d-> AI   : audio I/O
    MERT -d-> MI   : send MIDI bytes

    ' Status path — upward to conductor
    CLK -u-> TC : AudioUpdate · tokio::sync::mpsc
    ME  -u-> TC : MidiUpdate  · tokio::sync::mpsc

    ' Conductor fan-out — upward to shell
    TC   -u-> ZMQS : EngineUpdate
    ZMQS -u-> ZMQC : EngineUpdate · SUB inproc
    ZMQC -u-> SHELL : EngineUpdate
    TC   -r-> SUB  : EngineUpdate · in-process
    SUB  -u-> SHELL : EngineUpdate · SubscriptionHandle
}

note bottom
  Post-MVP Model A: extract TheConductor into mush-conductor binary.
  Change ZmqClient addresses from inproc:// to tcp://5252/5253.
  No other code changes required.
end note

@enduml
```

## Midi Commands

Available in mish, nu_midi_pligin, mush

ports - show available ports
port - no arg show active live port, with arg set active live port
send - midi event or segment to active live port
load - from file
    .mid 
    .mudl
play - play loaded file or recorded events
stop - stop playback or record - map tp space bar
monitor - stream received events to ui, buffer in memory for pre-roll record
record - buffer input events in memory, backup to temp file
save - to file 
    .mid
    .mudl

Post-MVP 

routes 
route 

## Audio Commands 

Available in mash, nu_plugin_live_audio, mush

hosts - show available hosts 
host - no arg show active/live host, with arg set active/live host
devices - show available devices
device - no args show active/live device, with arg set active/live device

Post MVP 

load - from file
play - play loaded file or recorded events
stop - stop playback or record - map tp space bar
monitor - stream received events to ui (meter, bar chart), buffer in memory for pre-roll record
record - buffer input audio in memory, backup to temp file
save - to file 
    .mid
    .mudl
routes
route 
    
## DAW Engines 

Host all shared functionality in mush-core.

MIDI specific functionality in mish-midi-core
Audio specific functionality in mash-audio-core

Shells and plugins are thin front-ends.

---

## Outcome 

#### 8.3.1 Audit `mish-midi-core` ✅ Complete

Findings:

**Production-ready — use as-is:**
- `enumerate.rs`        — `list_midi_ports()` via `midir`
- `timeline.rs`         — `load_midi()`, `load_midi_bytes()`, `TempoMap`
- `engine.rs`           — `DesktopMidiOutput`, `MidiEngineRt`
- `display.rs`          — `list_enumerated_devices()`
- `types.rs`            — `MidiPortInfo`, `MidiPortDirection`, `MidiError`

**Delete — superseded by Conductor + handle_command:**
- `engine.rs`           — `MidiEngine`, `MidiResponse`

**Missing — must be added:**
- `midir` input connection for `midi monitor`
- Port selection — `open_first_port()` hardcodes port 0
- `.mudl` writer
- `.mudl` parser
- Typed error for `load_midi_bytes` — currently `Box<dyn Error>`
- Replace all `eprintln!` in `clock.rs` with `tracing`

**Moves to `mush-core`:**
- `TimelineEvent<P>`    — made generic over payload type
                          `HasSamplePos` impl moves with it
- `EventPayload`        — renamed `MidiPayload`, stays in `mish-midi-core`
- `AudioPayload`        — stub in `mash-audio-core`, post-MVP
- `Clock`               — depends on `mush-core`; keeping it in
                          `mish-midi-core` creates a circular dependency
- `ClockArgs<P>`        — moves with `Clock`

**Stays in `mish-midi-core`:**
- `MidiPayload`         — renamed from `EventPayload`
- `load_midi()`         — returns `Vec<TimelineEvent<MidiPayload>>`
- `list_midi_ports()`
- `DesktopMidiOutput`
- `MidiEngineRt`
- `display.rs`
- `types.rs`

**Nothing moves to `mush-core` from `mish-midi-core` except:**
`TimelineEvent<P>`, `Clock`, `ClockArgs<P>`
`mush-core` already has everything else it needs.

**Circular dependency avoided by:**
`mush-core` owns `TimelineEvent<P>` and `Clock`.
`mish-midi-core` depends on `mush-core` — not the reverse.
`Conductor` in `mush-core` calls into `mish-midi-core` types
via trait objects or concrete types passed through `ClockArgs`.

