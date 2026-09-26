use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::error::{OpenIssaError, Result};

/// Immutable budget constraints allocated for a research operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchBudget {
    pub max_requests: usize,
    pub max_pages: usize,
    pub max_depth: usize,
    pub max_bytes: usize,
    pub max_runtime_secs: u64,
}

impl Default for ResearchBudget {
    fn default() -> Self {
        Self {
            max_requests: 25,
            max_pages: 50,
            max_depth: 3,
            max_bytes: 10 * 1024 * 1024, // 10 MB total
            max_runtime_secs: 120,       // 2 minutes
        }
    }
}

/// Thread-safe tracker that monitors consumed resources against the allocated `ResearchBudget`.
#[derive(Debug, Clone)]
pub struct BudgetTracker {
    budget: ResearchBudget,
    start_time: Instant,
    requests_used: Arc<AtomicUsize>,
    pages_used: Arc<AtomicUsize>,
    bytes_used: Arc<AtomicUsize>,
}

impl BudgetTracker {
    pub fn new(budget: ResearchBudget) -> Self {
        Self {
            budget,
            start_time: Instant::now(),
            requests_used: Arc::new(AtomicUsize::new(0)),
            pages_used: Arc::new(AtomicUsize::new(0)),
            bytes_used: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Check if the operation is still within budget limits.
    pub fn check_limits(&self) -> Result<()> {
        if self.start_time.elapsed() > Duration::from_secs(self.budget.max_runtime_secs) {
            return Err(OpenIssaError::BudgetExhausted {
                reason: format!("Runtime exceeded {}s limit", self.budget.max_runtime_secs),
            });
        }

        if self.requests_used.load(Ordering::Relaxed) >= self.budget.max_requests {
            return Err(OpenIssaError::BudgetExhausted {
                reason: format!("Requests exceeded {} limit", self.budget.max_requests),
            });
        }

        if self.bytes_used.load(Ordering::Relaxed) >= self.budget.max_bytes {
            return Err(OpenIssaError::BudgetExhausted {
                reason: format!("Byte consumption exceeded {} limit", self.budget.max_bytes),
            });
        }

        Ok(())
    }

    /// Attempt to acquire a request lease.
    pub fn acquire_request_lease(&self) -> Result<()> {
        self.check_limits()?;
        let current = self.requests_used.fetch_add(1, Ordering::SeqCst);
        if current >= self.budget.max_requests {
            return Err(OpenIssaError::BudgetExhausted {
                reason: format!("Max requests of {} reached", self.budget.max_requests),
            });
        }
        Ok(())
    }

    /// Track byte consumption from network streams.
    pub fn record_bytes(&self, bytes: usize) -> Result<()> {
        let current = self.bytes_used.fetch_add(bytes, Ordering::SeqCst);
        if current + bytes > self.budget.max_bytes {
            return Err(OpenIssaError::PayloadTooLarge {
                size: current + bytes,
                max: self.budget.max_bytes,
            });
        }
        Ok(())
    }

    /// Record a processed page.
    pub fn record_page(&self) -> Result<()> {
        self.pages_used.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn requests_count(&self) -> usize {
        self.requests_used.load(Ordering::Relaxed)
    }

    pub fn bytes_count(&self) -> usize {
        self.bytes_used.load(Ordering::Relaxed)
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_exhaustion() {
        let budget = ResearchBudget {
            max_requests: 2,
            ..Default::default()
        };
        let tracker = BudgetTracker::new(budget);

        assert!(tracker.acquire_request_lease().is_ok());
        assert!(tracker.acquire_request_lease().is_ok());
        assert!(tracker.acquire_request_lease().is_err());
    }
}
