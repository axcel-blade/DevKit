//! Minimal ANSI color helpers for the CLI/menu output.
//!
//! No extra crate: DevKit only ever prints plain lines, so a handful of
//! `\x1b[...m` wrappers cover it. Colors are skipped when NO_COLOR is set or
//! stdout isn't a terminal, so piped/redirected output stays clean.

use std::io::IsTerminal;
use std::sync::OnceLock;

fn colors_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
    })
}

fn paint(code: &str, text: &str) -> String {
    if colors_enabled() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub fn bold(text: &str) -> String {
    paint("1", text)
}

pub fn green(text: &str) -> String {
    paint("32", text)
}

pub fn yellow(text: &str) -> String {
    paint("33", text)
}

pub fn red(text: &str) -> String {
    paint("31", text)
}

pub fn cyan(text: &str) -> String {
    paint("36", text)
}

pub fn dim(text: &str) -> String {
    paint("2", text)
}

/// Color a plugin/env status label the same way everywhere it's printed.
pub fn status_label(state: &str) -> String {
    match state {
        "installed" => green(state),
        "partial" => yellow(state),
        _ => dim(state),
    }
}

/// A `[ ok ]` / `[missing]` marker for env checks.
pub fn check_mark(ok: bool) -> String {
    if ok {
        green("[ok]")
    } else {
        red("[missing]")
    }
}
