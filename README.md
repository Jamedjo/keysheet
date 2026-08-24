# sway-cheatsheet

Your sway keybindings, read out of the config that defines them. A cheatsheet
written by hand is wrong the day after you change a binding; this one cannot
drift, because there is nowhere for it to drift from.

Two programs, with JSON between them:

    sway-keylist      reads the config and prints what it found
    sway-cheatsheet   runs that and draws it

The seam is deliberate. The sheet has no parser of its own, so what is on
screen and what comes out of the pipe cannot disagree — and the reading half,
being a filter that writes JSON to stdout, is useful to things that are not a
window.

## sway-keylist

    $ sway-keylist | jq -r '.[] | "\(.keys)\t\(.description)"'
    Super+l         Lock the screen
    Super+Return    Open a terminal
    Super+?         Keyboard help
    …

It starts at `~/.config/sway/config` and follows every `include`. Three things
make that more than a grep:

- **Variables are resolved.** Otherwise every row reads `$mod+…` rather than
  `Super+…`. Longest name first, so `$mod` does not eat the front of
  `$mode_system`.
- **Modes are tracked.** A binding inside `mode "resize"` is not one you can
  press right now, and a list that mixes the two is a list that lies.
- **The comment above a binding is its description.** "Toggle night light"
  beats `exec /home/james/.config/sway/scripts/sway-nightlight`. Where there is
  no comment, the command stands in for one, tidied a little.

What comes out is then made readable rather than complete: bindings you cannot
press are dropped, so are media keys already labelled by the picture on the
key, runs like `Super+1` … `Super+0` fold into one row, and a binding reachable
three ways is said once. `--all` turns all of that off and prints every `bindsym` it
found in the order the config writes them, which is the mode to reach for when
the question is "what is actually bound" rather than "what should I put on a
sheet".

    --grouped         sort into the groups in groups.toml, in printing order
    --all             every bindsym found, nothing dropped, folded or deduped
    --config <path>   sway config to start from
    --groups <path>   taxonomy to sort by, with --grouped
    --compact         one line of JSON rather than indented

stdout is nothing but JSON. Anything worth saying — a groups.toml that does not
parse, a config that is not there — goes to stderr.

## Groups

`groups.toml` decides what a binding is filed under. The one compiled in covers
what sway's own example configuration binds and nothing else, because anything
past that is somebody's particular setup; a copy at
`~/.config/sway-cheatsheet/groups.toml` replaces it wholesale.

Groups are tried in the order they are written, most specific first, because
the words overlap: "move container to workspace" is about workspaces though it
moves a window. That is a matching order and not a reading one, so `order` may
list them again in the order they should be printed.

## sway-cheatsheet

An ordinary window, not a layer-shell overlay: a reference is something you may
want beside the thing you are doing, moved or tabbed behind something else, and
that is what the window tree is for.

Any key closes it, and so does losing focus. That is the point rather than a
hazard — what you open it for is a shortcut, and pressing the shortcut is the
same gesture as putting the sheet away. `SWAY_CHEATSHEET_PIN` leaves it up,
which is how you photograph it or work on it.

The groups are dealt by hand into lanes rather than given to a flow box, always
to whichever lane is shortest. A flow box gives every child in a line the
height of the tallest, which with groups of four rows and groups of twelve left
craters.

## Building

The sheet needs GTK 4.12+; `sway-keylist` needs nothing but libc.

    cargo build --release
    install -Dm755 target/release/sway-keylist ~/.local/bin/sway-keylist
    install -Dm755 target/release/sway-cheatsheet ~/.local/bin/sway-cheatsheet

Install them together: the sheet looks for `sway-keylist` beside itself first,
and only then on PATH.

    cargo build --release --bin sway-keylist --no-default-features

builds the reading half alone, with no GTK anywhere near it.

## Theming

`cheatsheet.css` here is compiled in as the fallback. The one that actually
runs is `~/.config/sway-cheatsheet/style.css`, kept in the dotfiles repo beside
waybar's and swaync's. It is read at startup and the window is built fresh
every time it opens, so editing it is the same as reloading it.
