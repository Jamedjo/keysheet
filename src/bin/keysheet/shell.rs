//! Dressing the window, which is the only thing here that is about GTK rather
//! than about bindings.

use gtk::gdk;
use std::path::Path;

/// Load a stylesheet for the whole application.
///
/// The compiled-in `fallback` is used when the file is not there, so a sheet
/// looks like itself on a machine with no config written yet. Read once at
/// startup.
pub fn load_css(path: &Path, fallback: &str) {
    let css = std::fs::read_to_string(path).unwrap_or_else(|_| fallback.to_string());

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
