//! Shared download progress UI for large SDK installs.

use indicatif::{ProgressBar, ProgressStyle};

/// Throttled single-line progress bar for `download::download_file`.
pub struct DownloadProgress {
    bar: ProgressBar,
    finished: bool,
}

impl DownloadProgress {
    pub fn new(label: &str) -> Self {
        let bar = ProgressBar::new(0);
        bar.set_style(
            ProgressStyle::with_template(
                "{prefix}  [{bar:28}]  {percent:>3}%  {bytes}/{total_bytes}",
            )
            .unwrap()
            .progress_chars("#>-"),
        );
        bar.set_prefix(label.to_string());
        Self {
            bar,
            finished: false,
        }
    }

    /// Report progress; call from a `download_file` progress callback.
    pub fn update(&mut self, downloaded: u64, total: Option<u64>) {
        if self.finished {
            return;
        }
        if let Some(total) = total {
            if self.bar.length() != Some(total) {
                self.bar.set_length(total);
            }
            self.bar.set_position(downloaded);
            if downloaded >= total {
                self.done();
            }
        } else {
            self.bar.set_position(downloaded);
        }
    }

    /// End the progress line.
    pub fn done(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        self.bar.finish_and_clear();
        println!(
            "{}  done ({} bytes)",
            self.bar.prefix(),
            self.bar.position()
        );
    }
}

pub fn download_progress(label: &str) -> DownloadProgress {
    DownloadProgress::new(label)
}
