//! GitHub release checking, changelog fetching, and install-script updates.

use std::cmp::Ordering;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

const DEFAULT_REPO: &str = "mholtzhausen/promptly";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const USER_AGENT: &str = concat!("promptly/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    pub current: String,
    pub latest: String,
    pub latest_tag: String,
    pub changelog: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateCheckOutcome {
    UpToDate { current: String, latest: String },
    UpdateAvailable(UpdateInfo),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedVersion {
    major: u64,
    minor: u64,
    patch: u64,
    build: u64,
}

fn install_repo() -> String {
    std::env::var("PROMPTLY_INSTALL_REPO").unwrap_or_else(|_| DEFAULT_REPO.to_string())
}

fn github_get(url: &str) -> Result<(u16, String, String)> {
    let mut request = ureq::get(url).set("User-Agent", USER_AGENT);
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    let response = request
        .call()
        .with_context(|| format!("HTTP request failed: {url}"))?;
    let status = response.status();
    let final_url = response.get_url().to_string();
    let body = response
        .into_string()
        .with_context(|| format!("failed to read response body from {url} ({status})"))?;
    Ok((status, final_url, body))
}

fn raw_github_get(path: &str) -> Result<String> {
    let repo = install_repo();
    let url = format!("https://raw.githubusercontent.com/{repo}/{path}");
    let (status, _, body) = github_get(&url)?;
    if !(200..300).contains(&status) {
        bail!("HTTP {status} fetching {url}");
    }
    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LatestReleaseApi {
    tag_name: String,
    #[serde(default)]
    body: String,
}

/// Extract `v0.8.0+1` from `https://github.com/org/repo/releases/tag/v0.8.0%2B1`.
pub fn tag_from_releases_url(url: &str) -> Option<String> {
    let marker = "/releases/tag/";
    let encoded = url.find(marker).map(|idx| &url[idx + marker.len()..])?;
    let encoded = encoded.split(&['?', '#'][..]).next()?;
    if encoded.is_empty() {
        return None;
    }
    Some(decode_percent(encoded))
}

pub fn decode_percent(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

fn fetch_latest_tag_via_redirect() -> Result<String> {
    let repo = install_repo();
    let url = format!("https://github.com/{repo}/releases/latest");
    let (status, final_url, _) = github_get(&url)?;
    if !(200..300).contains(&status) {
        bail!("HTTP {status} fetching {url}");
    }
    tag_from_releases_url(&final_url)
        .with_context(|| format!("could not parse release tag from redirect URL: {final_url}"))
}

fn fetch_latest_tag_via_atom() -> Result<String> {
    let repo = install_repo();
    let url = format!("https://github.com/{repo}/releases.atom");
    let (status, _, body) = github_get(&url)?;
    if !(200..300).contains(&status) {
        bail!("HTTP {status} fetching {url}");
    }

    if let Some(tag) = atom_first_entry_title(&body) {
        return Ok(tag);
    }

    bail!("could not parse latest release from Atom feed");
}

fn atom_first_entry_title(atom: &str) -> Option<String> {
    let entry = atom.find("<entry>")?;
    let rest = &atom[entry..];
    let title_open = rest.find("<title>")? + "<title>".len();
    let title_rest = &rest[title_open..];
    let title_end = title_rest.find("</title>")?;
    let title = title_rest[..title_end].trim();
    if title.is_empty() {
        return None;
    }
    Some(title.to_string())
}

fn fetch_latest_release_via_api() -> Result<(String, String)> {
    let repo = install_repo();
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let mut request = ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    let response = request
        .call()
        .with_context(|| format!("GitHub API request failed: {url}"))?;
    let status = response.status();
    let body = response
        .into_string()
        .with_context(|| format!("failed to read GitHub API response body ({status})"))?;
    if !(200..300).contains(&status) {
        bail!("GitHub API returned HTTP {status}: {body}");
    }
    let release: LatestReleaseApi =
        serde_json::from_str(&body).context("could not parse GitHub API release JSON")?;
    Ok((release.tag_name, release.body))
}

fn resolve_latest_release() -> Result<(String, String)> {
    match fetch_latest_tag_via_redirect() {
        Ok(tag) => {
            log::debug!("Resolved latest release tag via redirect: {tag}");
            return Ok((tag, String::new()));
        }
        Err(e) => log::debug!("Latest release redirect lookup failed: {e:#}"),
    }

    match fetch_latest_tag_via_atom() {
        Ok(tag) => {
            log::debug!("Resolved latest release tag via Atom feed: {tag}");
            return Ok((tag, String::new()));
        }
        Err(e) => log::debug!("Latest release Atom feed lookup failed: {e:#}"),
    }

    let (tag, body) = fetch_latest_release_via_api()
        .context("could not determine latest release tag (redirect, Atom feed, and API all failed)")?;
    log::debug!("Resolved latest release tag via GitHub API: {tag}");
    Ok((tag, body))
}

pub fn normalize_tag(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

fn parse_version(raw: &str) -> Option<ParsedVersion> {
    let version = normalize_tag(raw.trim());
    let (core, build) = match version.split_once('+') {
        Some((core, build)) => (core, build.parse::<u64>().ok()?),
        None => (version, 0),
    };
    let mut parts = core.split('.');
    Some(ParsedVersion {
        major: parts.next()?.parse().ok()?,
        minor: parts.next()?.parse().ok()?,
        patch: parts.next()?.parse().ok()?,
        build,
    })
}

pub fn compare_versions(left: &str, right: &str) -> Ordering {
    match (parse_version(left), parse_version(right)) {
        (Some(a), Some(b)) => a
            .major
            .cmp(&b.major)
            .then_with(|| a.minor.cmp(&b.minor))
            .then_with(|| a.patch.cmp(&b.patch))
            .then_with(|| a.build.cmp(&b.build)),
        _ => left.cmp(right),
    }
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    compare_versions(latest, current) == Ordering::Greater
}

fn fetch_latest_release() -> Result<(String, String)> {
    resolve_latest_release()
}

pub fn changelog_between(current: &str, latest: &str, markdown: &str) -> String {
    let mut sections = Vec::new();
    let mut current_section: Option<(String, String)> = None;

    for line in markdown.lines() {
        if let Some(rest) = line.strip_prefix("## [") {
            if let Some((version, _)) = rest.split_once(']') {
                if let Some((ver, body)) = current_section.take() {
                    if is_newer(&ver, current) && compare_versions(&ver, latest) != Ordering::Greater
                    {
                        sections.push(format!("## [{ver}]{body}"));
                    }
                }
                let version = version.to_string();
                if parse_version(&version).is_none() {
                    continue;
                }
                if compare_versions(&version, latest) == Ordering::Greater {
                    break;
                }
                current_section = Some((version, String::new()));
                continue;
            }
        }
        if let Some((_, body)) = current_section.as_mut() {
            body.push('\n');
            body.push_str(line);
        }
    }

    if let Some((ver, body)) = current_section {
        if is_newer(&ver, current) && compare_versions(&ver, latest) != Ordering::Greater {
            sections.push(format!("## [{ver}]{body}"));
        }
    }

    let changelog = sections.join("\n\n").trim().to_string();
    if changelog.is_empty() {
        format!("No changelog entries found between {current} and {latest}.")
    } else {
        changelog
    }
}

fn fetch_changelog(current: &str, latest_tag: &str, release_body: &str) -> String {
    let changelog_path = format!("{latest_tag}/CHANGELOG.md");
    match raw_github_get(&changelog_path) {
        Ok(markdown) => changelog_between(current, normalize_tag(latest_tag), &markdown),
        Err(e) => {
            log::warn!("Failed to fetch CHANGELOG.md for {latest_tag}: {e}");
            if release_body.trim().is_empty() {
                format!(
                    "See release notes: https://github.com/{}/releases/tag/{latest_tag}",
                    install_repo()
                )
            } else {
                release_body.to_string()
            }
        }
    }
}

/// Check GitHub for updates relative to the running binary.
pub fn check_for_updates() -> Result<UpdateCheckOutcome> {
    let (latest_tag, release_body) = fetch_latest_release()?;
    let latest = normalize_tag(&latest_tag).to_string();
    let current = CURRENT_VERSION.to_string();

    if !is_newer(&latest, &current) {
        return Ok(UpdateCheckOutcome::UpToDate { current, latest });
    }

    let changelog = fetch_changelog(&current, &latest_tag, &release_body);
    Ok(UpdateCheckOutcome::UpdateAvailable(UpdateInfo {
        current,
        latest,
        latest_tag,
        changelog,
    }))
}

fn signal_running_instance() -> Result<()> {
    let lock_path = crate::config::lock_file_path();
    let contents = match std::fs::read_to_string(&lock_path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e).with_context(|| format!("read lock file {}", lock_path.display())),
    };

    let pid = contents
        .lines()
        .find_map(|line| line.strip_prefix("pid=").and_then(|pid| pid.parse::<i32>().ok()))
        .context("lock file missing pid=")?;

    if pid == std::process::id() as i32 {
        return Ok(());
    }

    #[cfg(unix)]
    {
        let status = unsafe { libc::kill(pid, libc::SIGTERM) };
        if status != 0 {
            let err = std::io::Error::last_os_error();
            log::warn!("Failed to signal running instance pid={pid}: {err}");
        }
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        log::warn!("Cannot signal running instance on non-Unix platforms");
    }

    Ok(())
}

/// Run the curl install script and shut down the running instance on success.
pub fn run_update() -> Result<()> {
    let repo = install_repo();
    let script_url = format!(
        "https://raw.githubusercontent.com/{repo}/main/scripts/install.sh"
    );

    let mut command = Command::new("bash");
    command
        .arg("-c")
        .arg(format!("curl -fsSL {script_url:?} | bash"))
        .env("PROMPTLY_MANAGE_SERVICE", "1")
        .env("PROMPTLY_INSTALL_REPO", &repo)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        command.env("GITHUB_TOKEN", token);
    }

    let status = command
        .status()
        .context("failed to run install script")?;
    if !status.success() {
        bail!("install script exited with {status}");
    }

    if Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "promptly.service"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        log::info!("Update complete; promptly.service restart should replace this instance");
        return Ok(());
    }

    signal_running_instance()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_for_updates_succeeds_on_network() {
        let outcome = check_for_updates().expect("update check should succeed");
        match outcome {
            UpdateCheckOutcome::UpToDate { latest, .. } | UpdateCheckOutcome::UpdateAvailable(
                UpdateInfo { latest, .. },
            ) => assert!(!latest.is_empty()),
        }
    }

    #[test]
    fn redirect_resolves_latest_tag_on_network() {
        let tag = fetch_latest_tag_via_redirect().expect("redirect should resolve a tag");
        assert!(tag.starts_with('v') || tag.chars().next().is_some_and(|c| c.is_ascii_digit()));
    }

    #[test]
    fn compare_versions_orders_semver_and_build_metadata() {
        assert_eq!(compare_versions("0.8.0", "0.8.0+1"), Ordering::Less);
        assert_eq!(compare_versions("0.8.0+1", "0.8.0+2"), Ordering::Less);
        assert_eq!(compare_versions("0.8.0+1", "0.9.0"), Ordering::Less);
        assert_eq!(compare_versions("0.8.0+1", "0.8.0+1"), Ordering::Equal);
        assert_eq!(compare_versions("v0.8.0+1", "0.8.0+1"), Ordering::Equal);
    }

    #[test]
    fn normalize_tag_strips_v_prefix() {
        assert_eq!(normalize_tag("v0.8.0+1"), "0.8.0+1");
        assert_eq!(normalize_tag("0.8.0"), "0.8.0");
    }

    #[test]
    fn tag_from_releases_url_decodes_percent_encoding() {
        let url = "https://github.com/mholtzhausen/promptly/releases/tag/v0.8.0%2B1";
        assert_eq!(
            tag_from_releases_url(url).as_deref(),
            Some("v0.8.0+1")
        );
    }

    #[test]
    fn atom_first_entry_title_parses_latest() {
        let atom = r#"<feed><entry><title>v0.9.0</title></entry></feed>"#;
        assert_eq!(atom_first_entry_title(atom).as_deref(), Some("v0.9.0"));
    }

    #[test]
    fn decode_percent_handles_plus() {
        assert_eq!(decode_percent("v0.8.0%2B1"), "v0.8.0+1");
    }

    #[test]
    fn changelog_between_collects_intermediate_versions() {
        let markdown = r#"# Changelog

## [Unreleased]

## [0.9.0] - 2026-07-01

### Added
- New feature

## [0.8.0+1] - 2026-06-25

### Fixed
- Bug fix

## [0.8.0] - 2026-06-25

### Added
- Older feature
"#;
        let changelog = changelog_between("0.8.0+1", "0.9.0", markdown);
        assert!(changelog.contains("## [0.9.0]"));
        assert!(changelog.contains("New feature"));
        assert!(!changelog.contains("Older feature"));
        assert!(!changelog.contains("Bug fix"));
    }
}
