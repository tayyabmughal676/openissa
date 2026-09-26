use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::budget::ResearchBudget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Initialized,
    Running,
    Completed,
    BudgetExceeded,
    HaltedForHumanVerification,
}

/// An overarching research operation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSession {
    pub id: Uuid,
    pub goal: String,
    pub budget: ResearchBudget,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl ResearchSession {
    pub fn new(goal: String, budget: ResearchBudget) -> Self {
        Self {
            id: Uuid::new_v4(),
            goal,
            budget,
            status: SessionStatus::Initialized,
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    pub fn halt_for_verification(&mut self) {
        self.status = SessionStatus::HaltedForHumanVerification;
    }
}
