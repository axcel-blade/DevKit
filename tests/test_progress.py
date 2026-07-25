"""Tests for shared download progress UI."""

from io import StringIO

from devkit.progress import DownloadProgress


def test_progress_bar_with_total(monkeypatch):
    buf = StringIO()
    progress = DownloadProgress("Downloading Test", bar_width=10)
    progress._stream = buf
    monkeypatch.setattr("devkit.progress.time.monotonic", lambda: 1.0)

    progress(50 * 1024 * 1024, 100 * 1024 * 1024)
    out = buf.getvalue()
    assert "Downloading Test" in out
    assert "50%" in out
    assert "[" in out and "]" in out

    progress(100 * 1024 * 1024, 100 * 1024 * 1024)
    assert progress._finished
    assert buf.getvalue().endswith("\n")


def test_progress_done_is_idempotent():
    buf = StringIO()
    progress = DownloadProgress("X")
    progress._stream = buf
    progress.done()
    progress.done()
    assert buf.getvalue().count("\n") == 1


def test_progress_without_total(monkeypatch):
    buf = StringIO()
    progress = DownloadProgress("DL")
    progress._stream = buf
    monkeypatch.setattr("devkit.progress.time.monotonic", lambda: 10.0)
    progress(5 * 1024 * 1024, None)
    assert "5.0 MiB" in buf.getvalue()
