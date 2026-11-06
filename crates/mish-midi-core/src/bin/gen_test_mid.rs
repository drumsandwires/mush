//! Run with: cargo run -p mish-midi-core --bin gen_test_mid
//! Generates tests/fixtures/test.mid — a minimal 4-bar MIDI file
//! at 120 BPM with a simple C major scale.

fn main() {
    // Minimal valid MIDI file — format 0, 1 track, 480 ticks/beat
    // 8 NoteOn/NoteOff pairs — C major scale, quarter notes, 120 BPM
    #[rustfmt::skip]
    let bytes: Vec<u8> = vec![
        // Header chunk
        0x4D, 0x54, 0x68, 0x64,  // "MThd"
        0x00, 0x00, 0x00, 0x06,  // chunk length = 6
        0x00, 0x00,              // format 0
        0x00, 0x01,              // 1 track
        0x01, 0xE0,              // 480 ticks per beat

        // Track chunk
        0x4D, 0x54, 0x72, 0x6B,  // "MTrk"
        0x00, 0x00, 0x00, 0x60,  // chunk length = 96 bytes

        // Tempo: 120 BPM = 500000 us/beat
        0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20,

        // C4 NoteOn  delta=0
        0x00, 0x90, 0x3C, 0x64,
        // C4 NoteOff delta=480
        0x83, 0x60, 0x80, 0x3C, 0x00,

        // D4 NoteOn  delta=0
        0x00, 0x90, 0x3E, 0x64,
        // D4 NoteOff delta=480
        0x83, 0x60, 0x80, 0x3E, 0x00,

        // E4 NoteOn  delta=0
        0x00, 0x90, 0x40, 0x64,
        // E4 NoteOff delta=480
        0x83, 0x60, 0x80, 0x40, 0x00,

        // F4 NoteOn  delta=0
        0x00, 0x90, 0x41, 0x64,
        // F4 NoteOff delta=480
        0x83, 0x60, 0x80, 0x41, 0x00,

        // G4 NoteOn  delta=0
        0x00, 0x90, 0x43, 0x64,
        // G4 NoteOff delta=480
        0x83, 0x60, 0x80, 0x43, 0x00,

        // A4 NoteOn  delta=0
        0x00, 0x90, 0x45, 0x64,
        // A4 NoteOff delta=480
        0x83, 0x60, 0x80, 0x45, 0x00,

        // B4 NoteOn  delta=0
        0x00, 0x90, 0x47, 0x64,
        // B4 NoteOff delta=480
        0x83, 0x60, 0x80, 0x47, 0x00,

        // C5 NoteOn  delta=0
        0x00, 0x90, 0x48, 0x64,
        // C5 NoteOff delta=480
        0x83, 0x60, 0x80, 0x48, 0x00,

        // End of track
        0x00, 0xFF, 0x2F, 0x00,
    ];

    let path = std::path::Path::new("tests/fixtures/test.mid");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, &bytes).unwrap();
    println!("Written {} bytes to {}", bytes.len(), path.display());
}
