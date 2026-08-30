//! Download and extract archives into the machine `dev` folder.
//!
//! Supports ZIP (Windows/macOS SDKs) and tar.gz / tar.xz (typical Linux SDKs).

use crate::paths::{cache_dir, ensure_home};
use crate::platform::pick_for_os;
use anyhow::{bail, Context, Result};
use std::fs;
use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub type ProgressCallback<'a> = &'a mut dyn FnMut(u64, Option<u64>);

/// Well-known, highly-available hosts to probe for connectivity. Several are
/// tried (rather than just one) so a single blocked or momentarily-slow host
/// doesn't produce a false "offline" result.
const CONNECTIVITY_PROBES: &[(&str, u16)] =
    &[("github.com", 443), ("1.1.1.1", 443), ("8.8.8.8", 443)];

const CONNECTIVITY_TIMEOUT: Duration = Duration::from_secs(2);

/// Return whether the network appears reachable, without downloading
/// anything. Used to fail fast with a clear message before a plugin's real
/// download attempt fails deep in the stack with a cryptic I/O error.
pub fn has_internet_access() -> bool {
    for (host, port) in CONNECTIVITY_PROBES {
        let Ok(mut addrs) = (*host, *port).to_socket_addrs() else {
            continue;
        };
        if let Some(addr) = addrs.next() {
            if TcpStream::connect_timeout(&addr, CONNECTIVITY_TIMEOUT).is_ok() {
                return true;
            }
        }
    }
    false
}

/// Check connectivity and return a friendly error if the network is
/// unreachable. Call this before any function in this module that hits the
/// network, so plugin installs fail fast with an actionable message instead
/// of a raw connection-refused/timeout error from deep inside a download.
pub fn ensure_internet_access() -> Result<()> {
    if has_internet_access() {
        return Ok(());
    }
    bail!(
        "No internet connection detected. Installing SDKs downloads files over \
         the network — check your connection and try again."
    )
}

/// Download a JSON document and return the parsed value (object or array).
pub fn download_json_value(url: &str) -> Result<serde_json::Value> {
    ensure_internet_access()?;
    let resp = ureq::get(url)
        .set("User-Agent", "DevKit")
        .call()
        .with_context(|| format!("Failed to download JSON {url}"))?;
    let value: serde_json::Value = resp
        .into_json()
        .with_context(|| format!("Failed to parse JSON from {url}"))?;
    Ok(value)
}

/// Download a JSON document and return it as an object map.
pub fn download_json(url: &str) -> Result<serde_json::Map<String, serde_json::Value>> {
    match download_json_value(url)? {
        serde_json::Value::Object(map) => Ok(map),
        _ => bail!("Expected JSON object from {url}"),
    }
}

fn filename_from_response(url: &str, resp: &ureq::Response, explicit: Option<&str>) -> String {
    if let Some(name) = explicit {
        return name.to_string();
    }
    if let Some(cd) = resp.header("Content-Disposition") {
        if let Some(idx) = cd.find("filename=") {
            let part = cd[idx + "filename=".len()..]
                .trim()
                .trim_matches(|c| c == '"' || c == '\'');
            if !part.is_empty() {
                if let Some(name) = Path::new(part).file_name() {
                    return name.to_string_lossy().to_string();
                }
            }
        }
    }
    let final_url = resp.get_url();
    if let Some(name) = filename_from_url(final_url) {
        return name;
    }
    filename_from_url(url).unwrap_or_else(|| "download.bin".to_string())
}

fn filename_from_url(url: &str) -> Option<String> {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let name = Path::new(path).file_name()?.to_string_lossy().to_string();
    if name.is_empty() || name == "latest" {
        None
    } else {
        Some(name)
    }
}

/// Download `url` into the DevKit cache (or `dest`). Returns the downloaded file path.
pub fn download_file(
    url: &str,
    dest: Option<&Path>,
    filename: Option<&str>,
    mut progress: Option<ProgressCallback>,
) -> Result<PathBuf> {
    ensure_home()?;
    ensure_internet_access()?;
    let resp = ureq::get(url)
        .set("User-Agent", "DevKit")
        .call()
        .with_context(|| format!("Failed to download {url}"))?;

    let dest_path = match dest {
        Some(d) => d.to_path_buf(),
        None => cache_dir().join(filename_from_response(url, &resp, filename)),
    };
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let total: Option<u64> = resp
        .header("Content-Length")
        .and_then(|v| v.parse::<u64>().ok());

    let mut reader = resp.into_reader();
    let mut out = fs::File::create(&dest_path)?;
    let mut buf = [0u8; 256 * 1024];
    let mut downloaded: u64 = 0;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        std::io::Write::write_all(&mut out, &buf[..n])?;
        downloaded += n as u64;
        if let Some(cb) = progress.as_deref_mut() {
            cb(downloaded, total);
        }
    }

    Ok(dest_path)
}

fn unique_top_level<'a, I: Iterator<Item = &'a str>>(names: I) -> Option<String> {
    let mut tops: Vec<String> = Vec::new();
    for name in names {
        let normalized = name.replace('\\', "/");
        let normalized = normalized.trim_matches('/');
        if normalized.is_empty() {
            continue;
        }
        let top = normalized.split('/').next().unwrap().to_string();
        if !tops.contains(&top) {
            tops.push(top);
        }
        if tops.len() > 1 {
            return None;
        }
    }
    tops.into_iter().next()
}

fn copy_tree_contents(source: &Path, dest: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn prepare_dest(dest: &Path) -> Result<()> {
    if dest.exists() {
        fs::remove_dir_all(dest)?;
    }
    fs::create_dir_all(dest)?;
    Ok(())
}

/// Extract a ZIP into `dest`.
pub fn extract_zip(zip_path: &Path, dest: &Path, strip_top_level: bool) -> Result<PathBuf> {
    prepare_dest(dest)?;
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    let top = if strip_top_level {
        unique_top_level(names.iter().map(|s| s.as_str()))
    } else {
        None
    };

    match top {
        None => {
            archive.extract(dest)?;
        }
        Some(top) => {
            let tmp = tempfile::tempdir()?;
            archive.extract(tmp.path())?;
            let source = tmp.path().join(&top);
            if source.is_dir() {
                copy_tree_contents(&source, dest)?;
            } else {
                copy_tree_contents(tmp.path(), dest)?;
            }
        }
    }
    Ok(dest.to_path_buf())
}

fn open_tar_archive(tar_path: &Path) -> Result<Box<dyn Read>> {
    let name = tar_path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let file = fs::File::open(tar_path)?;
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Ok(Box::new(flate2::read::GzDecoder::new(file)))
    } else if name.ends_with(".tar.xz") {
        Ok(Box::new(xz2::read::XzDecoder::new(file)))
    } else if name.ends_with(".tar.bz2") {
        bail!(
            "bzip2 tar archives are not supported: {}",
            tar_path.display()
        );
    } else {
        // Sniff magic bytes for a plain .tar or unrecognized extension.
        let mut probe = fs::File::open(tar_path)?;
        let mut magic = [0u8; 6];
        let n = probe.read(&mut magic).unwrap_or(0);
        let file = fs::File::open(tar_path)?;
        if n >= 2 && &magic[..2] == b"\x1f\x8b" {
            Ok(Box::new(flate2::read::GzDecoder::new(file)))
        } else if n >= 6 && &magic[..6] == b"\xfd7zXZ\x00" {
            Ok(Box::new(xz2::read::XzDecoder::new(file)))
        } else {
            Ok(Box::new(file))
        }
    }
}

/// Extract a `.tar`, `.tar.gz`, `.tgz`, or `.tar.xz` into `dest`.
pub fn extract_tar(tar_path: &Path, dest: &Path, strip_top_level: bool) -> Result<PathBuf> {
    prepare_dest(dest)?;
    let tmp = tempfile::tempdir()?;

    {
        let reader = open_tar_archive(tar_path)?;
        let mut archive = tar::Archive::new(reader);
        archive.unpack(tmp.path())?;
    }

    let top = if strip_top_level {
        let entries: Vec<String> = fs::read_dir(tmp.path())?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        if entries.len() == 1 {
            let only = &entries[0];
            if tmp.path().join(only).is_dir() {
                Some(only.clone())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    match top {
        Some(top) => copy_tree_contents(&tmp.path().join(top), dest)?,
        None => copy_tree_contents(tmp.path(), dest)?,
    }
    Ok(dest.to_path_buf())
}

/// Return `zip`, `tar`, or raise if unknown.
pub fn detect_archive_format(path: &Path) -> Result<&'static str> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if name.ends_with(".zip") {
        return Ok("zip");
    }
    if name.ends_with(".tar.gz")
        || name.ends_with(".tgz")
        || name.ends_with(".tar.xz")
        || name.ends_with(".tar.bz2")
        || name.ends_with(".tar")
    {
        return Ok("tar");
    }
    let mut file = fs::File::open(path)?;
    let mut magic = [0u8; 6];
    let n = file.read(&mut magic)?;
    if n >= 2 && &magic[..2] == b"PK" {
        return Ok("zip");
    }
    if (n >= 2 && &magic[..2] == b"\x1f\x8b")
        || (n >= 5 && &magic[..5] == b"ustar")
        || (n >= 6 && &magic[..6] == b"\xfd7zXZ\x00")
    {
        return Ok("tar");
    }
    bail!("Unsupported archive format: {}", path.display())
}

/// Extract a ZIP or tar archive into `dest`.
pub fn extract_archive(archive_path: &Path, dest: &Path, strip_top_level: bool) -> Result<PathBuf> {
    match detect_archive_format(archive_path)? {
        "zip" => extract_zip(archive_path, dest, strip_top_level),
        _ => extract_tar(archive_path, dest, strip_top_level),
    }
}

/// Download an archive and extract it into `dest` (all platforms).
pub fn install_archive_from_url(
    url: &str,
    dest: &Path,
    strip_top_level: bool,
    filename: Option<&str>,
    progress: Option<ProgressCallback>,
) -> Result<PathBuf> {
    let archive = download_file(url, None, filename, progress)?;
    extract_archive(&archive, dest, strip_top_level)
}

/// Download the URL for the current OS and extract it.
pub fn install_archive_from_urls(
    urls: &[(&str, &str)],
    dest: &Path,
    strip_top_level: bool,
    progress: Option<ProgressCallback>,
) -> Result<PathBuf> {
    let url = pick_for_os(urls, None)?;
    install_archive_from_url(&url, dest, strip_top_level, None, progress)
}

/// Extract an existing ZIP file into `dest`.
pub fn install_zip_file(zip_path: &Path, dest: &Path, strip_top_level: bool) -> Result<PathBuf> {
    extract_zip(zip_path, dest, strip_top_level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_internet_access_does_not_panic() {
        // Result depends on the test runner's network access, so this only
        // exercises the code path rather than asserting a specific outcome.
        let _ = has_internet_access();
    }

    #[test]
    fn ensure_internet_access_error_message_is_actionable() {
        // Can't force an offline result without mocking the network, but a
        // failing check must produce a clear, non-empty message.
        if !has_internet_access() {
            let err = ensure_internet_access().unwrap_err();
            assert!(err.to_string().contains("internet"));
        }
    }

    #[test]
    fn unique_top_level_detects_common_root() {
        let names = vec!["sdk/bin/a", "sdk/lib/b"];
        assert_eq!(unique_top_level(names.into_iter()), Some("sdk".to_string()));
    }

    #[test]
    fn unique_top_level_none_when_mixed() {
        let names = vec!["a/x", "b/y"];
        assert_eq!(unique_top_level(names.into_iter()), None);
    }

    #[test]
    fn detect_archive_format_by_extension() {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("thing.zip");
        fs::write(&zip_path, b"PK\x03\x04").unwrap();
        assert_eq!(detect_archive_format(&zip_path).unwrap(), "zip");

        let tar_path = dir.path().join("thing.tar.gz");
        fs::write(&tar_path, b"\x1f\x8b\x08\x00\x00\x00").unwrap();
        assert_eq!(detect_archive_format(&tar_path).unwrap(), "tar");
    }

    fn make_zip(path: &Path, entries: &[(&str, &str)]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let file = fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        for (name, content) in entries {
            zip.start_file(*name, options).unwrap();
            std::io::Write::write_all(&mut zip, content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }

    fn make_tar_gz(path: &Path, entries: &[(&str, &str)]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let file = fs::File::create(path).unwrap();
        let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(enc);
        for (name, content) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, name, content.as_bytes())
                .unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap();
    }

    #[test]
    fn extract_zip_strips_top_level() {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("sdk.zip");
        make_zip(
            &zip_path,
            &[
                ("flutter/bin/flutter", "#!/bin/sh\n"),
                ("flutter/README.md", "Flutter\n"),
            ],
        );
        let dest = dir.path().join("out");
        extract_zip(&zip_path, &dest, true).unwrap();
        assert!(dest.join("bin").join("flutter").is_file());
        assert!(dest.join("README.md").is_file());
        assert!(!dest.join("flutter").exists());
    }

    #[test]
    fn extract_zip_keeps_top_level_when_mixed() {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("sdk.zip");
        make_zip(
            &zip_path,
            &[("flutter/bin/flutter", "x"), ("other/file.txt", "y")],
        );
        let dest = dir.path().join("out");
        extract_zip(&zip_path, &dest, true).unwrap();
        assert!(dest.join("flutter").join("bin").join("flutter").is_file());
        assert!(dest.join("other").join("file.txt").is_file());
    }

    #[test]
    fn extract_tar_gz_strips_top_level() {
        let dir = tempfile::tempdir().unwrap();
        let tar_path = dir.path().join("sdk.tar.gz");
        make_tar_gz(
            &tar_path,
            &[
                ("flutter/bin/flutter", "#!/bin/sh\n"),
                ("flutter/README.md", "Flutter\n"),
            ],
        );
        let dest = dir.path().join("out");
        extract_tar(&tar_path, &dest, true).unwrap();
        assert!(dest.join("bin").join("flutter").is_file());
        assert!(dest.join("README.md").is_file());
    }

    #[test]
    fn extract_archive_auto_detects_zip_and_tar() {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("a.zip");
        make_zip(&zip_path, &[("sdk/bin/x", "1")]);
        let tar_path = dir.path().join("a.tar.gz");
        make_tar_gz(&tar_path, &[("sdk/bin/y", "2")]);

        extract_archive(&zip_path, &dir.path().join("z"), true).unwrap();
        extract_archive(&tar_path, &dir.path().join("t"), true).unwrap();
        assert!(dir.path().join("z").join("bin").join("x").is_file());
        assert!(dir.path().join("t").join("bin").join("y").is_file());
    }
}
