//! Open http(s) URLs in the system default browser.

use std::process::Command;

pub fn is_allowed_browser_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

pub fn open_url_in_browser(url: &str) -> anyhow::Result<()> {
    if !is_allowed_browser_url(url) {
        anyhow::bail!("Refusing to open non-http(s) URL");
    }
    let status = Command::new("xdg-open").arg(url).status()?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("xdg-open exited with {status}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_http_and_https_urls() {
        assert!(is_allowed_browser_url("https://claude.ai/new"));
        assert!(is_allowed_browser_url("http://example.com"));
    }

    #[test]
    fn rejects_non_http_urls() {
        assert!(!is_allowed_browser_url("file:///etc/passwd"));
        assert!(!is_allowed_browser_url("javascript:alert(1)"));
    }
}
