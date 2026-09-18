// Shared utility modules expose a broader API than any single plugin
// currently exercises (mirrors the Python modules they're ported from,
// where not every helper is called by every plugin).
#![allow(dead_code)]

mod cli;
mod download;
mod env;
mod installers;
mod paths;
mod platform;
mod plugin;
mod plugin_utils;
mod plugins;
mod progress;
mod registry;
mod theme;

fn main() {
    let code = match cli::main(None) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Error: {err:?}");
            1
        }
    };
    std::process::exit(code);
}
