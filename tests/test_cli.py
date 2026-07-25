"""CLI smoke tests (no real user env mutations)."""

from pathlib import Path

from devkit.cli import main
from devkit.plugin import EnvSpec


def test_cli_list(capsys):
    assert main(["list"]) == 0
    out = capsys.readouterr().out
    assert "hello" in out


def test_cli_doctor(monkeypatch, tmp_path: Path, capsys):
    monkeypatch.setenv("DEVKIT_HOME", str(tmp_path / "dev"))
    assert main(["doctor"]) == 0
    out = capsys.readouterr().out
    assert "Dev root" in out
    assert "hello" in out


def test_cli_install_uninstall(monkeypatch, tmp_path: Path, capsys):
    monkeypatch.setenv("DEVKIT_HOME", str(tmp_path / "dev"))

    applied: list[EnvSpec] = []
    reverted: list[EnvSpec] = []

    class FakeEnv:
        def apply(self, spec: EnvSpec) -> None:
            applied.append(spec)

        def revert(self, spec: EnvSpec) -> None:
            reverted.append(spec)

        def check(self, spec: EnvSpec) -> dict[str, bool]:
            return {k: True for k in list(spec.vars) + [f"PATH:{p}" for p in spec.paths]}

    monkeypatch.setattr("devkit.cli.EnvManager", FakeEnv)

    assert main(["install", "hello"]) == 0
    assert applied
    install_dir = tmp_path / "dev" / "hello"
    assert (install_dir / ".devkit-hello").is_file()
    assert (install_dir / "bin" / "hello.txt").is_file()

    assert main(["status", "hello"]) == 0
    out = capsys.readouterr().out
    assert "installed" in out

    assert main(["uninstall", "hello"]) == 0
    assert reverted
    assert not install_dir.exists()


def test_cli_unknown_plugin():
    assert main(["install", "does-not-exist"]) == 1
