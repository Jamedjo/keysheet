# keysheet

A sheet of the keybindings you configured, read out of the config that defines
them. A cheatsheet written by hand is wrong the day after you change a binding;
this one cannot drift, because there is nowhere for it to drift from.

Two programs, with JSON between them:

    keylist <program>   reads that program's config and prints what it found
    keysheet <name>     runs that and draws it

Nothing in `keysheet` knows about any particular program. The title, the
footer, the groups and the command to run all come from
`~/.config/keysheet/<name>.toml`, because every one of those is somebody's
particular setup — including the chord that opens the sheet, which may not be
a chord at all.

## The adaptor contract

An adaptor is **any command that prints a JSON array of bindings to stdout**:

```json
[{"keys": "<Leader>ff", "description": "Find a file"}]
```

`keys` and `description` are required; `command` and `mode` are optional, and
only matter if you want to group on what a binding runs or to tell modes apart.
That is the whole interface. `keylist sway` is built in; `keylist vim` runs
`keylist-vim` from PATH, so adding a program means writing one command, in any
language, without touching this repo.

If you are having something write an adaptor for you, the paragraph above is
the brief.

## keylist sway

    $ keylist sway | jq -r '.[] | "\(.keys)\t\(.description)"'
    Super+Return    Start a terminal
    Super+Shift+q   Kill focused window
    Super+d         Start your launcher
    …

It starts at `~/.config/sway/config` and follows every `include`. Three things
make that more than a grep:

- **Variables are resolved.** Otherwise every row reads `$mod+…` rather than
  `Super+…`. Longest name first, so `$mod` does not eat the front of
  `$modifier`.
- **Modes are tracked.** A binding inside `mode "resize"` is not one you can
  press right now, and a list that mixes the two is a list that lies.
- **The comment above a binding is its description.** "Toggle night light"
  beats `exec ~/.local/bin/nightlight-toggle`. Where there is
  no comment, the command stands in for one, tidied a little.

What comes out is then made readable rather than complete: bindings you cannot
press are dropped, so are media keys already labelled by the picture on the
key, runs like `Super+1` … `Super+0` fold into one row, and a binding reachable
three ways is said once. `--all` turns all of that off and prints every
`bindsym` it found in the order the config writes them.

    --all             every bindsym found, nothing dropped, folded or deduped
    --config <path>   the config to start from
    --compact         one line of JSON rather than indented

stdout is nothing but JSON. Anything worth saying — a config that is not there,
an adaptor that does not exist — goes to stderr.

## keylist vim, and other line-based configs

Most keybinding configs put one binding on a line. A `[lines]` table in the
sheet file describes that shape, and `keylist <name>` reads it:

```toml
[lines]
files   = ["~/.vimrc"]
comment = '"'
binding = '^\s*(?P<mode>[nvxsoilct])?(?:nore)?map!?\s+(?P<keys>\S+)\s+(?P<command>.+)$'
strip   = ['^:', '(?i)<cr>', '<Plug>']
skip    = ['(?i)^<nop>$']
```

`binding` needs `keys` and `command` captures; `mode` is optional. The
comment line directly above a binding is its description. Without one, the
command is used, minus anything matching `strip`. Bindings whose command
matches `skip` are left out.

`sheets/vim.toml` is compiled in, so `keysheet vim` shows your `~/.vimrc`
maps with no config written. tmux, i3 and hyprland need only a `[lines]`
table of their own.

## The sheet file

`keysheet sway` reads `~/.config/keysheet/sway.toml`. Every key in it is
optional, and a sheet with no file at all still works: it runs `keylist <name>`
and prints whatever comes back.

```toml
title  = "Keyboard"
hint   = "any key runs its binding and closes the sheet"
source = ["keylist", "sway"]    # the default for a sheet named sway
style  = "style.css"            # relative to ~/.config/keysheet
fallback = "Other"              # omit and unmatched bindings are dropped
order  = ["Focus", "Move", "Launch"]

[foot]
note  = "read dynamically from ~/.config/sway/config"
keys  = "Super+?"
label = "opens this sheet"

[[group]]
name = "Focus"
keywords = ["focus"]

[[group]]
name = "Learning"
keys = ["Super+Tab", "Super+grave"]

[[group]]
name = "Noise"
keywords = ["swaynag"]
hidden = true
```

Groups are tried in the order they are written, most specific first, because
the words overlap: "move container to workspace" is about workspaces though it
moves a window. That is a matching order and not a reading one, so `order` may
list them again in the order they should be printed.

Three things make it a sheet rather than a dump. `keys` matches a whole chord
exactly, which is how you call out the handful you are still learning. `hidden`
drops what it matches instead of printing it. And leaving `fallback` out drops
everything no group claimed — so a sheet can be a shortlist rather than an
inventory, which is what you want the moment a program has three hundred
bindings and you care about nine.

`sheets/sway.toml` here is compiled in and stands in until you write your own.

## keysheet

An ordinary window, not a layer-shell overlay: a reference is something you may
want beside the thing you are doing, moved or tabbed behind something else, and
that is what the window tree is for.

Any key closes it, and so does losing focus. That is the point rather than a
hazard — what you open it for is a shortcut, and pressing the shortcut is the
same gesture as putting the sheet away. `KEYSHEET_PIN=1` leaves it up, which is
how you photograph it.

Each sheet gets its own application id — `dev.jamedjo.keysheet.sway` — so two
sheets toggle independently rather than closing each other, and one window rule
can still catch them all:

    for_window [app_id="^dev\.jamedjo\.keysheet"] floating enable, resize set 1400 900, move position center

## Building

The sheet needs GTK 4.12+; `keylist` needs nothing but libc.

    cargo build --release
    install -Dm755 target/release/keylist  ~/.local/bin/keylist
    install -Dm755 target/release/keysheet ~/.local/bin/keysheet

Install them together: the sheet looks for its source command beside itself
first, and only then on PATH.

    cargo build --release --bin keylist --no-default-features

builds the reading half alone, with no GTK anywhere near it.
