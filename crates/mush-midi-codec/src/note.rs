use crate::error::ParseError;

/// Parse a scientific pitch notation string to a MIDI note number (0–127).
///
/// Accepts: `C4`, `Bb3`, `F#5`, `C-1`, `B#3`, `Cb4`, `E#4`, `Fb4`.
/// Case-insensitive.
pub fn note_from_str(s: &str) -> Result<u8, ParseError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ParseError::UnknownNote(s.to_string()));
    }

    let bytes = s.as_bytes();
    let pitch_class = match bytes[0].to_ascii_uppercase() {
        b'C' => 0i8,
        b'D' => 2,
        b'E' => 4,
        b'F' => 5,
        b'G' => 7,
        b'A' => 9,
        b'B' => 11,
        _ => return Err(ParseError::UnknownNote(s.to_string())),
    };

    let mut idx = 1usize;
    let accidental: i8 = if idx < bytes.len() {
        match bytes[idx] {
            b'#' => {
                idx += 1;
                1
            }
            b'b' => {
                idx += 1;
                -1
            }
            _ => 0,
        }
    } else {
        0
    };

    let octave_str = &s[idx..];
    let octave: i8 = octave_str
        .parse()
        .map_err(|_| ParseError::UnknownNote(s.to_string()))?;

    // MIDI note = (octave + 1) * 12 + pitch_class + accidental
    // C-1 = 0, C4 = 60, G9 = 127
    let note = (octave as i16 + 1) * 12 + pitch_class as i16 + accidental as i16;

    if !(0..=127).contains(&note) {
        return Err(ParseError::ByteOutOfRange(note as i64));
    }

    Ok(note as u8)
}

/// Format a MIDI note number (0–127) as scientific pitch notation.
///
/// Flats are preferred over sharps (e.g. `Bb4` not `A#4`).
pub fn note_to_str(note: u8) -> String {
    // Pitch class names — flat preferred for accidentals.
    // const NAMES: [&str; 12] = [
    //     "C", "Db", "D", "Eb", "E", "F", "Fb", "G", "Ab", "A", "Bb", "B",
    // ];
    // Correct flat-preferred mapping:
    const FLAT_NAMES: [&str; 12] = [
        "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
    ];

    let pc = (note % 12) as usize;
    let octave = (note / 12) as i8 - 1;
    format!("{}{}", FLAT_NAMES[pc], octave)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn middle_c() {
        assert_eq!(note_from_str("C4").unwrap(), 60);
    }

    #[test]
    fn lowest_note() {
        assert_eq!(note_from_str("C-1").unwrap(), 0);
    }

    #[test]
    fn highest_note() {
        assert_eq!(note_from_str("G9").unwrap(), 127);
    }

    #[test]
    fn sharp() {
        assert_eq!(note_from_str("C#4").unwrap(), 61);
    }

    #[test]
    fn flat() {
        assert_eq!(note_from_str("Bb4").unwrap(), 70);
    }

    #[test]
    fn enharmonic_b_sharp() {
        // B#3 = C4 = 60
        assert_eq!(note_from_str("B#3").unwrap(), 60);
    }

    #[test]
    fn enharmonic_cb() {
        // Cb4 = B3 = 59
        assert_eq!(note_from_str("Cb4").unwrap(), 59);
    }

    #[test]
    fn enharmonic_e_sharp() {
        // E#4 = F4 = 65
        assert_eq!(note_from_str("E#4").unwrap(), 65);
    }

    #[test]
    fn enharmonic_fb() {
        // Fb4 = E4 = 64
        assert_eq!(note_from_str("Fb4").unwrap(), 64);
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(note_from_str("c4").unwrap(), 60);
        assert_eq!(note_from_str("bb4").unwrap(), 70);
    }

    #[test]
    fn format_middle_c() {
        assert_eq!(note_to_str(60), "C4");
    }

    #[test]
    fn format_flat_preferred() {
        assert_eq!(note_to_str(70), "Bb4"); // not A#4
        assert_eq!(note_to_str(61), "Db4"); // not C#4
    }

    #[test]
    fn format_lowest() {
        assert_eq!(note_to_str(0), "C-1");
    }

    #[test]
    fn format_highest() {
        assert_eq!(note_to_str(127), "G9");
    }

    #[test]
    fn out_of_range() {
        assert!(note_from_str("C10").is_err());
    }

    #[test]
    fn roundtrip_all_notes() {
        for n in 0u8..=127 {
            let s = note_to_str(n);
            assert_eq!(
                note_from_str(&s).unwrap(),
                n,
                "roundtrip failed for note {n}"
            );
        }
    }
}

