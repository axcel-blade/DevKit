"""Plugin contract for DevKit installers."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path


class InstallState(str, Enum):
    """Whether a plugin appears installed."""

    INSTALLED = "installed"
    NOT_INSTALLED = "not_installed"
    PARTIAL = "partial"


@dataclass(frozen=True)
class EnvSpec:
    """Environment changes a plugin needs after install.

    Attributes:
        paths: Directories to prepend to the user PATH.
        vars: Environment variable name -> value mappings.
    """

    paths: list[Path] = field(default_factory=list)
    vars: dict[str, str] = field(default_factory=dict)


@dataclass(frozen=True)
class InstallContext:
    """Runtime context passed into plugin install/uninstall.

    Attributes:
        install_dir: Target folder, e.g. ``C:\\dev\\flutter``.
        home: Machine ``dev`` root, e.g. ``C:\\dev`` (see ``DEVKIT_HOME``).
        version: Optional SDK version / feature request from ``install --version``.
        channel: Optional release channel from ``install --channel`` (e.g. Flutter).
    """

    install_dir: Path
    home: Path
    version: str | None = None
    channel: str | None = None


@dataclass(frozen=True)
class InstallResult:
    """Outcome of a successful plugin install."""

    install_dir: Path
    message: str = ""


@dataclass(frozen=True)
class PluginStatus:
    """Reported status for a plugin."""

    state: InstallState
    install_dir: Path | None = None
    detail: str = ""


class Plugin(ABC):
    """Abstract installer plugin.

    Subclass this and place the module under ``devkit.plugins`` (or register
    it with the registry) to expose ``devkit install <id>``.
    """

    id: str
    name: str
    description: str = ""

    @abstractmethod
    def status(self, ctx: InstallContext) -> PluginStatus:
        """Return whether this plugin is installed."""

    @abstractmethod
    def install(self, ctx: InstallContext) -> InstallResult:
        """Install the tool into ``ctx.install_dir``."""

    @abstractmethod
    def uninstall(self, ctx: InstallContext) -> None:
        """Remove the tool from ``ctx.install_dir``."""

    @abstractmethod
    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        """Declare PATH entries and env vars for this install."""
