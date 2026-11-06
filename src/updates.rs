//! The update of the program from the releases of GitHub: it asks for the latest release, tells what is installed and what is new (with the notes of the version),
//! downloads the package of this system that the release carries, checks its SHA-256 against the `.sha256` file that goes with it, and puts the new program in
//! the place of the running one, keeping the old one so that the change can be undone. Nothing is installed without the person asking for it, nothing is sent
//! about the person or the machine (the only requests are the one for the release and the ones for the package and its checksum), and a package whose
//! checksum does not match is never installed.
//!
//! The names of the packages are the ones the release workflow publishes: `bungard-gui-<version>-<system>.zip` (Windows) or `.tar.gz` (Linux and macOS), each with
//! a `<name>.sha256` file, `<system>` being `windows-x86_64`, `windows-x86`, `linux-x86_64`, `macos-arm64` or `macos-x86_64`.
//!
//! A running program cannot be overwritten on Windows, but it can be renamed: the running file becomes `<name>.previous` and the new one takes its name. On the
//! other systems the old file is copied to `<name>.previous` and the new one replaces it in one atomic step. The program is restarted afterwards to use the new one.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sha2::{Digest, Sha256};

/// The name of the program file without the suffix of the system.
pub const PROGRAM: &str = "bungard-gui";
/// The most that the answer of the API, a checksum file or a package may weigh: a limit that keeps a wrong address from filling the disk.
const MAX_JSON_BYTES: u64 = 4 * 1024 * 1024;
const MAX_SUM_BYTES: u64 = 4 * 1024;
const MAX_PACKAGE_BYTES: u64 = 400 * 1024 * 1024;
const MAX_PROGRAM_BYTES: u64 = 600 * 1024 * 1024;

/// What is asked of the network.
#[derive(Debug, Clone)]
pub struct Config {
    /// The API address that answers with the latest release (`releases/latest` of the repository).
    pub api_url: String,
    /// Seconds a connection may wait for the other end.
    pub timeout_s: u64,
}

/// Why a step did not end well; the words are in English because they are the technical detail that is shown under the translated heading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    Network(String),
    Answer(String),
    NoPackage,
    Checksum(String),
    Archive(String),
    Disk(String),
    Refused(String),
    Cancelled,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Network(s) => write!(f, "network: {s}"),
            Failure::Answer(s) => write!(f, "answer of the server: {s}"),
            Failure::NoPackage => f.write_str("the release has no package for this system"),
            Failure::Checksum(s) => write!(f, "checksum: {s}"),
            Failure::Archive(s) => write!(f, "package: {s}"),
            Failure::Disk(s) => write!(f, "disk: {s}"),
            Failure::Refused(s) => write!(f, "refused: {s}"),
            Failure::Cancelled => f.write_str("cancelled"),
        }
    }
}

impl std::error::Error for Failure {}

type Done<T> = Result<T, Failure>;

// ---- versions and names

/// The numbers of a version (`0.2.0`, `v0.8.0.1`): one to four parts.
pub fn version_parts(v: &str) -> Option<Vec<u64>> {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let parts: Vec<u64> = v
        .split('.')
        .map(|p| p.parse::<u64>().ok())
        .collect::<Option<_>>()?;
    (1..=4).contains(&parts.len()).then_some(parts)
}

/// Whether `new` is a later version than `current` (a part that is missing counts as zero). A version that cannot be read is never later.
pub fn is_newer(new: &str, current: &str) -> bool {
    let (Some(mut a), Some(mut b)) = (version_parts(new), version_parts(current)) else {
        return false;
    };
    let n = a.len().max(b.len());
    a.resize(n, 0);
    b.resize(n, 0);
    a > b
}

/// The word of this system in the names of the packages, `None` when the release has no package for it.
pub fn platform_label() -> Option<&'static str> {
    if cfg!(all(target_os = "windows", target_pointer_width = "64")) {
        Some("windows-x86_64")
    } else if cfg!(all(target_os = "windows", target_pointer_width = "32")) {
        Some("windows-x86")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some("linux-x86_64")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some("macos-arm64")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Some("macos-x86_64")
    } else {
        None
    }
}

/// The name of the package of a system in a version, as the release workflow makes it.
pub fn package_name(version: &str, label: &str) -> String {
    let ext = if label.starts_with("windows") {
        "zip"
    } else {
        "tar.gz"
    };
    format!("{PROGRAM}-{version}-{label}.{ext}")
}

/// The name of the program file inside a package.
fn program_file(label: &str) -> String {
    if label.starts_with("windows") {
        format!("{PROGRAM}.exe")
    } else {
        PROGRAM.to_string()
    }
}

/// The notes of a release as plain lines: the marks of the text (bold, code, links) are taken off and the words between them stay, names in quotes included.
pub fn plain_notes(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in body.lines() {
        let mut line = raw.trim().trim_start_matches('#').trim().to_string();
        let bullet = ["- ", "* ", "+ "].iter().any(|b| line.starts_with(b));
        if bullet {
            line = line[2..].trim().to_string();
        }
        // [text](address) -> text
        while let Some(open) = line.find("](") {
            let Some(start) = line[..open].rfind('[') else {
                break;
            };
            let Some(close) = line[open..].find(')') else {
                break;
            };
            let text = line[start + 1..open].to_string();
            line.replace_range(start..open + close + 1, &text);
        }
        let line = line.replace("**", "").replace('`', "");
        let line = line.trim();
        if !line.is_empty() {
            out.push(if bullet {
                format!("- {line}")
            } else {
                line.to_string()
            });
        }
    }
    out
}

// ---- the release

/// The package of a release that is for this system, and the file with its checksum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub url: String,
    pub sum_url: String,
    pub size: u64,
    /// The checksum the API itself gives for the package (`sha256:...`), when it does; it has to agree with the checksum file.
    pub digest: Option<String>,
}

/// The latest release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub tag: String,
    pub version: String,
    pub notes: Vec<String>,
    pub page: String,
    /// `None` when the release carries no package for this system.
    pub package: Option<Package>,
}

/// Whether an address may be used: `https`, or plain `http` only towards this same computer and only when the API address is also of this computer (the tests
/// and a local mirror).
pub fn address_allowed(url: &str, api_url: &str) -> bool {
    fn local(u: &str) -> bool {
        let Some(rest) = u.strip_prefix("http://") else {
            return false;
        };
        let host = rest.split(['/', '?']).next().unwrap_or("");
        let host = if let Some(h) = host.strip_prefix('[') {
            h.split(']').next().unwrap_or("")
        } else {
            host.split(':').next().unwrap_or("")
        };
        matches!(host, "127.0.0.1" | "localhost" | "::1")
    }
    url.starts_with("https://")
        || (local(url) && (local(api_url) || api_url.starts_with("https://")))
}

/// Reads the answer of the API. `label` is the system whose package is wanted. A draft or a pre-release is not a release to update to.
pub fn parse_release(text: &str, label: &str, api_url: &str) -> Done<Release> {
    let v: serde_json::Value =
        serde_json::from_str(text).map_err(|e| Failure::Answer(format!("not JSON: {e}")))?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    if v.get("draft").and_then(|x| x.as_bool()).unwrap_or(false)
        || v.get("prerelease")
            .and_then(|x| x.as_bool())
            .unwrap_or(false)
    {
        return Err(Failure::Answer(
            "the latest release is a draft or a pre-release".into(),
        ));
    }
    let tag = s("tag_name");
    if version_parts(&tag).is_none() {
        return Err(Failure::Answer(format!(
            "the tag \"{tag}\" is not a version"
        )));
    }
    let version = tag.trim_start_matches(['v', 'V']).to_string();
    let wanted = package_name(&version, label);
    let wanted_sum = format!("{wanted}.sha256");
    let assets = v
        .get("assets")
        .and_then(|a| a.as_array())
        .cloned()
        .unwrap_or_default();
    let find = |name: &str| {
        assets
            .iter()
            .find(|a| a.get("name").and_then(|n| n.as_str()) == Some(name))
    };
    let package = match (find(&wanted), find(&wanted_sum)) {
        (Some(p), Some(sum)) => {
            let url_of = |a: &serde_json::Value| {
                a.get("browser_download_url")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let (url, sum_url) = (url_of(p), url_of(sum));
            if !address_allowed(&url, api_url) || !address_allowed(&sum_url, api_url) {
                return Err(Failure::Refused(
                    "the package is not at a secure address".into(),
                ));
            }
            Some(Package {
                name: wanted,
                url,
                sum_url,
                size: p.get("size").and_then(|n| n.as_u64()).unwrap_or(0),
                digest: p
                    .get("digest")
                    .and_then(|d| d.as_str())
                    .and_then(|d| d.strip_prefix("sha256:"))
                    .map(|d| d.to_ascii_lowercase()),
            })
        }
        _ => None,
    };
    Ok(Release {
        notes: plain_notes(&s("body")),
        page: s("html_url"),
        tag,
        version,
        package,
    })
}

// ---- the network

fn agent(cfg: &Config) -> ureq::Agent {
    let t = Duration::from_secs(cfg.timeout_s.clamp(1, 600));
    ureq::AgentBuilder::new()
        .timeout_connect(t)
        .timeout_read(t)
        .timeout_write(t)
        .redirects(5)
        .user_agent(&format!(
            "EH3D-BUNGARD-GUI/{} (update check)",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
}

fn get(cfg: &Config, url: &str) -> Done<ureq::Response> {
    if !address_allowed(url, &cfg.api_url) {
        return Err(Failure::Refused(format!(
            "the address {url} is not allowed"
        )));
    }
    agent(cfg)
        .get(url)
        .set(
            "Accept",
            "application/vnd.github+json, application/octet-stream",
        )
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => Failure::Answer(format!("HTTP {code}")),
            other => Failure::Network(other.to_string()),
        })
}

fn read_text(cfg: &Config, url: &str, max: u64) -> Done<String> {
    let mut buf = Vec::new();
    get(cfg, url)?
        .into_reader()
        .take(max + 1)
        .read_to_end(&mut buf)
        .map_err(|e| Failure::Network(e.to_string()))?;
    if buf.len() as u64 > max {
        return Err(Failure::Answer("the answer is too long".into()));
    }
    String::from_utf8(buf).map_err(|_| Failure::Answer("the answer is not text".into()))
}

/// Asks for the latest release.
pub fn fetch_release(cfg: &Config, label: &str) -> Done<Release> {
    let text = read_text(cfg, &cfg.api_url, MAX_JSON_BYTES)?;
    parse_release(&text, label, &cfg.api_url)
}

/// Downloads the package to `dest`, calling `progress(received, total)` as it comes; the work stops when `cancel` is set.
pub fn download(
    cfg: &Config,
    pkg: &Package,
    dest: &Path,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64, Option<u64>),
) -> Done<()> {
    let resp = get(cfg, &pkg.url)?;
    let total = resp
        .header("Content-Length")
        .and_then(|n| n.parse::<u64>().ok())
        .or((pkg.size > 0).then_some(pkg.size));
    if total.is_some_and(|t| t > MAX_PACKAGE_BYTES) {
        return Err(Failure::Refused("the package is too big".into()));
    }
    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(dest).map_err(|e| Failure::Disk(e.to_string()))?;
    let mut buf = vec![0u8; 64 * 1024];
    let mut got = 0u64;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(Failure::Cancelled);
        }
        let n = reader
            .read(&mut buf)
            .map_err(|e| Failure::Network(e.to_string()))?;
        if n == 0 {
            break;
        }
        got += n as u64;
        if got > MAX_PACKAGE_BYTES {
            return Err(Failure::Refused("the package is too big".into()));
        }
        file.write_all(&buf[..n])
            .map_err(|e| Failure::Disk(e.to_string()))?;
        progress(got, total);
    }
    file.flush().map_err(|e| Failure::Disk(e.to_string()))?;
    if total.is_some_and(|t| t != got) {
        return Err(Failure::Network(format!(
            "the package came to {got} bytes, not the {} it announced",
            total.unwrap_or(0)
        )));
    }
    Ok(())
}

// ---- the checksum

/// The checksum a `.sha256` file holds (`<64 hex digits>  <name>`): the digits, once the name in the file is the package's own.
pub fn parse_sum_file(text: &str, name: &str) -> Done<String> {
    let line = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let mut it = line.split_whitespace();
    let hex = it.next().unwrap_or("").to_ascii_lowercase();
    if hex.len() != 64 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Failure::Checksum(
            "the checksum file is not a SHA-256".into(),
        ));
    }
    if let Some(file) = it.next() {
        if file.trim_start_matches('*') != name {
            return Err(Failure::Checksum(format!(
                "the checksum file is for {file}, not for {name}"
            )));
        }
    }
    Ok(hex)
}

/// The SHA-256 of a file, in lower-case hexadecimal.
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut f = std::fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Checks the downloaded file against the checksum file (and against the one of the API when it gave one).
pub fn verify(file: &Path, pkg: &Package, sum_text: &str) -> Done<()> {
    let expected = parse_sum_file(sum_text, &pkg.name)?;
    if let Some(d) = &pkg.digest {
        if *d != expected {
            return Err(Failure::Checksum(
                "the checksum file and the release do not agree".into(),
            ));
        }
    }
    let got = sha256_file(file).map_err(|e| Failure::Disk(e.to_string()))?;
    if got != expected {
        return Err(Failure::Checksum(format!(
            "the package has SHA-256 {got}, the release says {expected}"
        )));
    }
    Ok(())
}

// ---- the package

/// Writes the program file of a package (the only thing it takes) to `out`. A name with a path in it is read by its last part, so a package cannot place a file elsewhere.
pub fn extract_program(archive: &Path, label: &str, out: &Path) -> Done<()> {
    let want = program_file(label);
    let bad = |e: &dyn std::fmt::Display| Failure::Archive(e.to_string());
    let disk = |e: std::io::Error| Failure::Disk(e.to_string());
    let mut written = false;
    if label.starts_with("windows") {
        let mut z = zip::ZipArchive::new(std::fs::File::open(archive).map_err(disk)?)
            .map_err(|e| bad(&e))?;
        for i in 0..z.len() {
            let entry = z.by_index(i).map_err(|e| bad(&e))?;
            if entry.is_dir() || last_part(entry.name()) != want {
                continue;
            }
            copy_limited(entry, out)?;
            written = true;
            break;
        }
    } else {
        let gz = flate2::read::GzDecoder::new(std::fs::File::open(archive).map_err(disk)?);
        let mut t = tar::Archive::new(gz);
        for entry in t.entries().map_err(|e| bad(&e))? {
            let entry = entry.map_err(|e| bad(&e))?;
            if !entry.header().entry_type().is_file() {
                continue;
            }
            let name = entry
                .path()
                .map_err(|e| bad(&e))?
                .to_string_lossy()
                .into_owned();
            if last_part(&name) != want {
                continue;
            }
            copy_limited(entry, out)?;
            written = true;
            break;
        }
    }
    if !written {
        return Err(Failure::Archive(format!("the package has no {want}")));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(out, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| Failure::Disk(e.to_string()))?;
    }
    Ok(())
}

fn last_part(name: &str) -> &str {
    name.rsplit(['/', '\\']).next().unwrap_or("")
}

fn copy_limited(mut from: impl Read, out: &Path) -> Done<()> {
    let mut file = std::fs::File::create(out).map_err(|e| Failure::Disk(e.to_string()))?;
    let n = std::io::copy(&mut (&mut from).take(MAX_PROGRAM_BYTES + 1), &mut file)
        .map_err(|e| Failure::Archive(e.to_string()))?;
    if n > MAX_PROGRAM_BYTES {
        return Err(Failure::Archive(
            "the program in the package is too big".into(),
        ));
    }
    if n == 0 {
        return Err(Failure::Archive(
            "the program in the package is empty".into(),
        ));
    }
    Ok(())
}

// ---- putting it in place, and putting it back

fn side_path(exe: &Path, suffix: &str) -> PathBuf {
    let mut name = exe.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    exe.with_file_name(name)
}

/// Where the program that was replaced is kept, and the file that says which version it is.
pub fn previous_path(exe: &Path) -> PathBuf {
    side_path(exe, ".previous")
}

fn previous_version_path(exe: &Path) -> PathBuf {
    side_path(exe, ".previous.version")
}

/// The version of the program that is kept for the undo, when there is one.
pub fn previous_version(exe: &Path) -> Option<String> {
    previous_path(exe).is_file().then(|| {
        std::fs::read_to_string(previous_version_path(exe))
            .map(|v| v.trim().to_string())
            .ok()
            .filter(|v| !v.is_empty())
            .unwrap_or_default()
    })
}

/// Where the new program is written before it takes the place of the running one: beside it, so that taking its place is one rename.
pub fn staging_path(exe: &Path) -> PathBuf {
    side_path(exe, ".new")
}

fn remove_if_there(p: &Path) -> Done<()> {
    match std::fs::remove_file(p) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Failure::Disk(format!("{}: {e}", p.display()))),
    }
}

/// Puts the program at `new` (written beside `exe`) in the place of `exe`, keeping the old one as `exe.previous` with its version.
pub fn install(new: &Path, exe: &Path, current_version: &str) -> Done<()> {
    let disk = |p: &Path, e: std::io::Error| Failure::Disk(format!("{}: {e}", p.display()));
    let prev = previous_path(exe);
    remove_if_there(&prev)?;
    remove_if_there(&previous_version_path(exe))?;
    if cfg!(windows) {
        // a running file cannot be replaced, but it can be renamed
        std::fs::rename(exe, &prev).map_err(|e| disk(exe, e))?;
        if let Err(e) = std::fs::rename(new, exe) {
            let _ = std::fs::rename(&prev, exe);
            return Err(disk(new, e));
        }
    } else {
        std::fs::copy(exe, &prev).map_err(|e| disk(&prev, e))?;
        if let Err(e) = std::fs::rename(new, exe) {
            let _ = std::fs::remove_file(&prev);
            return Err(disk(new, e));
        }
    }
    let _ = std::fs::write(previous_version_path(exe), current_version);
    Ok(())
}

/// Puts the kept program back and keeps the one that was running as the previous one (so the undo can be undone). Returns the version that is back.
pub fn undo(exe: &Path, current_version: &str) -> Done<String> {
    let disk = |p: &Path, e: std::io::Error| Failure::Disk(format!("{}: {e}", p.display()));
    let prev = previous_path(exe);
    if !prev.is_file() {
        return Err(Failure::Refused("there is no earlier version kept".into()));
    }
    let restored = previous_version(exe).unwrap_or_default();
    let tmp = side_path(exe, ".swap");
    remove_if_there(&tmp)?;
    std::fs::rename(exe, &tmp).map_err(|e| disk(exe, e))?;
    if let Err(e) = std::fs::rename(&prev, exe) {
        let _ = std::fs::rename(&tmp, exe);
        return Err(disk(&prev, e));
    }
    std::fs::rename(&tmp, &prev).map_err(|e| disk(&tmp, e))?;
    let _ = std::fs::write(previous_version_path(exe), current_version);
    Ok(restored)
}

// ---- the history

/// One line of the history of updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Seconds since 1970 (UTC).
    pub time: u64,
    /// `update`, `undo` or `failed`.
    pub what: String,
    pub from: String,
    pub to: String,
    pub note: String,
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// `yyyy-mm-dd hh:mm` (UTC) of a time in seconds.
pub fn format_time(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // civil from days (proleptic Gregorian)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}",
        rem / 3_600,
        rem % 3_600 / 60
    )
}

const HISTORY_MAX: usize = 50;

fn clean(s: &str) -> String {
    s.replace(['\t', '\r', '\n'], " ")
}

/// Adds a line to the history file (the oldest are dropped beyond the last fifty).
pub fn add_history(file: &Path, e: &Entry) {
    let mut all = read_history(file);
    all.push(e.clone());
    let from = all.len().saturating_sub(HISTORY_MAX);
    let text: String = all[from..]
        .iter()
        .map(|e| {
            format!(
                "{}\t{}\t{}\t{}\t{}\n",
                e.time,
                clean(&e.what),
                clean(&e.from),
                clean(&e.to),
                clean(&e.note)
            )
        })
        .collect();
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(file, text);
}

/// The history, the oldest first.
pub fn read_history(file: &Path) -> Vec<Entry> {
    let Ok(text) = std::fs::read_to_string(file) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            Some(Entry {
                time: f.first()?.parse().ok()?,
                what: f.get(1)?.to_string(),
                from: f.get(2)?.to_string(),
                to: f.get(3)?.to_string(),
                note: f.get(4).unwrap_or(&"").to_string(),
            })
        })
        .collect()
}

// ---- the work, in a thread, with its steps in view

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Ask GitHub for the latest release.
    Ask,
    Download,
    Verify,
    Install,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Waiting,
    Running,
    Done,
    Failed,
}

/// What the window shows, kept by the thread that works.
#[derive(Debug, Default, Clone)]
pub struct Shared {
    pub busy: bool,
    /// A look at GitHub was made (with this result in `release` or `error`).
    pub checked: bool,
    pub release: Option<Release>,
    pub steps: Vec<(Step, StepState, f32)>,
    pub log: Vec<String>,
    pub error: Option<String>,
    /// The new program is in place (or the old one is back): it is used when the program is started again.
    pub restart_needed: bool,
    /// The line to add to the history of the window (the thread adds it to the file too).
    pub last: Option<Entry>,
}

pub type State = Arc<Mutex<Shared>>;

fn with(st: &State, f: impl FnOnce(&mut Shared)) {
    if let Ok(mut s) = st.lock() {
        f(&mut s);
    }
}

fn step(st: &State, which: Step, state: StepState, fraction: f32) {
    with(st, |s| {
        if let Some(x) = s.steps.iter_mut().find(|x| x.0 == which) {
            x.1 = state;
            x.2 = fraction;
        }
    });
}

fn say(st: &State, line: String) {
    with(st, |s| s.log.push(line));
}

/// Looks for the latest release in a thread.
pub fn spawn_check(
    st: &State,
    cfg: Config,
    label: Option<&'static str>,
) -> std::thread::JoinHandle<()> {
    with(st, |s| {
        s.busy = true;
        s.error = None;
        s.log.clear();
        s.steps = vec![(Step::Ask, StepState::Running, 0.0)];
    });
    let st = st.clone();
    std::thread::spawn(move || {
        let _t = crate::applog::thread_guard("update-check");
        let r = match label {
            Some(l) => fetch_release(&cfg, l),
            None => Err(Failure::NoPackage),
        };
        match &r {
            Ok(rel) => crate::applog::info("update", format!("latest release {}", rel.tag)),
            Err(e) => {
                crate::applog::warn("update", format!("the look for a new version failed: {e}"))
            }
        }
        with(&st, |s| {
            s.busy = false;
            s.checked = true;
            match r {
                Ok(rel) => {
                    s.log.push(format!("latest release: {}", rel.tag));
                    s.steps = vec![(Step::Ask, StepState::Done, 1.0)];
                    s.release = Some(rel);
                }
                Err(e) => {
                    s.log.push(e.to_string());
                    s.steps = vec![(Step::Ask, StepState::Failed, 0.0)];
                    s.error = Some(e.to_string());
                }
            }
        });
    })
}

/// What an update needs to know about where it works.
#[derive(Debug, Clone)]
pub struct Target {
    /// The running program (the file that is replaced).
    pub exe: PathBuf,
    /// Its version.
    pub version: String,
    /// The folder for the download.
    pub work: PathBuf,
    /// The history file.
    pub history: PathBuf,
    pub label: &'static str,
}

/// Downloads, checks and installs a release in a thread. Nothing is changed unless the checksum agrees.
pub fn spawn_update(
    st: &State,
    cfg: Config,
    release: Release,
    target: Target,
    cancel: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    with(st, |s| {
        s.busy = true;
        s.error = None;
        s.log.clear();
        s.restart_needed = false;
        s.steps = vec![
            (Step::Download, StepState::Waiting, 0.0),
            (Step::Verify, StepState::Waiting, 0.0),
            (Step::Install, StepState::Waiting, 0.0),
        ];
    });
    let st = st.clone();
    std::thread::spawn(move || {
        let _t = crate::applog::thread_guard("update");
        let r = run_update(&st, &cfg, &release, &target, &cancel);
        match &r {
            Ok(()) => crate::applog::info(
                "update",
                format!("updated from {} to {}", target.version, release.version),
            ),
            Err(e) => crate::applog::error("update", format!("the update failed: {e}")),
        }
        let _ = std::fs::remove_dir_all(&target.work);
        let (what, note) = match &r {
            Ok(()) => ("update", String::new()),
            Err(e) => ("failed", e.to_string()),
        };
        let entry = Entry {
            time: now(),
            what: what.into(),
            from: target.version.clone(),
            to: release.version.clone(),
            note,
        };
        add_history(&target.history, &entry);
        with(&st, |s| {
            s.busy = false;
            s.last = Some(entry);
            match r {
                Ok(()) => s.restart_needed = true,
                Err(e) => {
                    s.log.push(e.to_string());
                    s.error = Some(e.to_string());
                }
            }
        });
    })
}

fn run_update(
    st: &State,
    cfg: &Config,
    release: &Release,
    t: &Target,
    cancel: &AtomicBool,
) -> Done<()> {
    let pkg = release.package.clone().ok_or(Failure::NoPackage)?;
    if !is_newer(&release.version, &t.version) {
        return Err(Failure::Refused("the release is not newer".into()));
    }
    std::fs::create_dir_all(&t.work).map_err(|e| Failure::Disk(e.to_string()))?;
    let file = t.work.join(&pkg.name);
    // 1. the download
    step(st, Step::Download, StepState::Running, 0.0);
    say(st, format!("download {}", pkg.name));
    let r = download(cfg, &pkg, &file, cancel, &mut |got, total| {
        let f = total
            .filter(|t| *t > 0)
            .map_or(0.0, |t| got as f32 / t as f32);
        step(st, Step::Download, StepState::Running, f.min(1.0));
    });
    if let Err(e) = r {
        step(st, Step::Download, StepState::Failed, 0.0);
        return Err(e);
    }
    step(st, Step::Download, StepState::Done, 1.0);
    // 2. the checksum: the package is not opened before it agrees
    step(st, Step::Verify, StepState::Running, 0.0);
    let verified =
        read_text(cfg, &pkg.sum_url, MAX_SUM_BYTES).and_then(|sum| verify(&file, &pkg, &sum));
    if let Err(e) = verified {
        step(st, Step::Verify, StepState::Failed, 0.0);
        return Err(e);
    }
    say(st, "SHA-256 agrees".into());
    step(st, Step::Verify, StepState::Done, 1.0);
    // 3. the program is taken out of the package and put in place
    step(st, Step::Install, StepState::Running, 0.0);
    let staged = staging_path(&t.exe);
    let placed = extract_program(&file, t.label, &staged).and_then(|()| {
        step(st, Step::Install, StepState::Running, 0.5);
        install(&staged, &t.exe, &t.version)
    });
    if let Err(e) = placed {
        let _ = std::fs::remove_file(&staged);
        step(st, Step::Install, StepState::Failed, 0.0);
        return Err(e);
    }
    say(
        st,
        format!("{} is in place; {} is kept", release.version, t.version),
    );
    step(st, Step::Install, StepState::Done, 1.0);
    Ok(())
}

/// Puts the earlier version back (in the same thread-free way the window needs: it is quick). Adds the line to the history.
pub fn run_undo(st: &State, target: &Target) -> Done<String> {
    let r = undo(&target.exe, &target.version);
    let entry = Entry {
        time: now(),
        what: if r.is_ok() { "undo" } else { "failed" }.into(),
        from: target.version.clone(),
        to: r.as_ref().cloned().unwrap_or_default(),
        note: r.as_ref().err().map(|e| e.to_string()).unwrap_or_default(),
    };
    add_history(&target.history, &entry);
    with(st, |s| {
        s.last = Some(entry);
        match &r {
            Ok(v) => {
                s.restart_needed = true;
                s.error = None;
                s.log.push(format!("back to {v}"));
            }
            Err(e) => s.error = Some(e.to_string()),
        }
    });
    r
}

/// Starts the program again (the one now at `exe`) with the same arguments; the caller closes the window afterwards.
pub fn restart(exe: &Path) -> std::io::Result<()> {
    std::process::Command::new(exe)
        .args(std::env::args_os().skip(1))
        .spawn()
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::net::TcpListener;

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("eh3d-bg-upd-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// A server of this computer that answers the paths it is given and counts the requests (the tests never touch the network).
    struct Fake {
        port: u16,
        stop: Arc<AtomicBool>,
        hits: Arc<Mutex<Vec<String>>>,
    }

    impl Drop for Fake {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
        }
    }

    /// The answers can name their own server with `{PORT}`, which is only known once the server is listening.
    fn fill_port(body: &[u8], port: u16) -> Vec<u8> {
        match std::str::from_utf8(body) {
            Ok(t) => t.replace("{PORT}", &port.to_string()).into_bytes(),
            Err(_) => body.to_vec(),
        }
    }

    fn serve(routes: HashMap<String, Vec<u8>>) -> Fake {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        l.set_nonblocking(true).unwrap();
        let port = l.local_addr().unwrap().port();
        let stop = Arc::new(AtomicBool::new(false));
        let hits = Arc::new(Mutex::new(Vec::new()));
        let (s2, h2) = (stop.clone(), hits.clone());
        std::thread::spawn(move || {
            while !s2.load(Ordering::Relaxed) {
                match l.accept() {
                    Ok((mut c, _)) => {
                        c.set_nonblocking(false).unwrap();
                        let mut buf = [0u8; 4096];
                        let n = c.read(&mut buf).unwrap_or(0);
                        let req = String::from_utf8_lossy(&buf[..n]).to_string();
                        let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();
                        h2.lock().unwrap().push(path.clone());
                        let (code, body) = match routes.get(&path) {
                            Some(b) => ("200 OK", fill_port(b, port)),
                            None => ("404 Not Found", b"no".to_vec()),
                        };
                        let head = format!(
                            "HTTP/1.1 {code}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        let _ = c.write_all(head.as_bytes());
                        let _ = c.write_all(&body);
                    }
                    Err(_) => std::thread::sleep(Duration::from_millis(3)),
                }
            }
        });
        Fake { port, stop, hits }
    }

    fn tar_gz(program: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        {
            let gz = flate2::write::GzEncoder::new(&mut out, flate2::Compression::fast());
            let mut b = tar::Builder::new(gz);
            for (name, data) in [
                ("bungard-gui-0.2.0-linux-x86_64/README.md", &b"readme"[..]),
                ("bungard-gui-0.2.0-linux-x86_64/bungard-gui", program),
            ] {
                let mut h = tar::Header::new_gnu();
                h.set_size(data.len() as u64);
                h.set_mode(0o644);
                h.set_cksum();
                b.append_data(&mut h, name, data).unwrap();
            }
            b.into_inner().unwrap().finish().unwrap();
        }
        out
    }

    fn zip_of(program: &[u8]) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            let o = zip::write::SimpleFileOptions::default();
            z.start_file("bungard-gui-0.2.0-windows-x86_64/LICENSE", o)
                .unwrap();
            z.write_all(b"licence").unwrap();
            z.start_file("bungard-gui-0.2.0-windows-x86_64/bungard-gui.exe", o)
                .unwrap();
            z.write_all(program).unwrap();
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    fn hex(data: &[u8]) -> String {
        Sha256::digest(data)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    fn release_json(port: u16, label: &str, version: &str, with_sum: bool) -> String {
        let name = package_name(version, label);
        let base = format!("http://127.0.0.1:{port}");
        let mut assets = vec![format!(
            r#"{{"name":"{name}","browser_download_url":"{base}/pkg/{name}","size":10}}"#
        )];
        if with_sum {
            assets.push(format!(
                r#"{{"name":"{name}.sha256","browser_download_url":"{base}/pkg/{name}.sha256","size":100}}"#
            ));
        }
        format!(
            r#"{{"tag_name":"v{version}","draft":false,"prerelease":false,"html_url":"{base}/page","body":"- **Bold** gain with `code` and the name \"Cut and run\"\r\n- [Link](http://x.test/a) text","assets":[{}]}}"#,
            assets.join(",")
        )
    }

    fn cfg(port: u16) -> Config {
        Config {
            api_url: format!("http://127.0.0.1:{port}/latest"),
            timeout_s: 5,
        }
    }

    #[test]
    fn versions_are_compared_by_their_numbers() {
        assert!(is_newer("0.2.0", "0.1.9"));
        assert!(is_newer("v0.10.0", "0.9.9"));
        assert!(is_newer("0.8.0.1", "0.8.0"));
        assert!(is_newer("0.8.1.0", "0.8.0.9"));
        assert!(!is_newer("0.2.0", "0.2.0"));
        assert!(!is_newer("0.2.0.0", "0.2"));
        assert!(!is_newer("0.1.9", "0.2.0"));
        assert!(!is_newer("soon", "0.2.0"));
        assert!(!is_newer("1.2.3.4.5", "0.0.1"));
    }

    #[test]
    fn the_package_names_are_the_ones_of_the_release_workflow() {
        assert_eq!(
            package_name("0.1.9", "windows-x86"),
            "bungard-gui-0.1.9-windows-x86.zip"
        );
        assert_eq!(
            package_name("0.1.9", "windows-x86_64"),
            "bungard-gui-0.1.9-windows-x86_64.zip"
        );
        assert_eq!(
            package_name("0.1.9", "linux-x86_64"),
            "bungard-gui-0.1.9-linux-x86_64.tar.gz"
        );
        assert_eq!(
            package_name("0.1.9", "macos-arm64"),
            "bungard-gui-0.1.9-macos-arm64.tar.gz"
        );
        assert!(
            platform_label().is_some()
                || cfg!(not(any(windows, target_os = "linux", target_os = "macos")))
        );
    }

    #[test]
    fn the_notes_keep_their_words_and_lose_the_marks() {
        let n = plain_notes(
            "- **Bold:** a \"Name\" with `code` and [a link](http://x.test).\r\n\r\nAnd more",
        );
        assert_eq!(
            n,
            vec![
                "- Bold: a \"Name\" with code and a link.".to_string(),
                "And more".to_string()
            ]
        );
    }

    #[test]
    fn plain_http_is_only_for_this_computer() {
        let api = "http://127.0.0.1:9/latest";
        assert!(address_allowed("https://github.com/x", api));
        assert!(address_allowed("http://127.0.0.1:9/pkg", api));
        assert!(address_allowed("http://localhost:9/pkg", api));
        assert!(!address_allowed("http://example.com/pkg", api));
        assert!(!address_allowed("http://127.0.0.1.example.com/pkg", api));
        assert!(!address_allowed("ftp://127.0.0.1/pkg", api));
        // a local package is not taken from a remote API over plain http either
        assert!(!address_allowed(
            "http://127.0.0.1:9/pkg",
            "http://example.com/latest"
        ));
    }

    #[test]
    fn the_release_is_read_with_the_package_of_the_system() {
        let json = release_json(9, "linux-x86_64", "0.2.0", true);
        let r = parse_release(&json, "linux-x86_64", "http://127.0.0.1:9/latest").unwrap();
        assert_eq!((r.version.as_str(), r.tag.as_str()), ("0.2.0", "v0.2.0"));
        let p = r.package.unwrap();
        assert_eq!(p.name, "bungard-gui-0.2.0-linux-x86_64.tar.gz");
        assert!(p.sum_url.ends_with(".tar.gz.sha256"));
        assert!(r.notes[0].contains("\"Cut and run\""));
        // another system has no package in this release; without its checksum a package is not offered
        let other = parse_release(&json, "macos-arm64", "http://127.0.0.1:9/latest").unwrap();
        assert!(other.package.is_none());
        let json = release_json(9, "linux-x86_64", "0.2.0", false);
        assert!(
            parse_release(&json, "linux-x86_64", "http://127.0.0.1:9/latest")
                .unwrap()
                .package
                .is_none()
        );
        // a draft, a tag that is not a version and an address that is not allowed are refused
        let draft = release_json(9, "linux-x86_64", "0.2.0", true)
            .replace("\"draft\":false", "\"draft\":true");
        assert!(parse_release(&draft, "linux-x86_64", "http://127.0.0.1:9/latest").is_err());
        let tag = release_json(9, "linux-x86_64", "0.2.0", true).replace("v0.2.0\"", "nightly\"");
        assert!(parse_release(&tag, "linux-x86_64", "http://127.0.0.1:9/latest").is_err());
        let remote = release_json(9, "linux-x86_64", "0.2.0", true);
        assert!(matches!(
            parse_release(&remote, "linux-x86_64", "http://example.com/latest"),
            Err(Failure::Refused(_))
        ));
        assert!(parse_release("not json", "linux-x86_64", "https://x").is_err());
    }

    #[test]
    fn the_checksum_file_must_be_a_sha256_of_this_package() {
        let h = hex(b"abc");
        assert_eq!(
            parse_sum_file(&format!("{h}  a.zip\n"), "a.zip").unwrap(),
            h
        );
        assert_eq!(
            parse_sum_file(&format!("{}  *a.zip", h.to_uppercase()), "a.zip").unwrap(),
            h
        );
        assert!(parse_sum_file(&format!("{h}  other.zip"), "a.zip").is_err());
        assert!(parse_sum_file("deadbeef  a.zip", "a.zip").is_err());
        assert!(parse_sum_file("", "a.zip").is_err());
    }

    #[test]
    fn the_history_is_kept_and_cut_to_the_last_fifty() {
        let d = scratch("history");
        let f = d.join("sub").join("updates.log");
        for i in 0..55u64 {
            add_history(
                &f,
                &Entry {
                    time: i,
                    what: "update".into(),
                    from: "a\tb".into(),
                    to: "c".into(),
                    note: "x\ny".into(),
                },
            );
        }
        let h = read_history(&f);
        assert_eq!(h.len(), 50);
        assert_eq!(h[0].time, 5);
        assert_eq!(h[49].from, "a b");
        assert_eq!(format_time(0), "1970-01-01 00:00");
        assert_eq!(format_time(1_700_000_000), "2023-11-14 22:13");
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn a_package_gives_its_program_only() {
        let d = scratch("extract");
        let tgz = d.join("p.tar.gz");
        std::fs::write(&tgz, tar_gz(b"NEW LINUX")).unwrap();
        let out = d.join("out");
        extract_program(&tgz, "linux-x86_64", &out).unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), b"NEW LINUX");
        let z = d.join("p.zip");
        std::fs::write(&z, zip_of(b"NEW WINDOWS")).unwrap();
        let out2 = d.join("out2");
        extract_program(&z, "windows-x86_64", &out2).unwrap();
        assert_eq!(std::fs::read(&out2).unwrap(), b"NEW WINDOWS");
        // a package of another system has no such program; a file that is no package is refused
        assert!(extract_program(&z, "linux-x86_64", &d.join("o3")).is_err());
        std::fs::write(d.join("junk"), b"junk").unwrap();
        assert!(extract_program(&d.join("junk"), "linux-x86_64", &d.join("o4")).is_err());
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn installing_keeps_the_old_program_and_the_undo_swaps_them_back() {
        let d = scratch("install");
        let exe = d.join(format!("bungard-gui{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&exe, b"OLD").unwrap();
        let new = staging_path(&exe);
        std::fs::write(&new, b"NEW").unwrap();
        install(&new, &exe, "0.1.9").unwrap();
        assert_eq!(std::fs::read(&exe).unwrap(), b"NEW");
        assert_eq!(std::fs::read(previous_path(&exe)).unwrap(), b"OLD");
        assert_eq!(previous_version(&exe).as_deref(), Some("0.1.9"));
        assert!(!new.exists());
        // the undo puts the old one back and keeps the new one for the redo
        assert_eq!(undo(&exe, "0.2.0").unwrap(), "0.1.9");
        assert_eq!(std::fs::read(&exe).unwrap(), b"OLD");
        assert_eq!(std::fs::read(previous_path(&exe)).unwrap(), b"NEW");
        assert_eq!(previous_version(&exe).as_deref(), Some("0.2.0"));
        // a second install replaces the kept one
        let new = staging_path(&exe);
        std::fs::write(&new, b"NEWER").unwrap();
        install(&new, &exe, "0.1.9").unwrap();
        assert_eq!(std::fs::read(previous_path(&exe)).unwrap(), b"OLD");
        // nothing kept: nothing to undo
        std::fs::remove_file(previous_path(&exe)).unwrap();
        assert!(undo(&exe, "0.2.0").is_err());
        assert_eq!(std::fs::read(&exe).unwrap(), b"NEWER");
        let _ = std::fs::remove_dir_all(d);
    }

    fn target(d: &Path, label: &'static str) -> Target {
        let exe = d.join("bungard-gui");
        std::fs::write(&exe, b"OLD").unwrap();
        Target {
            exe,
            version: "0.1.9".into(),
            work: d.join("work"),
            history: d.join("updates.log"),
            label,
        }
    }

    fn wait(st: &State, h: std::thread::JoinHandle<()>) -> Shared {
        h.join().unwrap();
        st.lock().unwrap().clone()
    }

    #[test]
    fn an_update_downloads_checks_installs_and_is_written_in_the_history() {
        let d = scratch("full");
        let pkg = tar_gz(b"NEW PROGRAM");
        let name = package_name("0.2.0", "linux-x86_64");
        let mut routes = HashMap::new();
        routes.insert(format!("/pkg/{name}"), pkg.clone());
        routes.insert(
            format!("/pkg/{name}.sha256"),
            format!("{}  {name}\n", hex(&pkg)).into_bytes(),
        );
        routes.insert(
            "/latest".into(),
            release_json(0, "linux-x86_64", "0.2.0", true)
                .replace("127.0.0.1:0", "127.0.0.1:{PORT}")
                .into_bytes(),
        );
        let fake = serve(routes);
        let st: State = Arc::default();
        let c = cfg(fake.port);
        let h = spawn_check(&st, c.clone(), Some("linux-x86_64"));
        let s = wait(&st, h);
        assert!(s.checked && s.error.is_none(), "{:?}", s.error);
        let rel = s.release.clone().unwrap();
        assert_eq!(rel.version, "0.2.0");
        // the check asks for the release only, never for the package
        assert_eq!(fake.hits.lock().unwrap().as_slice(), ["/latest"]);
        let t = target(&d, "linux-x86_64");
        let h = spawn_update(&st, c, rel, t.clone(), Arc::new(AtomicBool::new(false)));
        let s = wait(&st, h);
        assert!(s.error.is_none(), "{:?}", s.error);
        assert!(s.restart_needed);
        assert!(s.steps.iter().all(|x| x.1 == StepState::Done));
        assert_eq!(std::fs::read(&t.exe).unwrap(), b"NEW PROGRAM");
        assert_eq!(std::fs::read(previous_path(&t.exe)).unwrap(), b"OLD");
        assert!(!t.work.exists(), "the download is cleaned up");
        assert!(!staging_path(&t.exe).exists());
        let h = read_history(&t.history);
        assert_eq!(
            (h[0].what.as_str(), h[0].from.as_str(), h[0].to.as_str()),
            ("update", "0.1.9", "0.2.0")
        );
        // the undo
        let back = run_undo(
            &st,
            &Target {
                version: "0.2.0".into(),
                ..t.clone()
            },
        )
        .unwrap();
        assert_eq!(back, "0.1.9");
        assert_eq!(std::fs::read(&t.exe).unwrap(), b"OLD");
        assert_eq!(read_history(&t.history)[1].what, "undo");
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn a_package_whose_checksum_does_not_agree_is_never_installed() {
        let d = scratch("badsum");
        let pkg = tar_gz(b"TAMPERED");
        let name = package_name("0.2.0", "linux-x86_64");
        let mut routes = HashMap::new();
        routes.insert(format!("/pkg/{name}"), pkg);
        // the checksum is the one of another file
        routes.insert(
            format!("/pkg/{name}.sha256"),
            format!(
                "{}  {name}
",
                hex(b"the real one")
            )
            .into_bytes(),
        );
        routes.insert(
            "/latest".into(),
            release_json(0, "linux-x86_64", "0.2.0", true)
                .replace("127.0.0.1:0", "127.0.0.1:{PORT}")
                .into_bytes(),
        );
        let fake = serve(routes);
        let st: State = Arc::default();
        let c = cfg(fake.port);
        let s = wait(&st, spawn_check(&st, c.clone(), Some("linux-x86_64")));
        let t = target(&d, "linux-x86_64");
        let s2 = wait(
            &st,
            spawn_update(
                &st,
                c,
                s.release.unwrap(),
                t.clone(),
                Arc::new(AtomicBool::new(false)),
            ),
        );
        assert!(
            s2.error.as_deref().unwrap_or("").contains("checksum"),
            "{:?}",
            s2.error
        );
        assert!(!s2.restart_needed);
        assert!(s2.steps.iter().any(|x| x.1 == StepState::Failed));
        assert_eq!(std::fs::read(&t.exe).unwrap(), b"OLD");
        assert!(!previous_path(&t.exe).exists());
        assert!(!staging_path(&t.exe).exists());
        assert_eq!(read_history(&t.history)[0].what, "failed");
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn a_release_that_is_not_newer_or_has_no_package_changes_nothing() {
        let d = scratch("notnewer");
        let st: State = Arc::default();
        let c = Config {
            api_url: "http://127.0.0.1:1/latest".into(),
            timeout_s: 2,
        };
        let t = target(&d, "linux-x86_64");
        let rel = |version: &str, package: Option<Package>| Release {
            tag: format!("v{version}"),
            version: version.into(),
            notes: vec![],
            page: String::new(),
            package,
        };
        let p = Package {
            name: package_name("0.1.9", "linux-x86_64"),
            url: "http://127.0.0.1:1/p".into(),
            sum_url: "http://127.0.0.1:1/s".into(),
            size: 0,
            digest: None,
        };
        let s = wait(
            &st,
            spawn_update(
                &st,
                c.clone(),
                rel("0.1.9", Some(p)),
                t.clone(),
                Arc::new(AtomicBool::new(false)),
            ),
        );
        assert!(s.error.as_deref().unwrap_or("").contains("not newer"));
        let s = wait(
            &st,
            spawn_update(
                &st,
                c,
                rel("0.3.0", None),
                t.clone(),
                Arc::new(AtomicBool::new(false)),
            ),
        );
        assert!(s.error.as_deref().unwrap_or("").contains("no package"));
        assert_eq!(std::fs::read(&t.exe).unwrap(), b"OLD");
        // no system label: nothing is looked for
        let s = wait(
            &st,
            spawn_check(
                &st,
                Config {
                    api_url: "http://127.0.0.1:1/latest".into(),
                    timeout_s: 1,
                },
                None,
            ),
        );
        assert!(s.error.is_some());
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn a_server_that_does_not_answer_is_told_and_nothing_hangs() {
        let probe = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);
        let st: State = Arc::default();
        let s = wait(&st, spawn_check(&st, cfg(port), Some("linux-x86_64")));
        assert!(s.checked && s.error.is_some() && s.release.is_none());
        let fake = serve(HashMap::new());
        let s = wait(&st, spawn_check(&st, cfg(fake.port), Some("linux-x86_64")));
        assert!(
            s.error.as_deref().unwrap_or("").contains("404"),
            "{:?}",
            s.error
        );
    }
}
