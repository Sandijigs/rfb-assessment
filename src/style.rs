use owo_colors::OwoColorize;
use std::fmt::Display;
use std::io::IsTerminal;
use std::sync::OnceLock;

fn colors_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        // Respect https://no-color.org/ and only colour when stdout is a real TTY.
        std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
    })
}

pub fn magenta<T: Display>(v: T) -> String {
    if colors_enabled() {
        v.to_string().bright_magenta().bold().to_string()
    } else {
        v.to_string()
    }
}

pub fn green<T: Display>(v: T) -> String {
    if colors_enabled() {
        v.to_string().bright_green().bold().to_string()
    } else {
        v.to_string()
    }
}

pub fn cyan<T: Display>(v: T) -> String {
    if colors_enabled() {
        v.to_string().bright_cyan().bold().to_string()
    } else {
        v.to_string()
    }
}

pub fn yellow<T: Display>(v: T) -> String {
    if colors_enabled() {
        v.to_string().bright_yellow().bold().to_string()
    } else {
        v.to_string()
    }
}
