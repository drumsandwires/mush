use expectrl::Regex as ExpectRegex;
use expectrl::{Error, Expect, spawn};
use std::path::PathBuf;

pub struct ReplSession<E: Expect> {
    session: E,
    prompt_pattern: String,
}

impl<E: Expect> ReplSession<E> {
    pub fn execute(&mut self, cmd: &str) -> Result<Vec<u8>, Error> {
        self.session.send_line(cmd)?;
        let found = self
            .session
            .expect(ExpectRegex(self.prompt_pattern.clone()))?;
        Ok(found.before().to_vec())
    }

    pub fn exit(&mut self) -> Result<(), Error> {
        self.session.send_line("quit")?;
        Ok(())
    }
}

/// Resolves the path to a workspace binary built by Cargo.
pub fn cargo_bin(name: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("tests/ has no parent");

    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("target"));

    target_dir.join("debug").join(name)
}

/// Strips ANSI escape codes from raw PTY output bytes.
pub fn clean(bytes: &[u8]) -> String {
    let stripped = strip_ansi_escapes::strip(bytes);
    String::from_utf8_lossy(&stripped).into_owned()
}

fn prompt_regex(prompt: &str) -> String {
    let ansi = r"(?:\x1b\[[0-9;]*[a-zA-Z])*";
    let escaped = regex::escape(prompt.trim_end());
    format!("{ansi}{escaped}{ansi}")
}

pub fn spawn_repl(bin: &str, prompt: &str) -> Result<ReplSession<impl Expect>, Error> {
    let path = cargo_bin(bin);
    let mut session = spawn(path.to_str().expect("non-UTF8 path"))?;

    let pattern = prompt_regex(prompt);
    session.expect(ExpectRegex(pattern.clone()))?;

    Ok(ReplSession {
        session,
        prompt_pattern: pattern,
    })
}
