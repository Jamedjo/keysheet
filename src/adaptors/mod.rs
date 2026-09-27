//! The adaptors built in, which is one.
//!
//! sway is here because it is what keysheet was written against and because
//! its config is a file anyone can read. Everything else is a `keylist-<name>`
//! command somewhere on PATH: the contract is a flat JSON array of bindings on
//! stdout, which is small enough to implement in an afternoon or to describe
//! to something that will write it for you.

pub mod sway;

/// Whether `keylist <name>` is answered from in here.
pub fn built_in(name: &str) -> bool {
    name == "sway"
}
