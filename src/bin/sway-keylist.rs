//! sway-keylist: the keybindings sway is running, as JSON.
//!
//! A cheatsheet written by hand is wrong the day after you change a binding.
//! This reads ~/.config/sway/config and everything it includes, resolves the
//! variables, takes the comment above each binding as its description, and
//! prints the lot. What you do with it is your business: sway-cheatsheet draws
//! it, `jq` queries it, and a `--grouped` run is already sorted for printing.
//!
//! stdout is nothing but JSON. Anything worth saying goes to stderr, so the
//! output stays something you can pipe without reading first.

use std::path::PathBuf;
use std::process::ExitCode;

use sway_cheatsheet::bindings;
use sway_cheatsheet::groups::Groups;

const USAGE: &str = "\
sway-keylist -- the keybindings in your sway config, as JSON

usage: sway-keylist [options]

  --grouped         sort into the groups in groups.toml, in printing order,
                    as [{\"name\", \"bindings\"}] instead of a flat list
  --all             every bindsym found, in the order the config writes
                    them: all modes, nothing dropped, folded or deduped
  --config <path>   sway config to start from
                    (default: $XDG_CONFIG_HOME/sway/config)
  --groups <path>   taxonomy to sort by, with --grouped
                    (default: $XDG_CONFIG_HOME/sway-cheatsheet/groups.toml,
                    falling back to the one compiled in)
  --compact         one line of JSON rather than indented
  -h, --help        this
";

fn main() -> ExitCode {
    let options = match Options::parse() {
        Ok(Some(options)) => options,
        Ok(None) => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(complaint) => {
            eprintln!("sway-keylist: {complaint}\n");
            eprint!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    // Said out loud rather than printed as an empty list. A config that is not
    // there and a config that binds nothing are different situations, and only
    // one of them is worth drawing a blank sheet for.
    if !options.config.is_file() {
        eprintln!("sway-keylist: {}: no such config", options.config.display());
        return ExitCode::FAILURE;
    }

    let found = bindings::parse(&options.config);
    let listed = if options.all { found } else { bindings::tidy(found) };

    let json = if options.grouped {
        let taxonomy = Groups::load(&options.groups);
        render(&taxonomy.sort(listed), options.compact)
    } else {
        render(&listed, options.compact)
    };
    println!("{json}");
    ExitCode::SUCCESS
}

fn render<T: serde::Serialize>(value: &T, compact: bool) -> String {
    let json = if compact {
        serde_json::to_string(value)
    } else {
        serde_json::to_string_pretty(value)
    };
    json.expect("bindings serialise")
}

struct Options {
    config: PathBuf,
    groups: PathBuf,
    grouped: bool,
    all: bool,
    compact: bool,
}

impl Options {
    /// `Ok(None)` is --help: asked for and answered, not a failure.
    fn parse() -> Result<Option<Options>, String> {
        let mut options = Options {
            config: bindings::default_config(),
            groups: Groups::default_path(),
            grouped: false,
            all: false,
            compact: false,
        };

        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            let mut value = || {
                args.next()
                    .ok_or_else(|| format!("{arg} wants a path after it"))
            };
            match arg.as_str() {
                "--grouped" => options.grouped = true,
                "--all" => options.all = true,
                "--compact" => options.compact = true,
                "--config" => options.config = PathBuf::from(value()?),
                "--groups" => options.groups = PathBuf::from(value()?),
                "-h" | "--help" => return Ok(None),
                other => return Err(format!("unknown option {other}")),
            }
        }
        Ok(Some(options))
    }
}
