"""Tests for EnvManager (Unix backend + helpers)."""

from pathlib import Path

from devkit.env import EnvManager, _norm_path
from devkit.plugin import EnvSpec


def _unix_env(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr("devkit.env.is_windows", lambda: False)
    monkeypatch.setattr(Path, "home", classmethod(lambda cls: tmp_path))
    monkeypatch.setattr(
        "devkit.env.primary_shell_profile",
        lambda: tmp_path / ".bashrc",
    )


def test_unix_apply_and_check(monkeypatch, tmp_path: Path):
    _unix_env(monkeypatch, tmp_path)

    install = tmp_path / "sdk" / "bin"
    install.mkdir(parents=True)
    spec = EnvSpec(paths=[install], vars={"DEVKIT_TEST": "yes"})

    mgr = EnvManager()
    mgr.apply(spec)

    env_sh = tmp_path / ".devkit" / "env.sh"
    assert env_sh.is_file()
    text = env_sh.read_text(encoding="utf-8")
    assert _norm_path(install) in text
    assert 'export DEVKIT_TEST="yes"' in text

    profile = (tmp_path / ".bashrc").read_text(encoding="utf-8")
    assert ".devkit/env.sh" in profile

    checks = mgr.check(spec)
    assert all(checks.values())

    mgr.revert(spec)
    text_after = env_sh.read_text(encoding="utf-8")
    assert _norm_path(install) not in text_after
    assert "DEVKIT_TEST" not in text_after


def test_unix_merge_multiple_applies(monkeypatch, tmp_path: Path):
    _unix_env(monkeypatch, tmp_path)

    a = tmp_path / "a"
    b = tmp_path / "b"
    a.mkdir()
    b.mkdir()
    mgr = EnvManager()
    mgr.apply(EnvSpec(paths=[a], vars={"A": "1"}))
    mgr.apply(EnvSpec(paths=[b], vars={"B": "2"}))

    text = (tmp_path / ".devkit" / "env.sh").read_text(encoding="utf-8")
    assert _norm_path(a) in text
    assert _norm_path(b) in text
    assert 'export A="1"' in text
    assert 'export B="2"' in text
