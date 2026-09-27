//! The adaptors built in.
//!
//! `lines` reads any config with one binding per line, described by a
//! `[lines]` table in the sheet file. `sway` needs more than that: mode
//! blocks, variables and includes. Anything else is a `keylist-<name>`
//! command on PATH that prints a JSON array of bindings.

pub mod lines;
pub mod sway;

/// Whether `keylist <name>` is answered from in here.
pub fn built_in(name: &str) -> bool {
    name == "sway"
}
