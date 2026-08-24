//! Getting the list, which is done by asking for it rather than by parsing it.
//!
//! The sheet has no parser. It runs `sway-keylist --grouped` and draws what
//! comes back, so the only bindings it can show are the ones the command
//! prints -- which means the sheet cannot quietly drift from the JSON a script
//! sees. It costs one fork and exec each time the sheet opens, which is a few
//! milliseconds against a window build.

use crate::groups::Group;
use std::path::PathBuf;
use std::process::Command;

/// The other half of this pair.
pub const KEYLIST: &str = "sway-keylist";

/// Run the command and read its groups, or say why not.
///
/// The error is a sentence for a person: it goes on the sheet in place of the
/// bindings, because a window with nothing in it does not tell you that the
/// half doing the reading is missing.
pub fn load() -> Result<Vec<Group>, String> {
    let keylist = keylist_path();
    let output = Command::new(&keylist)
        .arg("--grouped")
        .arg("--compact")
        .output()
        .map_err(|e| format!("could not run {}: {e}", keylist.display()))?;

    if !output.status.success() {
        let complaint = String::from_utf8_lossy(&output.stderr);
        let complaint = complaint.trim();
        return Err(if complaint.is_empty() {
            format!("{KEYLIST} failed ({})", output.status)
        } else {
            complaint.to_string()
        });
    }

    serde_json::from_slice(&output.stdout).map_err(|e| format!("{KEYLIST} printed no groups: {e}"))
}

/// Prefer the sibling next to this binary, then whatever is on PATH.
///
/// The pair is installed together and is meant to stay together -- picking up
/// a `sway-keylist` from somewhere else on PATH while ignoring the one sitting
/// beside the running sheet would be the wrong way round. PATH is the fallback
/// for a build tree, where the two are not siblings yet.
fn keylist_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(KEYLIST)))
        .filter(|sibling| sibling.is_file())
        .unwrap_or_else(|| PathBuf::from(KEYLIST))
}
