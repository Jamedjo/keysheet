//! keysheet: a sheet of the keybindings you configured, for any program.
//!
//! Two halves, with JSON as the seam between them.
//!
//! An **adaptor** reads one program's configuration and prints its bindings:
//! a flat JSON array of `{keys, description}`, and nothing else is required of
//! it. `keylist` carries the one for sway and dispatches to `keylist-<name>`
//! on PATH for everything else, so adding a program means writing one command
//! -- in any language -- rather than changing anything here.
//!
//! The **sheet** (`keysheet <name>`) reads `~/.config/keysheet/<name>.toml`,
//! runs whatever source it names, sorts the reply into that file's groups and
//! draws it. It has no parser and no taxonomy of its own, and nothing in it
//! knows about sway.

pub mod adaptors;
pub mod binding;
pub mod sheet;
pub mod source;

use std::path::PathBuf;

/// The adaptor multiplexer, and the default source for a sheet.
pub const KEYLIST: &str = "keylist";

pub fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config")
        })
}
