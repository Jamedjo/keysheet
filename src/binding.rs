//! One binding, and the whole contract an adaptor has to meet.
//!
//! Deliberately four fields, three of them optional. An adaptor is any command
//! that prints an array of these to stdout -- which is a thing you can write in
//! a shell script, or describe to somebody in a sentence, and that matters more
//! here than expressing every shape a keymap can take.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    /// The chord, spelled the way the program spells it: "Super+Shift+d",
    /// "<Leader>ff", "C-x C-s". Nothing here parses it except to split a
    /// chord into caps on the sheet.
    pub keys: String,

    /// What it does, in words. The one field worth writing well: it is what
    /// the reader is scanning for.
    pub description: String,

    /// What it actually runs, where that is a meaningful thing to say. Kept
    /// because grouping matches against it and because "what is this bound
    /// to" is sometimes the question -- not because it belongs on the sheet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// The mode it belongs to, for programs that have them. Absent means
    /// always live.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

impl Binding {
    pub fn new(keys: String, description: String) -> Binding {
        Binding { keys, description, command: None, mode: None }
    }

    /// The text a group's keywords are matched against.
    pub fn haystack(&self) -> String {
        let mut text = format!("{} {}", self.keys, self.description);
        if let Some(command) = &self.command {
            text.push(' ');
            text.push_str(command);
        }
        text.to_lowercase()
    }

    pub fn command(&self) -> &str {
        self.command.as_deref().unwrap_or_default()
    }

    pub fn mode(&self) -> &str {
        self.mode.as_deref().unwrap_or("default")
    }
}
