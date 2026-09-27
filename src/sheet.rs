//! A sheet: what to run, what to call it, and how to sort what comes back.
//!
//! Nothing about a sheet is compiled in. `keysheet vim` reads
//! `~/.config/keysheet/vim.toml` and does what it says, which is the only way
//! a tool like this works for somebody whose desk is not this one -- the title
//! on the header, the line along the footer, the chord that opens it and the
//! groups it sorts into are all somebody's particular setup, and none of them
//! is something a program can guess.
//!
//! What it can guess is the source: a sheet named `vim` runs `keylist vim`
//! unless told otherwise, so the smallest useful sheet file is empty.

use crate::adaptors::lines::Lines;
use crate::binding::Binding;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Sheets that come with keysheet, used until the reader writes their own.
///
/// Only for adaptors shipped here, and for the same reason they are: a sane
/// taxonomy for sway is something this repo can have an opinion about, and a
/// sane taxonomy for somebody else's program is not.
fn built_in(name: &str) -> Option<&'static str> {
    match name {
        "sway" => Some(include_str!("../sheets/sway.toml")),
        "vim" => Some(include_str!("../sheets/vim.toml")),
        _ => None,
    }
}

/// A named run of bindings: what the lanes are dealt out of.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub bindings: Vec<Binding>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sheet {
    /// What the header says. Defaults to the sheet's own name.
    title: Option<String>,

    /// The line beside the title. A sheet only needs one if there is
    /// something a first-time reader cannot guess.
    hint: Option<String>,

    /// The command that prints the bindings, as argv. Defaults to
    /// `keylist <name>`.
    source: Option<Vec<String>>,

    /// A stylesheet of this sheet's own, relative to the config directory.
    /// Defaults to the shared `style.css`.
    style: Option<String>,

    /// Where a binding goes when no group claims it. Unset means bindings
    /// that match nothing are dropped, which is how you write a sheet that
    /// shows only what you asked for.
    fallback: Option<String>,

    /// The order groups are printed in, if that is not the order they are
    /// matched in. Anything left out follows in matching order.
    #[serde(default)]
    order: Vec<String>,

    #[serde(default)]
    foot: Foot,

    /// How `keylist <name>` reads a config with one binding per line.
    lines: Option<Lines>,

    /// Matched in the order written, most specific first, because the words
    /// overlap: "move container to workspace" is about workspaces though it
    /// moves a window.
    #[serde(default, rename = "group")]
    groups: Vec<Rule>,
}

/// The band along the bottom. Every part of it is optional and an empty one
/// is not drawn.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Foot {
    /// Where the sheet comes from, which is usually the argument for it.
    pub note: Option<String>,
    /// The chord that opens this sheet, printed as caps. keysheet cannot know
    /// it -- it may be a keybinding, a mouse button, a launcher entry, or
    /// nothing at all -- so it is written here or it is not shown.
    pub keys: Option<String>,
    /// What to say after those caps.
    pub label: Option<String>,
}

/// One entry in the taxonomy: a name, and what puts a binding under it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    name: String,
    /// Matched against the chord, the description and the command, lower-cased.
    #[serde(default)]
    keywords: Vec<String>,
    /// Matched against the start of the command. `exec` catches everything
    /// that launches something without naming every program.
    #[serde(default)]
    command_starts_with: Option<String>,
    /// Matched against the whole chord, exactly. This is how you call out the
    /// handful you are still learning: name them and they get their own group.
    #[serde(default)]
    keys: Vec<String>,
    /// Matched and then dropped. A group of things you never want on the
    /// sheet, written above the ones that would otherwise claim them.
    #[serde(default)]
    hidden: bool,
}

impl Sheet {
    /// Where sheets, stylesheets and anything else of keysheet's live.
    pub fn config_dir() -> PathBuf {
        crate::config_dir().join("keysheet")
    }

    pub fn path_for(name: &str) -> PathBuf {
        Sheet::config_dir().join(format!("{name}.toml"))
    }

    /// Read a sheet, or fall back to the defaults its name implies.
    ///
    /// A missing file is not an error. For an adaptor that ships here there is
    /// a sheet compiled in to stand in for it, so `keysheet sway` works on a
    /// machine with no config at all; for anything else `keysheet vim` runs
    /// `keylist vim` and prints whatever it says under one heading, which is
    /// enough to see an adaptor working before writing any config for it.
    ///
    /// A file that will not parse *is* an error, because falling back silently
    /// would draw a sheet nobody asked for.
    pub fn load(name: &str, path: &Path) -> Result<Sheet, String> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let Some(shipped) = built_in(name) else {
                    return Ok(Sheet::default());
                };
                return Ok(toml::from_str(shipped).expect("the compiled-in sheets parse"));
            }
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn title(&self, name: &str) -> String {
        self.title.clone().unwrap_or_else(|| capitalise(name))
    }

    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    pub fn foot(&self) -> &Foot {
        &self.foot
    }

    pub fn lines(&self) -> Option<&Lines> {
        self.lines.as_ref()
    }

    pub fn style(&self) -> PathBuf {
        Sheet::config_dir().join(self.style.as_deref().unwrap_or("style.css"))
    }

    /// The command to run, defaulting to the adaptor named after the sheet.
    pub fn source(&self, name: &str) -> Vec<String> {
        self.source
            .clone()
            .filter(|argv| !argv.is_empty())
            .unwrap_or_else(|| vec![crate::KEYLIST.to_string(), name.to_string()])
    }

    /// Deal the bindings into groups, in printing order, dropping the empty
    /// ones. A binding lands in the first group that claims it, and in none at
    /// all if nothing does and there is no fallback.
    pub fn sort(&self, bindings: Vec<Binding>) -> Vec<Group> {
        // A sheet with no taxonomy is not a mistake -- it is what you get
        // before you have written one -- so everything goes under the title
        // rather than under a heading the file never mentioned.
        if self.groups.is_empty() && self.fallback.is_none() {
            return vec![Group { name: String::new(), bindings }];
        }

        let mut out: Vec<Group> = self
            .print_order()
            .into_iter()
            .map(|name| Group { name, bindings: Vec::new() })
            .collect();

        for binding in bindings {
            match self.claim(&binding) {
                Some(name) => {
                    if let Some(group) = out.iter_mut().find(|g| g.name == name) {
                        group.bindings.push(binding);
                    }
                }
                None => continue,
            }
        }
        out.retain(|group| !group.bindings.is_empty());
        out
    }

    /// Which group takes this binding, if any. `None` is dropped: either a
    /// hidden group claimed it, or nothing did and there is no fallback.
    fn claim(&self, binding: &Binding) -> Option<String> {
        let haystack = binding.haystack();
        for rule in &self.groups {
            let matched = rule.keywords.iter().any(|word| haystack.contains(word))
                || rule.keys.iter().any(|chord| chord == &binding.keys)
                || rule
                    .command_starts_with
                    .as_deref()
                    .is_some_and(|prefix| binding.command().starts_with(prefix));
            if matched {
                return if rule.hidden { None } else { Some(rule.name.clone()) };
            }
        }
        self.fallback.clone()
    }

    /// The groups in the order they should be printed, the fallback last,
    /// hidden ones nowhere.
    fn print_order(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let visible: Vec<&String> = self
            .groups
            .iter()
            .filter(|rule| !rule.hidden)
            .map(|rule| &rule.name)
            .collect();
        for name in self.order.iter().chain(visible) {
            if !out.contains(name) {
                out.push(name.clone());
            }
        }
        if let Some(fallback) = &self.fallback {
            if !out.contains(fallback) {
                out.push(fallback.clone());
            }
        }
        out
    }
}

/// "sway" -> "Sway". Only the first letter, so "i3wm" does not become "I3WM"
/// and an already-capitalised name is left alone.
fn capitalise(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
