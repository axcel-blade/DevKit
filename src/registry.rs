//! Discover and look up DevKit plugins.
//!
//! Rust has no runtime module scan equivalent to Python's `pkgutil.iter_modules`,
//! so built-in plugins are explicitly listed in `plugins::all()` and registered here.

use crate::plugin::Plugin;
use std::collections::BTreeMap;

pub struct PluginRegistry {
    plugins: BTreeMap<&'static str, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: BTreeMap::new(),
        }
    }

    /// Register a plugin instance; later registrations overwrite the same id.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.insert(plugin.id(), plugin);
    }

    pub fn get(&self, plugin_id: &str) -> Option<&dyn Plugin> {
        self.plugins.get(plugin_id).map(|p| p.as_ref())
    }

    pub fn require(&self, plugin_id: &str) -> anyhow::Result<&dyn Plugin> {
        self.get(plugin_id).ok_or_else(|| {
            let known = self.ids().join(", ");
            let known = if known.is_empty() {
                "(none)".to_string()
            } else {
                known
            };
            anyhow::anyhow!("Unknown plugin '{plugin_id}'. Known: {known}")
        })
    }

    /// Return plugins sorted by id.
    pub fn all(&self) -> Vec<&dyn Plugin> {
        self.plugins.values().map(|p| p.as_ref()).collect()
    }

    /// Return sorted plugin ids.
    pub fn ids(&self) -> Vec<&'static str> {
        self.plugins.keys().copied().collect()
    }

    pub fn load_builtin(&mut self) {
        for plugin in crate::plugins::all() {
            self.register(plugin);
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a registry with built-in plugins.
pub fn default_registry() -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    registry.load_builtin();
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_has_core_plugins() {
        let registry = default_registry();
        assert!(registry.get("hello").is_some());
        assert!(registry.get("junit").is_some());
        assert!(registry.get("gradle").is_some());
        assert!(registry.get("maven").is_some());
        assert!(registry.get("pmd").is_some());
        assert!(registry.require("does-not-exist").is_err());
    }

    #[test]
    fn ids_are_sorted() {
        let registry = default_registry();
        let ids = registry.ids();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }
}
