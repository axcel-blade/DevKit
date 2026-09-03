//! Built-in DevKit plugins.
//!
//! Rust has no runtime module scan, so every plugin module is declared and
//! registered here explicitly (mirrors Python's `registry.load_builtin()`).

pub mod android;
pub mod bun;
pub mod cmake;
pub mod composer;
pub mod deno;
pub mod docker;
pub mod dotnet;
pub mod flutter;
pub mod git;
pub mod go;
pub mod gradle;
pub mod hello;
pub mod jdk;
pub mod junit;
pub mod kubectl;
pub mod maven;
pub mod mono;
pub mod mysql;
pub mod ninja;
pub mod node;
pub mod php;
pub mod platform_tools;
pub mod pmd;
pub mod pnpm;
pub mod postgresql;
pub mod python;
pub mod rust;
pub mod sqlite;
pub mod terraform;

use crate::plugin::Plugin;

/// Return every built-in plugin instance.
pub fn all() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(android::AndroidPlugin),
        Box::new(bun::BunPlugin),
        Box::new(cmake::CmakePlugin),
        Box::new(composer::ComposerPlugin),
        Box::new(deno::DenoPlugin),
        Box::new(docker::DockerPlugin),
        Box::new(dotnet::DotnetPlugin),
        Box::new(flutter::FlutterPlugin),
        Box::new(git::GitPlugin),
        Box::new(go::GoPlugin),
        Box::new(gradle::GradlePlugin),
        Box::new(hello::HelloPlugin),
        Box::new(jdk::JdkPlugin),
        Box::new(junit::JunitPlugin),
        Box::new(kubectl::KubectlPlugin),
        Box::new(maven::MavenPlugin),
        Box::new(mono::MonoPlugin),
        Box::new(mysql::MysqlPlugin),
        Box::new(ninja::NinjaPlugin),
        Box::new(node::NodePlugin),
        Box::new(php::PhpPlugin),
        Box::new(platform_tools::PlatformToolsPlugin),
        Box::new(pmd::PmdPlugin),
        Box::new(pnpm::PnpmPlugin),
        Box::new(postgresql::PostgresqlPlugin),
        Box::new(python::PythonPlugin),
        Box::new(rust::RustPlugin),
        Box::new(sqlite::SqlitePlugin),
        Box::new(terraform::TerraformPlugin),
    ]
}
