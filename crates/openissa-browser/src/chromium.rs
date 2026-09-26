use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tracing::{info, warn};

use crate::{BrowserRenderResult, BrowserSupervisor};
use openissa_core::error::{OpenIssaError, Result};

pub struct ChromiumSupervisor {
    executable_path: Option<PathBuf>,
    virtual_time_budget_ms: u64,
}

impl ChromiumSupervisor {
    pub fn new() -> Self {
        let executable_path = Self::detect_executable();
        if let Some(ref path) = executable_path {
            info!("Chromium supervisor detected browser at: {:?}", path);
        } else {
            warn!(
                "No Chromium-compatible browser binary found. Tier 4 dynamic rendering disabled."
            );
        }

        Self {
            executable_path,
            virtual_time_budget_ms: 3000,
        }
    }

    pub fn with_virtual_time_budget(mut self, ms: u64) -> Self {
        self.virtual_time_budget_ms = ms;
        self
    }

    pub fn is_available(&self) -> bool {
        self.executable_path.is_some()
    }

    /// Automatically find an installed Chrome, Brave, Edge, or Chromium binary
    pub fn detect_executable() -> Option<PathBuf> {
        // 1. Explicit environment variable override
        if let Ok(env_path) =
            std::env::var("CHROME_BIN").or_else(|_| std::env::var("CHROMIUM_PATH"))
        {
            let p = PathBuf::from(env_path.trim());
            if p.exists() {
                return Some(p);
            }
        }

        // 2. Platform-specific known candidate paths
        #[cfg(target_os = "macos")]
        let candidates = [
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/Applications/Arc.app/Contents/MacOS/Arc",
        ];

        #[cfg(target_os = "linux")]
        let candidates = [
            "/usr/bin/google-chrome-stable",
            "/usr/bin/google-chrome",
            "/usr/bin/chromium-browser",
            "/usr/bin/chromium",
            "/snap/bin/chromium",
        ];

        #[cfg(target_os = "windows")]
        let candidates = [
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe",
        ];

        for candidate in candidates {
            let path = Path::new(candidate);
            if path.exists() {
                return Some(path.to_path_buf());
            }
        }

        // 3. Fallback: Search PATH using standard library
        if let Ok(paths) = std::env::var("PATH") {
            for dir in std::env::split_paths(&paths) {
                for binary in &[
                    "google-chrome",
                    "google-chrome-stable",
                    "chromium",
                    "chromium-browser",
                    "chrome",
                ] {
                    let p = dir.join(binary);
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }

        None
    }

    fn extract_title(html: &str) -> String {
        if let Some(start) = html.find("<title>") {
            if let Some(end) = html[start + 7..].find("</title>") {
                return html[start + 7..start + 7 + end].trim().to_string();
            }
        }
        "Rendered Page".to_string()
    }
}

impl Default for ChromiumSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BrowserSupervisor for ChromiumSupervisor {
    async fn render_url(
        &self,
        url: &str,
        _wait_selector: Option<&str>,
    ) -> Result<BrowserRenderResult> {
        let exe = self.executable_path.as_ref().ok_or_else(|| {
            OpenIssaError::Internal(
                "No Chromium binary available on system for Tier 4 dynamic rendering".to_string(),
            )
        })?;

        let budget_arg = format!("--virtual-time-budget={}", self.virtual_time_budget_ms);

        let mut cmd = Command::new(exe);
        cmd.arg("--headless=new")
            .arg("--dump-dom")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg(&budget_arg)
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let timeout_duration = Duration::from_millis(self.virtual_time_budget_ms + 7000);

        let output = tokio::time::timeout(timeout_duration, cmd.output())
            .await
            .map_err(|_| OpenIssaError::FetchError {
                url: url.to_string(),
                message: "Chromium rendering timed out after 10s".to_string(),
            })?
            .map_err(|e| OpenIssaError::FetchError {
                url: url.to_string(),
                message: format!("Failed to launch Chromium: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Chromium exited with status {}: {}", output.status, stderr);
        }

        let html = String::from_utf8_lossy(&output.stdout).to_string();
        if html.trim().is_empty() {
            return Err(OpenIssaError::FetchError {
                url: url.to_string(),
                message: "Chromium produced empty DOM dump".to_string(),
            });
        }

        let title = Self::extract_title(&html);

        Ok(BrowserRenderResult {
            url: url.to_string(),
            html,
            title,
        })
    }
}
