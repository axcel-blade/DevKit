//! Built-in DevKit plugins.
//!
//! Rust has no runtime module scan, so every plugin module is declared and
//! registered here explicitly (mirrors Python's `registry.load_builtin()`).

pub mod anaconda;
pub mod android;
pub mod android_ndk;
pub mod bun;
pub mod chocolatey;
pub mod cmake;
pub mod composer;
pub mod deno;
pub mod docker;
pub mod dotnet;
pub mod flutter;
pub mod git;
pub mod go;
pub mod gradle;
pub mod jdk;
pub mod junit;
pub mod kubectl;
pub mod make;
pub mod maven;
pub mod mono;
pub mod msys2;
pub mod mysql;
pub mod ninja;
pub mod node;
pub mod php;
pub mod platform_tools;
pub mod pmd;
pub mod pnpm;
pub mod postgresql;
pub mod python;
pub mod qemu;
pub mod rust;
pub mod sqlite;
pub mod terraform;
pub mod uv;

use crate::plugin::Plugin;

/// Return every built-in plugin instance.
pub fn all() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(anaconda::AnacondaPlugin),
        Box::new(android::AndroidPlugin),
        Box::new(android_ndk::AndroidNdkPlugin),
        Box::new(bun::BunPlugin),
        Box::new(chocolatey::ChocolateyPlugin),
        Box::new(cmake::CmakePlugin),
        Box::new(composer::ComposerPlugin),
        Box::new(deno::DenoPlugin),
        Box::new(docker::DockerPlugin),
        Box::new(dotnet::DotnetPlugin),
        Box::new(flutter::FlutterPlugin),
        Box::new(git::GitPlugin),
        Box::new(go::GoPlugin),
        Box::new(gradle::GradlePlugin),
        Box::new(jdk::JdkPlugin),
        Box::new(junit::JunitPlugin),
        Box::new(kubectl::KubectlPlugin),
        Box::new(make::MakePlugin),
        Box::new(maven::MavenPlugin),
        Box::new(mono::MonoPlugin),
        Box::new(msys2::Msys2Plugin),
        Box::new(mysql::MysqlPlugin),
        Box::new(ninja::NinjaPlugin),
        Box::new(node::NodePlugin),
        Box::new(php::PhpPlugin),
        Box::new(platform_tools::PlatformToolsPlugin),
        Box::new(pmd::PmdPlugin),
        Box::new(pnpm::PnpmPlugin),
        Box::new(postgresql::PostgresqlPlugin),
        Box::new(python::PythonPlugin),
        Box::new(qemu::QemuPlugin),
        Box::new(rust::RustPlugin),
        Box::new(sqlite::SqlitePlugin),
        Box::new(terraform::TerraformPlugin),
        Box::new(uv::UvPlugin),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{current_os, HostOS};

    #[test]
    fn plugins_declare_os_support() {
        for plugin in all() {
            let hosts = plugin.supported_os();
            assert!(!hosts.is_empty(), "{} lists no supported OS", plugin.id());
            assert!(
                !hosts.contains(&HostOS::Other),
                "{} should not list Other",
                plugin.id()
            );
            match plugin.id() {
                "chocolatey" | "msys2" => {
                    assert_eq!(hosts, &[HostOS::Windows], "{}", plugin.id());
                }
                _ => {
                    assert!(
                        hosts.contains(&HostOS::Windows)
                            && hosts.contains(&HostOS::MacOS)
                            && hosts.contains(&HostOS::Linux),
                        "{} should support Windows, macOS, and Linux",
                        plugin.id()
                    );
                }
            }
            assert_eq!(
                plugin.supports_os(current_os()),
                hosts.contains(&current_os())
            );
        }
    }

    #[test]
    fn windows_only_plugins_explain_why_install_fails() {
        use crate::plugin::cannot_install_message;

        for plugin in all() {
            if !matches!(plugin.id(), "chocolatey" | "msys2") {
                continue;
            }
            let msg = cannot_install_message(plugin.as_ref());
            assert!(
                msg.contains(&format!("Cannot install {} on ", plugin.id())),
                "{msg}"
            );
            assert!(
                msg.contains(&format!(
                    "Reason: {} is only supported on Windows.",
                    plugin.name()
                )),
                "{msg}"
            );
        }
    }
}
