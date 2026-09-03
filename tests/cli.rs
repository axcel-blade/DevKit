//! CLI integration tests against the compiled binary.
//!
//! These tests run the compiled binary as a subprocess — there is no
//! injection point to fake the registry/`~/.devkit/env.sh` writer. This
//! file only exercises commands that do not call `EnvManager::apply`/
//! `revert` (not `install`/`uninstall`), so `cargo test` does not mutate
//! the developer's environment. Install/uninstall is covered in plugin
//! unit tests (see `src/plugins/hello.rs`).

use assert_cmd::Command;
use predicates::prelude::*;
use std::sync::Once;

static WARMUP: Once = Once::new();

fn devkit() -> Command {
    // Some macOS CI runners transiently fail to exec a binary immediately
    // after it's built ("Permission denied" — a Gatekeeper/AMFI check
    // settling, not a real permissions problem; see e.g.
    // rust-lang/cargo#5045 and similar reports across the Rust ecosystem).
    // Warm the binary up once, retrying briefly, before any test's real
    // assertions run against it.
    WARMUP.call_once(|| {
        for attempt in 0..10u32 {
            match Command::cargo_bin("devkit")
                .unwrap()
                .arg("--version")
                .output()
            {
                Ok(_) => break,
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    std::thread::sleep(std::time::Duration::from_millis(
                        100 * (attempt as u64 + 1),
                    ));
                }
                Err(_) => break,
            }
        }
    });
    Command::cargo_bin("devkit").unwrap()
}

#[test]
fn list_shows_registered_plugins() {
    let tmp = tempfile::tempdir().unwrap();
    // Isolate from the machine `dev` root. On macOS/Linux CI `/opt` often
    // looks owner-writable but the runner cannot create `/opt/dev`.
    devkit()
        .arg("list")
        .env("DEVKIT_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("git"))
        .stdout(predicate::str::contains("junit"))
        .stdout(predicate::str::contains("gradle"))
        .stdout(predicate::str::contains("maven"))
        .stdout(predicate::str::contains("pmd"));
}

#[test]
fn doctor_reports_dev_root_and_plugins() {
    let tmp = tempfile::tempdir().unwrap();
    devkit()
        .arg("doctor")
        .env("DEVKIT_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Dev root"))
        .stdout(predicate::str::contains("Rustc:"))
        .stdout(predicate::str::contains("Cargo:"))
        .stdout(predicate::str::contains("Internet:"))
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn status_unknown_plugin_before_install_is_not_installed() {
    let tmp = tempfile::tempdir().unwrap();
    devkit()
        .args(["status", "hello"])
        .env("DEVKIT_HOME", tmp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("not_installed"));
}

#[test]
fn install_reports_unknown_plugin() {
    let tmp = tempfile::tempdir().unwrap();
    devkit()
        .args(["install", "does-not-exist"])
        .env("DEVKIT_HOME", tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown plugin"));
}

#[test]
fn menu_lists_plugins_and_quits_on_q() {
    let tmp = tempfile::tempdir().unwrap();
    // No subcommand at all — a bare `devkit` should fall through to the menu
    // rather than clap's "a subcommand is required" usage error.
    devkit()
        .env("DEVKIT_HOME", tmp.path())
        .write_stdin("q\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("plugin menu"))
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn menu_via_explicit_subcommand_quits_on_eof() {
    let tmp = tempfile::tempdir().unwrap();
    // Closed stdin (EOF) must exit cleanly instead of hanging or erroring.
    devkit()
        .arg("menu")
        .env("DEVKIT_HOME", tmp.path())
        .write_stdin("")
        .assert()
        .success();
}

#[test]
fn version_flag_reports_package_version() {
    devkit()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
