"""User environment PATH and variable management.

- Windows: user environment via the registry
- macOS / Linux: ``~/.devkit/env.sh`` plus a source line in the user shell profile
"""

from __future__ import annotations

import os
import re
from pathlib import Path

from devkit.platform import is_windows, os_label, primary_shell_profile
from devkit.plugin import EnvSpec

_ENV_SH_BEGIN = "# >>> DevKit >>>"
_ENV_SH_END = "# <<< DevKit <<<"
_PROFILE_BEGIN = "# >>> DevKit >>>"
_PROFILE_END = "# <<< DevKit <<<"
_SOURCE_LINE = 'source "$HOME/.devkit/env.sh"'


def _norm_path(value: Path | str) -> str:
    return str(Path(value).expanduser().resolve())


def _path_list(path_value: str) -> list[str]:
    if not path_value:
        return []
    return [p for p in path_value.split(os.pathsep) if p]


def _join_paths(parts: list[str]) -> str:
    return os.pathsep.join(parts)


class EnvManager:
    """Apply and reverse :class:`EnvSpec` on the user environment."""

    def apply(self, spec: EnvSpec) -> None:
        """Prepend PATH entries and set env vars from ``spec``."""
        if is_windows():
            self._apply_windows(spec)
        else:
            self._apply_unix(spec)

    def revert(self, spec: EnvSpec) -> None:
        """Remove PATH entries and env vars declared by ``spec``."""
        if is_windows():
            self._revert_windows(spec)
        else:
            self._revert_unix(spec)

    def check(self, spec: EnvSpec) -> dict[str, bool]:
        """Return whether each expected PATH/var is present in the user env."""
        if is_windows():
            return self._check_windows(spec)
        return self._check_unix(spec)

    def backend_description(self) -> str:
        """Short description of how env changes are persisted on this OS."""
        if is_windows():
            return "Windows user environment (registry)"
        profile = primary_shell_profile()
        return f"{os_label()} shell: ~/.devkit/env.sh (sourced from {profile})"

    # --- Windows (user environment via registry) ---

    def _apply_windows(self, spec: EnvSpec) -> None:
        path_entries = [_norm_path(p) for p in spec.paths]
        current = self._read_user_env("Path") or ""
        parts = _path_list(current)
        lower = {p.lower() for p in parts}
        for entry in reversed(path_entries):
            if entry.lower() not in lower:
                parts.insert(0, entry)
                lower.add(entry.lower())
        self._write_user_env("Path", _join_paths(parts))

        for name, value in spec.vars.items():
            self._write_user_env(name, value)

        self._broadcast_env_change()

    def _revert_windows(self, spec: EnvSpec) -> None:
        path_entries = {_norm_path(p).lower() for p in spec.paths}
        current = self._read_user_env("Path") or ""
        parts = [p for p in _path_list(current) if p.lower() not in path_entries]
        self._write_user_env("Path", _join_paths(parts))

        for name in spec.vars:
            self._delete_user_env(name)

        self._broadcast_env_change()

    def _check_windows(self, spec: EnvSpec) -> dict[str, bool]:
        result: dict[str, bool] = {}
        current = self._read_user_env("Path") or ""
        lower_parts = {p.lower() for p in _path_list(current)}
        for p in spec.paths:
            key = f"PATH:{_norm_path(p)}"
            result[key] = _norm_path(p).lower() in lower_parts
        for name, value in spec.vars.items():
            actual = self._read_user_env(name)
            result[name] = actual == value
        return result

    @staticmethod
    def _user_env_key():
        import winreg

        return winreg.OpenKey(
            winreg.HKEY_CURRENT_USER,
            r"Environment",
            0,
            winreg.KEY_READ | winreg.KEY_SET_VALUE,
        )

    def _read_user_env(self, name: str) -> str | None:
        import winreg

        try:
            with self._user_env_key() as key:
                value, _ = winreg.QueryValueEx(key, name)
                return str(value)
        except FileNotFoundError:
            return None
        except OSError:
            return None

    def _write_user_env(self, name: str, value: str) -> None:
        import winreg

        with self._user_env_key() as key:
            kind = winreg.REG_EXPAND_SZ if name.lower() == "path" else winreg.REG_SZ
            winreg.SetValueEx(key, name, 0, kind, value)

    def _delete_user_env(self, name: str) -> None:
        import winreg

        try:
            with self._user_env_key() as key:
                winreg.DeleteValue(key, name)
        except FileNotFoundError:
            pass
        except OSError:
            pass

    @staticmethod
    def _broadcast_env_change() -> None:
        """Notify Windows that environment variables changed."""
        try:
            import ctypes

            HWND_BROADCAST = 0xFFFF
            WM_SETTINGCHANGE = 0x001A
            SMTO_ABORTIFHUNG = 0x0002
            result = ctypes.c_long()
            ctypes.windll.user32.SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                "Environment",
                SMTO_ABORTIFHUNG,
                5000,
                ctypes.byref(result),
            )
        except (AttributeError, OSError, ValueError):
            pass

    # --- macOS / Linux (managed snippet in ~/.devkit/env.sh) ---

    def _env_sh_path(self) -> Path:
        return Path.home() / ".devkit" / "env.sh"

    def _apply_unix(self, spec: EnvSpec) -> None:
        env_sh = self._env_sh_path()
        env_sh.parent.mkdir(parents=True, exist_ok=True)
        block = self._build_unix_block(spec)
        existing = env_sh.read_text(encoding="utf-8") if env_sh.exists() else ""
        updated = self._replace_or_append_block(existing, block)
        env_sh.write_text(updated, encoding="utf-8")
        self._ensure_profile_source()

    def _ensure_profile_source(self) -> Path:
        """Ensure the primary shell profile sources ``~/.devkit/env.sh``."""
        profile = primary_shell_profile()
        profile.parent.mkdir(parents=True, exist_ok=True)
        block = f"{_PROFILE_BEGIN}\n{_SOURCE_LINE}\n{_PROFILE_END}\n"
        existing = profile.read_text(encoding="utf-8") if profile.exists() else ""
        if _SOURCE_LINE in existing or "$HOME/.devkit/env.sh" in existing:
            return profile
        sep = "\n" if existing and not existing.endswith("\n") else ""
        profile.write_text(f"{existing}{sep}{block}", encoding="utf-8")
        return profile

    def _revert_unix(self, spec: EnvSpec) -> None:
        env_sh = self._env_sh_path()
        if not env_sh.exists():
            return
        text = env_sh.read_text(encoding="utf-8")
        block = self._extract_block(text)
        if block is None:
            return
        lines = block.splitlines()
        path_targets = {_norm_path(p) for p in spec.paths}
        var_names = set(spec.vars)
        kept: list[str] = []
        for line in lines:
            skip = False
            for p in path_targets:
                if f'PATH="{p}:' in line or f"PATH={p}:" in line:
                    skip = True
                    break
            m = re.match(r'^export\s+([A-Za-z_][A-Za-z0-9_]*)=', line)
            if m and m.group(1) in var_names:
                skip = True
            if not skip:
                kept.append(line)
        new_block = "\n".join(kept).strip()
        if new_block in {_ENV_SH_BEGIN, f"{_ENV_SH_BEGIN}\n{_ENV_SH_END}", ""}:
            updated = self._remove_block(text)
        else:
            if not new_block.startswith(_ENV_SH_BEGIN):
                new_block = f"{_ENV_SH_BEGIN}\n{new_block}\n{_ENV_SH_END}"
            if not new_block.endswith(_ENV_SH_END):
                new_block = f"{new_block.rstrip()}\n{_ENV_SH_END}"
            updated = self._replace_or_append_block(text, new_block)
        env_sh.write_text(updated, encoding="utf-8")

    def _check_unix(self, spec: EnvSpec) -> dict[str, bool]:
        result: dict[str, bool] = {}
        env_sh = self._env_sh_path()
        text = env_sh.read_text(encoding="utf-8") if env_sh.exists() else ""
        for p in spec.paths:
            key = f"PATH:{_norm_path(p)}"
            result[key] = _norm_path(p) in text
        for name, value in spec.vars.items():
            pattern = re.compile(
                rf'^export\s+{re.escape(name)}="{re.escape(value)}"\s*$',
                re.MULTILINE,
            )
            result[name] = bool(pattern.search(text))
        return result

    def _build_unix_block(self, spec: EnvSpec) -> str:
        lines = [_ENV_SH_BEGIN]
        for p in spec.paths:
            np = _norm_path(p)
            lines.append(f'export PATH="{np}:$PATH"')
        for name, value in spec.vars.items():
            safe = value.replace('"', '\\"')
            lines.append(f'export {name}="{safe}"')
        lines.append(_ENV_SH_END)
        return "\n".join(lines) + "\n"

    def _replace_or_append_block(self, text: str, block: str) -> str:
        existing_block = self._extract_block(text)
        if existing_block is None:
            sep = "\n" if text and not text.endswith("\n") else ""
            return f"{text}{sep}{block}"

        old_lines = [
            ln
            for ln in existing_block.splitlines()
            if ln not in {_ENV_SH_BEGIN, _ENV_SH_END} and ln.strip()
        ]
        new_inner = [
            ln
            for ln in block.splitlines()
            if ln not in {_ENV_SH_BEGIN, _ENV_SH_END} and ln.strip()
        ]
        merged: list[str] = []
        seen: set[str] = set()
        for ln in new_inner + old_lines:
            if ln in seen:
                continue
            seen.add(ln)
            merged.append(ln)
        new_block = "\n".join([_ENV_SH_BEGIN, *merged, _ENV_SH_END]) + "\n"
        return self._replace_block(text, new_block)

    def _extract_block(self, text: str) -> str | None:
        start = text.find(_ENV_SH_BEGIN)
        end = text.find(_ENV_SH_END)
        if start < 0 or end < 0 or end < start:
            return None
        return text[start : end + len(_ENV_SH_END)]

    def _replace_block(self, text: str, new_block: str) -> str:
        start = text.find(_ENV_SH_BEGIN)
        end = text.find(_ENV_SH_END)
        if start < 0 or end < 0:
            return text + new_block
        return text[:start] + new_block + text[end + len(_ENV_SH_END) :]

    def _remove_block(self, text: str) -> str:
        start = text.find(_ENV_SH_BEGIN)
        end = text.find(_ENV_SH_END)
        if start < 0 or end < 0:
            return text
        before = text[:start].rstrip("\n")
        after = text[end + len(_ENV_SH_END) :].lstrip("\n")
        if before and after:
            return before + "\n" + after
        return before + after
