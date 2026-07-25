"""DevKit install path helpers.

Default install root is a ``dev`` folder chosen per OS:

- Windows: ``C:\\dev`` (system drive + ``\\dev``)
- macOS: ``/opt/dev`` if writable, else ``~/dev``
- Linux: ``/opt/dev`` if writable, else ``~/dev``

Override with the ``DEVKIT_HOME`` environment variable.
Plugins install to ``<dev_root>/<plugin-id>`` (e.g. ``C:\\dev\\flutter``).
"""

from __future__ import annotations

import os
from pathlib import Path

from devkit.platform import HostOS, current_os


def _is_creatable(path: Path) -> bool:
    """Return True if ``path`` exists and is writable, or can be created."""
    if path.exists():
        return os.access(path, os.W_OK)
    parent = path.parent
    while not parent.exists() and parent != parent.parent:
        parent = parent.parent
    return parent.exists() and os.access(parent, os.W_OK)


def preferred_dev_roots() -> list[Path]:
    """Ordered candidate ``dev`` roots for the current OS."""
    host = current_os()
    if host == HostOS.WINDOWS:
        drive = os.environ.get("SystemDrive", "C:")
        return [Path(f"{drive}\\dev")]
    # macOS + Linux (+ other Unix): prefer machine /opt/dev, fall back to ~/dev
    return [Path("/opt/dev"), Path.home() / "dev"]


def default_dev_root() -> Path:
    """Return the best default ``dev`` folder for this machine."""
    for candidate in preferred_dev_roots():
        if _is_creatable(candidate):
            return candidate
    # Last resort — always under the user home
    return Path.home() / "dev"


def home() -> Path:
    """Return DEVKIT_HOME / the machine ``dev`` root."""
    override = os.environ.get("DEVKIT_HOME")
    if override:
        return Path(override).expanduser().resolve()
    return default_dev_root().resolve()


def cache_dir() -> Path:
    """Return the download cache under the dev root."""
    return home() / ".cache"


def plugin_install_dir(plugin_id: str) -> Path:
    """Return ``<dev_root>/<plugin-id>``."""
    return home() / plugin_id


def ensure_home() -> Path:
    """Create the dev root and cache dirs if missing; return the root."""
    root = home()
    root.mkdir(parents=True, exist_ok=True)
    cache_dir().mkdir(parents=True, exist_ok=True)
    return root


def sdks_dir() -> Path:
    """Return the directory that holds installed SDKs (the dev root)."""
    return home()
