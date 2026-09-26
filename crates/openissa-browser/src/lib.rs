//! OpenISSA Browser: Lazy Chromium / CDP supervisor for dynamic JavaScript rendering.

use async_trait::async_trait;
use openissa_core::error::Result;

pub mod chromium;
pub use chromium::ChromiumSupervisor;

#[derive(Debug, Clone)]
pub struct BrowserRenderResult {
    pub url: String,
    pub html: String,
    pub title: String,
}

#[async_trait]
pub trait BrowserSupervisor: Send + Sync {
    async fn render_url(
        &self,
        url: &str,
        wait_selector: Option<&str>,
    ) -> Result<BrowserRenderResult>;
}

/// Fallback mock supervisor for lightweight environments.
pub struct LazyBrowser;

impl LazyBrowser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LazyBrowser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BrowserSupervisor for LazyBrowser {
    async fn render_url(
        &self,
        url: &str,
        _wait_selector: Option<&str>,
    ) -> Result<BrowserRenderResult> {
        Ok(BrowserRenderResult {
            url: url.to_string(),
            html: format!(
                "<html><body><h1>Rendered content for {}</h1></body></html>",
                url
            ),
            title: "Dynamic Page".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lazy_browser_mock() {
        let browser = LazyBrowser::new();
        let res = browser.render_url("https://example.com", None).await;
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(res.html.contains("https://example.com"));
    }

    #[test]
    fn test_detect_chromium_executable() {
        // Should not panic on any system
        let _ = ChromiumSupervisor::detect_executable();
    }
}
