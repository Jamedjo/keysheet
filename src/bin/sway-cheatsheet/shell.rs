//! Dressing the window, which is the only thing here that is about GTK rather
//! than about bindings.

use gtk::gdk;

/// Load a stylesheet for the whole application.
///
/// `name` picks the file under ~/.config; the compiled-in `fallback` is used
/// when there is none, so the sheet still looks like itself on a machine
/// without the dotfiles. Read once at startup, which is enough: the window is
/// built fresh every time it opens, so editing the file is the same as
/// reloading it.
pub fn load_css(name: &str, fallback: &str) {
    let user = sway_cheatsheet::config_dir().join(name);
    let css = std::fs::read_to_string(&user).unwrap_or_else(|_| fallback.to_string());

    let provider = gtk::CssProvider::new();
    provider.load_from_string(&css);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
