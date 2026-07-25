"""Shared download progress UI for large SDK installs."""

from __future__ import annotations

import sys
import time
from collections.abc import Callable

ProgressCallback = Callable[[int, int | None], None]


def _format_mib(n: int) -> str:
    return f"{n / (1024 * 1024):.1f}"


class DownloadProgress:
    """Throttle and render a single-line progress bar on stderr/stdout.

    Large SDKs (Flutter, JDK, Android tools) spam the terminal if every chunk
    prints a line; we redraw in place and limit updates to ~12 Hz or percent
    changes.

    Example::

        progress = DownloadProgress("Flutter")
        download_file(url, progress=progress)
        progress.done()
    """

    def __init__(self, label: str = "Downloading", *, bar_width: int = 28) -> None:
        self.label = label
        self.bar_width = bar_width
        self._last_draw = 0.0
        self._last_pct = -1
        self._finished = False
        self._stream = sys.stdout

    def __call__(self, downloaded: int, total: int | None) -> None:
        if self._finished:
            return
        now = time.monotonic()
        if total and total > 0:
            pct = min(100, downloaded * 100 // total)
            # Redraw on percent change, completion, or every ~80ms.
            if (
                pct != self._last_pct
                or downloaded >= total
                or now - self._last_draw >= 0.08
            ):
                self._draw(downloaded, total, pct)
                self._last_pct = pct
                self._last_draw = now
            if downloaded >= total:
                self.done()
        else:
            if now - self._last_draw >= 0.15:
                self._draw(downloaded, None, None)
                self._last_draw = now

    def _draw(self, downloaded: int, total: int | None, pct: int | None) -> None:
        if total and total > 0 and pct is not None:
            filled = max(0, min(self.bar_width, (pct * self.bar_width) // 100))
            bar = "#" * filled + "-" * (self.bar_width - filled)
            line = (
                f"\r{self.label}  [{bar}]  {pct:3d}%  "
                f"{_format_mib(downloaded)}/{_format_mib(total)} MiB"
            )
        else:
            line = f"\r{self.label}  {_format_mib(downloaded)} MiB"
        # Pad to clear leftovers from shorter previous lines.
        self._stream.write(line.ljust(80))
        self._stream.flush()

    def done(self) -> None:
        """End the progress line (print newline once)."""
        if self._finished:
            return
        self._finished = True
        self._stream.write("\n")
        self._stream.flush()


def download_progress(label: str = "Downloading") -> DownloadProgress:
    """Create a progress callback for ``download_file`` / ``install_archive_*``."""
    return DownloadProgress(label)
