use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

use crate::cleaner::clean_html_to_markdown;
use openissa_browser::BrowserSupervisor;
use openissa_core::error::{OpenIssaError, Result};

#[derive(Debug, Clone)]
pub struct FetchOptions {
    pub timeout: Duration,
    pub max_bytes: usize,
    pub user_agent: String,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(15),
            max_bytes: 5 * 1024 * 1024, // 5 MB
            user_agent: "OpenISSA/0.1 (+https://github.com/tayyabmughal676/openissa)".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FetchResult {
    pub url: String,
    pub status: u16,
    pub content_type: String,
    pub markdown: String,
    pub raw_bytes_count: usize,
    pub duration: Duration,
    pub requires_browser: bool,
}

pub struct SmartFetcher {
    client: Client,
    options: FetchOptions,
    browser: Option<Arc<dyn BrowserSupervisor>>,
}

impl SmartFetcher {
    pub fn new(options: FetchOptions) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&options.user_agent)
                .map_err(|e| OpenIssaError::Internal(e.to_string()))?,
        );

        let client = Client::builder()
            .timeout(options.timeout)
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(|e| OpenIssaError::FetchError {
                url: "init".to_string(),
                message: e.to_string(),
            })?;

        Ok(Self {
            client,
            options,
            browser: None,
        })
    }

    pub fn with_browser(mut self, browser: Arc<dyn BrowserSupervisor>) -> Self {
        self.browser = Some(browser);
        self
    }

    /// Fetch dynamically using headless Chromium (Tier 4 Ladder escalation)
    pub async fn fetch_browser(&self, url: &str) -> Result<FetchResult> {
        let start = Instant::now();
        let supervisor = self.browser.as_ref().ok_or_else(|| {
            OpenIssaError::Internal(
                "No BrowserSupervisor configured for dynamic Tier 4 rendering".to_string(),
            )
        })?;

        info!(
            "Tier 4 Escalation: Rendering '{}' via headless Chromium",
            url
        );
        let rendered = supervisor.render_url(url, None).await?;
        let markdown = clean_html_to_markdown(&rendered.html);

        Ok(FetchResult {
            url: url.to_string(),
            status: 200,
            content_type: "text/html; charset=utf-8".to_string(),
            markdown,
            raw_bytes_count: rendered.html.len(),
            duration: start.elapsed(),
            requires_browser: false,
        })
    }

    /// Fetch a URL directly using HTTP/2, streaming bytes and converting to clean Markdown.
    /// If an empty CSR shell is detected, automatically escalates to Tier 4 Chromium.
    pub async fn fetch(&self, url: &str) -> Result<FetchResult> {
        let start = Instant::now();

        let response =
            self.client
                .get(url)
                .send()
                .await
                .map_err(|e| OpenIssaError::FetchError {
                    url: url.to_string(),
                    message: e.to_string(),
                })?;

        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/html")
            .to_string();

        let raw_bytes = response
            .bytes()
            .await
            .map_err(|e| OpenIssaError::FetchError {
                url: url.to_string(),
                message: e.to_string(),
            })?;

        if raw_bytes.len() > self.options.max_bytes {
            return Err(OpenIssaError::PayloadTooLarge {
                size: raw_bytes.len(),
                max: self.options.max_bytes,
            });
        }

        let raw_text = String::from_utf8_lossy(&raw_bytes).to_string();

        // Heuristics for Tier 3 -> Tier 4 Escalation (Client-Side Rendering)
        let requires_browser = (raw_bytes.len() < 2048
            && (raw_text.contains("<div id=\"root\"></div>")
                || raw_text.contains("<div id=\"app\"></div>")))
            || raw_text.contains("Enable JavaScript to run this app");

        // If CSR stub detected and browser is available, escalate to Tier 4 Chromium
        if requires_browser && self.browser.is_some() {
            info!(
                "Tier 4 Ladder: CSR stub detected on '{}', escalating to Chromium",
                url
            );
            if let Ok(browser_res) = self.fetch_browser(url).await {
                return Ok(browser_res);
            }
        }

        let markdown = clean_html_to_markdown(&raw_text);

        Ok(FetchResult {
            url: url.to_string(),
            status,
            content_type,
            markdown,
            raw_bytes_count: raw_bytes.len(),
            duration: start.elapsed(),
            requires_browser,
        })
    }
}
