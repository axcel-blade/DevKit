"""DevKit command-line interface.

Commands: list, install, uninstall, status, doctor.
Install flow: plugin.install() then EnvManager.apply(plugin.env_spec()).
"""

from __future__ import annotations

import argparse
import platform
import sys

from devkit import __version__
from devkit.env import EnvManager
from devkit.paths import ensure_home, home, plugin_install_dir
from devkit.platform import os_label
from devkit.plugin import InstallContext, InstallState
from devkit.registry import default_registry


def _context(plugin_id: str) -> InstallContext:
    ensure_home()
    return InstallContext(install_dir=plugin_install_dir(plugin_id), home=home())


def cmd_list(_: argparse.Namespace) -> int:
    registry = default_registry()
    plugins = registry.all()
    if not plugins:
        print("No plugins registered.")
        return 0
    print(f"{'ID':<16} {'NAME':<20} {'STATUS'}")
    print("-" * 50)
    for plugin in plugins:
        ctx = _context(plugin.id)
        status = plugin.status(ctx)
        print(f"{plugin.id:<16} {plugin.name:<20} {status.state.value}")
    return 0


def cmd_install(args: argparse.Namespace) -> int:
    registry = default_registry()
    try:
        plugin = registry.require(args.plugin)
    except KeyError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    ctx = _context(plugin.id)
    status = plugin.status(ctx)
    if status.state == InstallState.INSTALLED and not args.force:
        print(f"{plugin.id} is already installed at {status.install_dir}")
        print("Use --force to reinstall.")
        return 0

    print(f"Installing {plugin.name} ({plugin.id}) into {ctx.install_dir} ...")
    result = plugin.install(ctx)
    spec = plugin.env_spec(ctx)
    EnvManager().apply(spec)
    msg = result.message or "done"
    print(f"Installed {plugin.id}: {msg}")
    print("Environment updated. Open a new terminal for PATH/env changes to take effect.")
    return 0


def cmd_uninstall(args: argparse.Namespace) -> int:
    registry = default_registry()
    try:
        plugin = registry.require(args.plugin)
    except KeyError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    ctx = _context(plugin.id)
    status = plugin.status(ctx)
    if status.state == InstallState.NOT_INSTALLED:
        print(f"{plugin.id} is not installed.")
        return 0

    spec = plugin.env_spec(ctx)
    EnvManager().revert(spec)
    plugin.uninstall(ctx)
    print(f"Uninstalled {plugin.id}.")
    print("Environment updated. Open a new terminal for PATH/env changes to take effect.")
    return 0


def cmd_status(args: argparse.Namespace) -> int:
    registry = default_registry()
    try:
        plugin = registry.require(args.plugin)
    except KeyError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    ctx = _context(plugin.id)
    status = plugin.status(ctx)
    print(f"Plugin:      {plugin.id} ({plugin.name})")
    print(f"State:       {status.state.value}")
    if status.install_dir:
        print(f"Install dir: {status.install_dir}")
    if status.detail:
        print(f"Detail:      {status.detail}")

    spec = plugin.env_spec(ctx)
    checks = EnvManager().check(spec)
    if checks:
        print("Environment:")
        for key, ok in checks.items():
            mark = "ok" if ok else "missing"
            print(f"  [{mark}] {key}")
    return 0 if status.state == InstallState.INSTALLED else 1


def cmd_doctor(_: argparse.Namespace) -> int:
    root = home()
    print(f"DevKit {__version__}")
    print(f"Platform:     {os_label()} ({platform.system()} {platform.release()})")
    print(f"Python:       {sys.version.split()[0]}")
    print(f"Dev root:     {root}  (override with DEVKIT_HOME)")
    print(f"Example path: {plugin_install_dir('flutter')}")

    try:
        ensure_home()
        probe = root / ".write_probe"
        probe.write_text("ok", encoding="utf-8")
        probe.unlink(missing_ok=True)
        print("Root writable: yes")
    except OSError as exc:
        print(f"Root writable: no ({exc})")
        return 1

    print(f"Env backend:  {EnvManager().backend_description()}")
    if platform.system() != "Windows":
        print("             Open a new terminal (or `source ~/.devkit/env.sh`) after install.")

    registry = default_registry()
    print(f"Plugins:      {', '.join(registry.ids()) or '(none)'}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="devkit",
        description="DevKit - developer environment setup application.",
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    sub = parser.add_subparsers(dest="command", required=True)

    p_list = sub.add_parser("list", help="List available plugins")
    p_list.set_defaults(func=cmd_list)

    p_install = sub.add_parser("install", help="Install a plugin")
    p_install.add_argument("plugin", help="Plugin id (e.g. hello)")
    p_install.add_argument(
        "--force",
        action="store_true",
        help="Reinstall even if already installed",
    )
    p_install.set_defaults(func=cmd_install)

    p_uninstall = sub.add_parser("uninstall", help="Uninstall a plugin")
    p_uninstall.add_argument("plugin", help="Plugin id")
    p_uninstall.set_defaults(func=cmd_uninstall)

    p_status = sub.add_parser("status", help="Show plugin install and env status")
    p_status.add_argument("plugin", help="Plugin id")
    p_status.set_defaults(func=cmd_status)

    p_doctor = sub.add_parser("doctor", help="Check DevKit environment health")
    p_doctor.set_defaults(func=cmd_doctor)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
