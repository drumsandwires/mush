use crate::error::ParseError;

/// Velocity dynamic markings ordered from softest to loudest.
const DYNAMICS: &[(&str, u8)] = &[
    ("pppp", 8),
    ("ppp",  16),
    ("pp",   32),
    ("p",    48),
    ("mp",   64),
    ("mf",   80),
    ("pf",   88),
    ("f",    96),
    ("ff",   112),
    ("fff",  120),
    ("ffff", 127),
];

/// Parse a velocity dynamic symbol to a MIDI velocity value (0–127).
/// Case-insensitive.
pub fn velocity_from_str(s: &str) -> Result<u8, ParseError> {
    let lower = s.to_ascii_lowercase();
    DYNAMICS
        .iter()
        .find(|(sym, _)| *sym == lower.as_str())
        .map(|(_, val)| *val)
        .ok_or_else(|| ParseError::UnknownVelocity(s.to_string()))
}

/// Format a MIDI velocity value as the nearest dynamic symbol.
/// Ties go to the louder value.
pub fn velocity_to_str(vel: u8) -> &'static str {
    // Find the entry with the smallest absolute distance.
    // On a tie, the later (louder) entry wins because we iterate
    // forward and use <=.
    let mut best_sym = DYNAMICS[0].0;
    let mut best_dist = vel.abs_diff(DYNAMICS[0].1);

    for (sym, val) in DYNAMICS.iter().skip(1) {
        let dist = vel.abs_diff(*val);
        if dist <= best_dist {
            best_dist = dist;
            best_sym  = sym;
        }
    }

    best_sym
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_symbols() {
        for (sym, val) in DYNAMICS {
            assert_eq!(velocity_from_str(sym).unwrap(), *val);
        }
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(velocity_from_str("MF").unwrap(), 80);
        assert_eq!(velocity_from_str("Ff").unwrap(), 112);
    }

    #[test]
    fn parse_unknown() {
        assert!(velocity_from_str("xyz").is_err());
    }

    #[test]
    fn format_exact_values() {
        assert_eq!(velocity_to_str(8),   "pppp");
        assert_eq!(velocity_to_str(80),  "mf");
        assert_eq!(velocity_to_str(127), "ffff");
    }

    #[test]
    fn format_nearest_ties_louder() {
        // midpoint between pp=32 and p=48 is 40 — ties go louder → p
        assert_eq!(velocity_to_str(40), "p");
    }

    #[test]
    fn format_zero_velocity() {
        assert_eq!(velocity_to_str(0), "pppp");
    }

    #[test]
    fn roundtrip_all_symbols() {
        for (sym, val) in DYNAMICS {
            assert_eq!(velocity_to_str(*val), *sym);
        }
    }
}