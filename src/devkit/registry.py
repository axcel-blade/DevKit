"""Discover and look up DevKit plugins.

Built-in plugins live under ``devkit.plugins`` and are loaded via
``pkgutil.iter_modules`` — any ``Plugin`` subclass with an ``id`` is registered.
"""

from __future__ import annotations

import importlib
import pkgutil
from collections.abc import Iterable

from devkit.plugin import Plugin


class PluginRegistry:
    """Registry of available installer plugins."""

    def __init__(self) -> None:
        self._plugins: dict[str, Plugin] = {}

    def register(self, plugin: Plugin) -> None:
        """Register a plugin instance; later registrations overwrite same id."""
        if not getattr(plugin, "id", None):
            raise ValueError("Plugin must define a non-empty id")
        self._plugins[plugin.id] = plugin

    def get(self, plugin_id: str) -> Plugin | None:
        """Return a plugin by id, or None."""
        return self._plugins.get(plugin_id)

    def require(self, plugin_id: str) -> Plugin:
        """Return a plugin by id or raise KeyError."""
        plugin = self.get(plugin_id)
        if plugin is None:
            known = ", ".join(sorted(self._plugins)) or "(none)"
            raise KeyError(f"Unknown plugin '{plugin_id}'. Known: {known}")
        return plugin

    def all(self) -> list[Plugin]:
        """Return plugins sorted by id."""
        return [self._plugins[k] for k in sorted(self._plugins)]

    def ids(self) -> list[str]:
        """Return sorted plugin ids."""
        return sorted(self._plugins)

    def load_builtin(self) -> None:
        """Import all modules under ``devkit.plugins`` and register Plugin subclasses."""
        import devkit.plugins as plugins_pkg

        for module_info in pkgutil.iter_modules(plugins_pkg.__path__, plugins_pkg.__name__ + "."):
            module = importlib.import_module(module_info.name)
            for attr_name in dir(module):
                attr = getattr(module, attr_name)
                if (
                    isinstance(attr, type)
                    and issubclass(attr, Plugin)
                    and attr is not Plugin
                    and getattr(attr, "id", None)
                ):
                    self.register(attr())


def default_registry(extra: Iterable[Plugin] | None = None) -> PluginRegistry:
    """Build a registry with built-in plugins (and optional extras)."""
    registry = PluginRegistry()
    registry.load_builtin()
    if extra:
        for plugin in extra:
            registry.register(plugin)
    return registry
