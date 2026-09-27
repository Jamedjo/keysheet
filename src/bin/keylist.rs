//! keylist: one program's keybindings, as JSON.
//!
//! `keylist sway` is built in. `keylist <name>` reads the `[lines]` table of
//! `~/.config/keysheet/<name>.toml` if it has one, and otherwise runs
//! `keylist-<name>` from PATH. All three print the same JSON array.
//!
//! stdout is nothing but JSON. Anything worth saying goes to stderr, so the
//! output stays something you can pipe without reading first.

use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use keysheet::adaptors;
use keysheet::sheet::Sheet;

const USAGE: &str = "\
keylist -- one program's keybindings, as JSON

usage: keylist <adaptor> [options]

adaptors

  sway              bindsym lines from ~/.config/sway/config and its includes
  <name>            the [lines] table in ~/.config/keysheet/<name>.toml, or
                    keylist-<name> from PATH, passed the rest of the arguments
  vim               ~/.vimrc, through the [lines] table of the shipped sheet

options for sway

  --all             every bindsym found, in the order the config writes
                    them: all modes, nothing dropped, folded or deduped
  --config <path>   the config to start from
                    (default: $XDG_CONFIG_HOME/sway/config)
  --compact         one line of JSON rather than indented

writing an adaptor

  A command named keylist-<name>, anywhere on PATH, that prints an array of
  {\"keys\", \"description\"} to stdout. \"command\" and \"mode\" are optional.
  Nothing else is required of it, in any language:

    [{\"keys\": \"<Leader>ff\", \"description\": \"Find a file\"}]
";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(adaptor) = args.next() else {
        eprint!("{USAGE}");
        return ExitCode::FAILURE;
    };
    if adaptor == "-h" || adaptor == "--help" {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }

    let rest: Vec<String> = args.collect();
    if adaptors::built_in(&adaptor) {
        return sway(rest);
    }
    let sheet = match Sheet::load(&adaptor, &Sheet::path_for(&adaptor)) {
        Ok(sheet) => sheet,
        Err(complaint) => {
            eprintln!("keylist: {complaint}");
            return ExitCode::FAILURE;
        }
    };
    match sheet.lines() {
        Some(lines) => from_lines(&adaptor, lines, &rest),
        None => delegate(&adaptor, rest),
    }
}

fn from_lines(adaptor: &str, lines: &adaptors::lines::Lines, rest: &[String]) -> ExitCode {
    let compact = match rest {
        [] => false,
        [flag] if flag == "--compact" => true,
        _ => {
            eprintln!("keylist {adaptor}: only --compact is understood");
            return ExitCode::FAILURE;
        }
    };
    match adaptors::lines::read(lines) {
        Ok(bindings) => {
            println!("{}", render(&bindings, compact));
            ExitCode::SUCCESS
        }
        Err(complaint) => {
            eprintln!("keylist {adaptor}: {complaint}");
            ExitCode::FAILURE
        }
    }
}

fn render(bindings: &[keysheet::binding::Binding], compact: bool) -> String {
    let json = if compact {
        serde_json::to_string(bindings)
    } else {
        serde_json::to_string_pretty(bindings)
    };
    json.expect("bindings serialise")
}

/// Hand over to `keylist-<name>`, replacing this process.
///
/// exec rather than spawn-and-wait: there is nothing left to do here, and an
/// adaptor that writes slowly or is killed by a signal should look to the
/// caller exactly as it would if it had been run directly.
fn delegate(adaptor: &str, rest: Vec<String>) -> ExitCode {
    let program = format!("keylist-{adaptor}");
    let error = Command::new(&program).args(&rest).exec();
    eprintln!("keylist: {program}: {error}");
    eprintln!("keylist: no adaptor for {adaptor}. `keylist --help` says how to write one.");
    ExitCode::FAILURE
}

fn sway(args: Vec<String>) -> ExitCode {
    let options = match Options::parse(args) {
        Ok(options) => options,
        Err(complaint) => {
            eprintln!("keylist sway: {complaint}\n");
            eprint!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    // Said out loud rather than printed as an empty list. A config that is not
    // there and a config that binds nothing are different situations, and only
    // one of them is worth drawing a blank sheet for.
    if !options.config.is_file() {
        eprintln!("keylist sway: {}: no such config", options.config.display());
        return ExitCode::FAILURE;
    }

    let found = adaptors::sway::parse(&options.config);
    let listed = if options.all { found } else { adaptors::sway::tidy(found) };

    println!("{}", render(&listed, options.compact));
    ExitCode::SUCCESS
}

struct Options {
    config: PathBuf,
    all: bool,
    compact: bool,
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Options, String> {
        let mut options = Options {
            config: adaptors::sway::default_config(),
            all: false,
            compact: false,
        };

        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--all" => options.all = true,
                "--compact" => options.compact = true,
                "--config" => {
                    options.config = PathBuf::from(
                        args.next().ok_or("--config wants a path after it")?,
                    )
                }
                other => return Err(format!("unknown option {other}")),
            }
        }
        Ok(options)
    }
}
