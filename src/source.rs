//! Running a sheet's source command and reading what it prints.
//!
//! The sheet has no parser. It runs a command and draws the JSON that comes
//! back, so what is on screen and what a script sees cannot disagree -- and
//! so an adaptor can be written in any language by anyone, without this repo
//! knowing it exists. It costs one fork and exec per launch,
//! which is a few milliseconds against building a window.

use crate::binding::Binding;
use std::path::PathBuf;
use std::process::Command;

/// Run the command and read its bindings, or say why not.
///
/// The error is a sentence for a person: it goes on the sheet in place of the
/// bindings, because a window with nothing in it does not say which half is
/// missing.
pub fn run(argv: &[String]) -> Result<Vec<Binding>, String> {
    let (program, args) = argv.split_first().ok_or("the source command is empty")?;
    let program = resolve(program);

    let output = Command::new(&program)
        .args(args)
        .output()
        .map_err(|e| format!("could not run {}: {e}", program.display()))?;

    if !output.status.success() {
        let complaint = String::from_utf8_lossy(&output.stderr);
        let complaint = complaint.trim();
        return Err(if complaint.is_empty() {
            format!("{} failed ({})", program.display(), output.status)
        } else {
            complaint.to_string()
        });
    }

    serde_json::from_slice(&output.stdout).map_err(|e| {
        format!("{} did not print a list of bindings: {e}", program.display())
    })
}

/// Prefer a sibling of the running binary, then whatever is on PATH.
///
/// keysheet and keylist are installed together and meant to stay together;
/// picking up a keylist from elsewhere on PATH while ignoring the one sitting
/// beside the running sheet would be the wrong way round. A source command
/// with a path in it is taken as written.
fn resolve(program: &str) -> PathBuf {
    if program.contains('/') {
        return PathBuf::from(program);
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(program)))
        .filter(|sibling| sibling.is_file())
        .unwrap_or_else(|| PathBuf::from(program))
}
