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


def test_cli_install_accepts_version_channel(monkeypatch, tmp_path: Path):
    monkeypatch.setenv("DEVKIT_HOME", str(tmp_path / "dev"))

    class FakeEnv:
        def apply(self, spec: EnvSpec) -> None:
            return None

        def revert(self, spec: EnvSpec) -> None:
            return None

        def check(self, spec: EnvSpec) -> dict[str, bool]:
            return {}

    monkeypatch.setattr("devkit.cli.EnvManager", FakeEnv)

    captured: dict = {}

    def fake_install(self, ctx):
        from devkit.platform import is_windows
        from devkit.plugin import InstallResult

        captured["version"] = ctx.version
        captured["channel"] = ctx.channel
        ctx.install_dir.mkdir(parents=True, exist_ok=True)
        (ctx.install_dir / "bin").mkdir(exist_ok=True)
        bin_name = "flutter.bat" if is_windows() else "flutter"
        (ctx.install_dir / "bin" / bin_name).write_text("x", encoding="utf-8")
        return InstallResult(ctx.install_dir, message="ok")

    monkeypatch.setattr(
        "devkit.plugins.flutter.FlutterPlugin.install",
        fake_install,
    )
    assert main(["install", "flutter", "--channel", "beta", "--version", "3.24.0"]) == 0
    assert captured == {"version": "3.24.0", "channel": "beta"}
