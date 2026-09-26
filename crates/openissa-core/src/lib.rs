//! OpenISSA Core: Domain models, error types, budgets, and research session entities.

pub mod budget;
pub mod config;
pub mod error;
pub mod evidence;
pub mod session;

pub use budget::{BudgetTracker, ResearchBudget};
pub use config::{GeneralConfig, LadderConfig, OpenIssaConfig, ProvidersConfig, ResearchConfig};
pub use error::{OpenIssaError, Result};
pub use evidence::{
    Claim, Contradiction, Evidence, EvidenceEdge, EvidenceGraph, Source, SourceTier,
};
pub use session::{ResearchSession, SessionStatus};
