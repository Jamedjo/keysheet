//! Two halves of one tool, with JSON as the seam between them.
//!
//! `sway-keylist` reads the sway config and prints what it found. It knows
//! nothing about screens: it parses (`bindings`), optionally sorts into
//! groups (`groups`), and writes JSON to stdout, which makes the list
//! something a script can have as well as something you can look at.
//!
//! `sway-cheatsheet` runs that and draws the result (`sheet`). It has no
//! parser of its own -- the only bindings it can show are the ones the
//! command prints, so what is on the sheet and what comes out of the pipe
//! cannot disagree.

pub mod bindings;
pub mod groups;
pub mod sheet;

use std::path::PathBuf;

/// Where both halves look for their files: sway's config, and the taxonomy
/// and stylesheet beside it.
pub fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config")
        })
}
