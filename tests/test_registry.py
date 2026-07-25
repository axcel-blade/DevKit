"""Tests for the plugin registry."""

from devkit.plugin import (
    EnvSpec,
    InstallContext,
    InstallResult,
    InstallState,
    Plugin,
    PluginStatus,
)
from devkit.registry import PluginRegistry, default_registry


class _FakePlugin(Plugin):
    id = "fake"
    name = "Fake"
    description = "test"

    def status(self, ctx: InstallContext) -> PluginStatus:
        return PluginStatus(state=InstallState.NOT_INSTALLED)

    def install(self, ctx: InstallContext) -> InstallResult:
        return InstallResult(install_dir=ctx.install_dir)

    def uninstall(self, ctx: InstallContext) -> None:
        return None

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        return EnvSpec()


def test_register_and_get():
    registry = PluginRegistry()
    registry.register(_FakePlugin())
    assert registry.get("fake") is not None
    assert registry.get("missing") is None
    assert registry.ids() == ["fake"]


def test_require_unknown():
    registry = PluginRegistry()
    try:
        registry.require("nope")
        raised = False
    except KeyError:
        raised = True
    assert raised


def test_default_registry_loads_hello():
    registry = default_registry()
    plugin = registry.get("hello")
    assert plugin is not None
    assert plugin.name == "Hello"
