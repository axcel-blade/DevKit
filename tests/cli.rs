//! CLI integration tests against the compiled binary.
//!
//! Unlike the Python test suite (which monkeypatches `EnvManager` to avoid
//! mutating the real user environment), these tests run the real compiled
//! binary as a subprocess — there is no injection point to fake out the
//! registry/`~/.devkit/env.sh` writer. So this file only exercises commands
//! that don't call `EnvManager::apply`/`revert` (i.e. not `install`/
//! `uninstall`), to avoid mutating the developer's real environment on every
//! `cargo test`. The install/uninstall round trip is covered at the plugin
//! level instead (see `src/plugins/hello.rs`'s test, which calls
//! `Plugin::install`/`uninstall` directly without touching `EnvManager`).

use assert_cmd::Command;
use predicates::prelude::*;

fn devkit() -> Command {
    Command::cargo_bin("devkit").unwrap()
}

#[test]
fn list_shows_registered_plugins() {
    devkit()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("git"));
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
fn version_flag_reports_package_version() {
    devkit()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
