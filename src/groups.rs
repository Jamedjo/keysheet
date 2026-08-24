//! How bindings are sorted into groups on the sheet, and where that is decided.
//!
//! Not here, is the short answer. The default below covers what sway's own
//! example configuration binds -- focus, move, resize, the layouts, the
//! scratchpad, workspaces, reload and exit -- and nothing else, because
//! anything else is somebody's particular setup. A `groups.toml` beside the
//! stylesheet replaces it wholesale, which is where a taxonomy full of one
//! person's script names belongs.
//!
//! Groups are tried in the order they are written, most specific first,
//! because the words overlap: "move container to workspace" is about
//! workspaces though it moves a window, and "move focus to the parent
//! container" is about neither. That is a matching order, not a reading one,
//! so `order` may list the groups again in the order they should be printed.

use crate::bindings::Binding;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const DEFAULT: &str = include_str!("../groups.toml");

/// A named run of bindings, and one object in the JSON `--grouped` prints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub bindings: Vec<Binding>,
}

#[derive(Deserialize)]
pub struct Groups {
    /// Where a binding goes when nothing matches.
    #[serde(default = "other")]
    fallback: String,
    #[serde(default, rename = "group")]
    groups: Vec<Rule>,
    /// How the groups should be printed, if that is not the order they are
    /// matched in. Anything left out follows in matching order.
    #[serde(default)]
    order: Vec<String>,
}

/// One entry in the taxonomy: a name, and what puts a binding under it.
#[derive(Deserialize)]
struct Rule {
    name: String,
    /// Matched against the command and its description, lower-cased.
    #[serde(default)]
    keywords: Vec<String>,
    /// Matched against the start of the command. `exec` catches everything
    /// that launches something without listing every program by name.
    #[serde(default)]
    command_starts_with: Option<String>,
}

fn other() -> String {
    "Other".to_string()
}

impl Groups {
    /// Where a taxonomy of one's own is kept, beside the stylesheet.
    pub fn default_path() -> PathBuf {
        crate::config_dir().join("sway-cheatsheet/groups.toml")
    }

    pub fn load(path: &Path) -> Groups {
        let text = std::fs::read_to_string(path).unwrap_or_else(|_| DEFAULT.to_string());
        toml::from_str(&text).unwrap_or_else(|e| {
            // A broken file is worth saying out loud rather than silently
            // falling back to a taxonomy the reader did not ask for. stderr,
            // so it never lands in the JSON on stdout.
            eprintln!("sway-keylist: {}: {e}", path.display());
            toml::from_str(DEFAULT).expect("the compiled-in groups parse")
        })
    }

    /// Deal the bindings into groups, in printing order, dropping the empty
    /// ones. A binding lands in exactly one group -- the first that claims it.
    pub fn sort(&self, bindings: Vec<Binding>) -> Vec<Group> {
        let mut out: Vec<Group> = self
            .order()
            .into_iter()
            .map(|name| Group { name, bindings: Vec::new() })
            .collect();
        for binding in bindings {
            let name = self.category(&binding.command, &binding.description);
            if let Some(group) = out.iter_mut().find(|g| g.name == name) {
                group.bindings.push(binding);
            }
        }
        out.retain(|group| !group.bindings.is_empty());
        out
    }

    pub fn category(&self, command: &str, description: &str) -> String {
        let text = format!("{command} {description}").to_lowercase();
        for group in &self.groups {
            if group.keywords.iter().any(|word| text.contains(word)) {
                return group.name.clone();
            }
            if let Some(prefix) = &group.command_starts_with {
                if command.starts_with(prefix.as_str()) {
                    return group.name.clone();
                }
            }
        }
        self.fallback.clone()
    }

    /// The groups in the order they should be printed, the fallback last.
    pub fn order(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for name in self.order.iter().chain(self.groups.iter().map(|g| &g.name)) {
            if !out.contains(name) {
                out.push(name.clone());
            }
        }
        if !out.contains(&self.fallback) {
            out.push(self.fallback.clone());
        }
        out
    }
}
