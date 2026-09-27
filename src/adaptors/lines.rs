//! Bindings from any config written one binding per line.
//!
//! sway, i3, vim, tmux and hyprland all look like this, so the sheet file
//! describes the shape and this reads it:
//!
//! ```toml
//! [lines]
//! files   = ["~/.vimrc"]
//! comment = '"'
//! binding = '^\s*(?P<mode>[nvxoic])?(nore)?map\s+(?P<keys>\S+)\s+(?P<command>.+)$'
//! ```
//!
//! The comment line directly above a binding is its description. Without
//! one, the command stands in, with anything matching `strip` removed.

use crate::binding::Binding;
use regex::Regex;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lines {
    /// Read in order. `~` is the home directory.
    pub files: Vec<String>,
    /// What a comment line starts with.
    pub comment: Option<String>,
    /// Needs `keys` and `command` captures. `mode` is optional.
    pub binding: String,
    /// Removed from a command before it stands in as a description.
    #[serde(default)]
    pub strip: Vec<String>,
    /// Bindings whose command matches any of these are left out.
    #[serde(default)]
    pub skip: Vec<String>,
}

pub fn read(lines: &Lines) -> Result<Vec<Binding>, String> {
    let binding = Regex::new(&lines.binding).map_err(|e| format!("binding: {e}"))?;
    for name in ["keys", "command"] {
        if !binding.capture_names().any(|n| n == Some(name)) {
            return Err(format!("binding: no (?P<{name}>...) capture"));
        }
    }
    let strip = compile(&lines.strip, "strip")?;
    let skip = compile(&lines.skip, "skip")?;

    let mut out = Vec::new();
    for file in &lines.files {
        let path = expand_home(file);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let mut comment: Option<String> = None;

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                comment = None;
                continue;
            }
            if let Some(body) = lines.comment.as_deref().and_then(|c| trimmed.strip_prefix(c)) {
                let body = body.trim();
                // A commented-out binding is not a description.
                comment = (!body.is_empty() && !binding.is_match(body)).then(|| body.to_string());
                continue;
            }
            let Some(caps) = binding.captures(line) else {
                comment = None;
                continue;
            };
            let command = caps["command"].trim().to_string();
            if skip.iter().any(|re| re.is_match(&command)) {
                comment = None;
                continue;
            }
            let description = comment.take().unwrap_or_else(|| {
                let bare = strip.iter().fold(command.clone(), |s, re| re.replace_all(&s, "").into_owned());
                bare.trim().to_string()
            });
            out.push(Binding {
                keys: caps["keys"].to_string(),
                description,
                command: Some(command),
                mode: caps.name("mode").map(|m| m.as_str().to_string()),
            });
        }
    }
    Ok(out)
}

fn compile(patterns: &[String], field: &str) -> Result<Vec<Regex>, String> {
    patterns
        .iter()
        .map(|p| Regex::new(p).map_err(|e| format!("{field}: {e}")))
        .collect()
}

fn expand_home(path: &str) -> PathBuf {
    match path.strip_prefix("~/") {
        Some(rest) => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(rest),
        None => PathBuf::from(path),
    }
}
