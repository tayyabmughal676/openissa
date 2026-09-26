use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Source quality hierarchy for evidence classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum SourceTier {
    /// RFCs, official API docs, official git repositories, official package registries.
    Primary = 1,
    /// Verified engineering blogs, conference papers, peer-reviewed articles.
    Secondary = 2,
    /// StackOverflow, Reddit discussions, forum threads.
    Community = 3,
    /// SEO aggregators, content farms, unverified AI summaries.
    Untrusted = 4,
}

impl SourceTier {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Primary => "Primary Documentation / Spec",
            Self::Secondary => "Engineering Blog / Technical Press",
            Self::Community => "Community Discussion / Forum",
            Self::Untrusted => "Low-Confidence Aggregator",
        }
    }

    /// Weight assigned during confidence calculation.
    pub fn weight(&self) -> f32 {
        match self {
            Self::Primary => 0.6,
            Self::Secondary => 0.35,
            Self::Community => 0.15,
            Self::Untrusted => 0.02,
        }
    }
}

/// A discovered web source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: Uuid,
    pub url: String,
    pub domain: String,
    pub title: Option<String>,
    pub tier: SourceTier,
    pub discovered_at: DateTime<Utc>,
}

impl Source {
    pub fn new(url: String, domain: String, tier: SourceTier) -> Self {
        Self {
            id: Uuid::new_v4(),
            url,
            domain,
            title: None,
            tier,
            discovered_at: Utc::now(),
        }
    }
}

/// An atomic fact or claim extracted during research.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: Uuid,
    pub session_id: Uuid,
    pub statement: String,
    pub confidence: f32,
    pub verified: bool,
    pub evidence_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl Claim {
    pub fn new(session_id: Uuid, statement: String, confidence: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            statement,
            confidence,
            verified: false,
            evidence_ids: Vec::new(),
            created_at: Utc::now(),
        }
    }
}

/// Specific citation or snippet supporting or refuting a claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: Uuid,
    pub claim_id: Uuid,
    pub source_id: Uuid,
    pub snippet: String,
    pub is_contradiction: bool,
    pub captured_at: DateTime<Utc>,
}

impl Evidence {
    pub fn new(claim_id: Uuid, source_id: Uuid, snippet: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            claim_id,
            source_id,
            snippet,
            is_contradiction: false,
            captured_at: Utc::now(),
        }
    }
}

/// Detected contradiction between claims or sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub id: Uuid,
    pub claim_a_id: Uuid,
    pub claim_b_id: Uuid,
    pub description: String,
    pub resolved_by_live_test: bool,
    pub winning_claim_id: Option<Uuid>,
}

/// Directed edge relating claims, evidence, and experimental verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceEdge {
    SupportedBy { claim_id: Uuid, evidence_id: Uuid },
    RefutedBy { claim_id: Uuid, evidence_id: Uuid },
    Contradicts { claim_a: Uuid, claim_b: Uuid },
    VerifiedByLiveTest { claim_id: Uuid },
}

/// Formal Directed Acyclic Graph (DAG) for research claims, source provenance, and contradictions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceGraph {
    pub sources: HashMap<Uuid, Source>,
    pub claims: HashMap<Uuid, Claim>,
    pub evidence: HashMap<Uuid, Evidence>,
    pub contradictions: Vec<Contradiction>,
    pub edges: Vec<EvidenceEdge>,
}

impl EvidenceGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new discovered web source to the graph.
    pub fn add_source(&mut self, url: String, domain: String, tier: SourceTier) -> Uuid {
        let source = Source::new(url, domain, tier);
        let id = source.id;
        self.sources.insert(id, source);
        id
    }

    /// Register a new hypothesis or extracted claim.
    pub fn add_claim(&mut self, session_id: Uuid, statement: String, confidence: f32) -> Uuid {
        let claim = Claim::new(session_id, statement, confidence);
        let id = claim.id;
        self.claims.insert(id, claim);
        id
    }

    /// Attach supporting or refuting evidence to a claim from a known source.
    pub fn attach_evidence(
        &mut self,
        claim_id: Uuid,
        source_id: Uuid,
        snippet: String,
        is_contradiction: bool,
    ) -> Option<Uuid> {
        if !self.claims.contains_key(&claim_id) || !self.sources.contains_key(&source_id) {
            return None;
        }

        let mut ev = Evidence::new(claim_id, source_id, snippet);
        ev.is_contradiction = is_contradiction;
        let ev_id = ev.id;
        self.evidence.insert(ev_id, ev);

        if let Some(claim) = self.claims.get_mut(&claim_id) {
            claim.evidence_ids.push(ev_id);
        }

        if is_contradiction {
            self.edges.push(EvidenceEdge::RefutedBy {
                claim_id,
                evidence_id: ev_id,
            });
        } else {
            self.edges.push(EvidenceEdge::SupportedBy {
                claim_id,
                evidence_id: ev_id,
            });
        }

        self.recompute_confidence(claim_id);
        Some(ev_id)
    }

    /// Record a contradiction between two competing claims.
    pub fn record_contradiction(
        &mut self,
        claim_a_id: Uuid,
        claim_b_id: Uuid,
        description: String,
    ) -> Uuid {
        let c = Contradiction {
            id: Uuid::new_v4(),
            claim_a_id,
            claim_b_id,
            description,
            resolved_by_live_test: false,
            winning_claim_id: None,
        };
        let id = c.id;
        self.contradictions.push(c);
        self.edges.push(EvidenceEdge::Contradicts {
            claim_a: claim_a_id,
            claim_b: claim_b_id,
        });

        self.recompute_confidence(claim_a_id);
        self.recompute_confidence(claim_b_id);
        id
    }

    /// Settle a contradiction definitively via a Web Lab live test.
    pub fn resolve_contradiction_with_live_test(
        &mut self,
        contradiction_id: Uuid,
        winning_claim_id: Uuid,
    ) -> bool {
        let mut found = false;
        for c in &mut self.contradictions {
            if c.id == contradiction_id {
                c.resolved_by_live_test = true;
                c.winning_claim_id = Some(winning_claim_id);
                found = true;
                break;
            }
        }

        if found {
            if let Some(winner) = self.claims.get_mut(&winning_claim_id) {
                winner.verified = true;
                winner.confidence = 1.0;
            }
            self.edges.push(EvidenceEdge::VerifiedByLiveTest {
                claim_id: winning_claim_id,
            });
        }

        found
    }

    /// Mark a claim as explicitly verified (e.g. from primary documentation or empirical observation).
    pub fn mark_verified(&mut self, claim_id: Uuid, verified: bool) {
        if let Some(claim) = self.claims.get_mut(&claim_id) {
            claim.verified = verified;
            if verified && claim.confidence < 0.95 {
                claim.confidence = 0.95;
            }
        }
    }

    /// Deterministically calculate confidence score for a claim based on source hierarchy and contradictions.
    pub fn recompute_confidence(&mut self, claim_id: Uuid) {
        let Some(claim) = self.claims.get(&claim_id) else {
            return;
        };

        if claim.verified {
            return;
        }

        let mut score: f32 = 0.2; // Base prior

        for ev_id in &claim.evidence_ids {
            if let Some(ev) = self.evidence.get(ev_id) {
                if let Some(src) = self.sources.get(&ev.source_id) {
                    if ev.is_contradiction {
                        score -= src.tier.weight() * 0.5;
                    } else {
                        score += src.tier.weight();
                    }
                }
            }
        }

        // Penalty for unresolved contradictions
        let has_unresolved_contradiction = self.contradictions.iter().any(|c| {
            (c.claim_a_id == claim_id || c.claim_b_id == claim_id) && !c.resolved_by_live_test
        });

        if has_unresolved_contradiction {
            score *= 0.6;
        }

        let bounded_score = score.clamp(0.0, 1.0);

        if let Some(claim_mut) = self.claims.get_mut(&claim_id) {
            claim_mut.confidence = (bounded_score * 100.0).round() / 100.0;
            if claim_mut.confidence >= 0.85 {
                claim_mut.verified = true;
            }
        }
    }

    /// Render human-readable Markdown summary of the verified Evidence Graph.
    pub fn format_markdown_summary(&self) -> String {
        let mut out = String::from("# Evidence Graph & Research Synthesis\n\n");

        out.push_str("## Discovered Claims\n\n");
        for claim in self.claims.values() {
            let status = if claim.verified {
                "Verified"
            } else {
                "Unverified"
            };
            out.push_str(&format!(
                "- **Claim**: {}\n  * **Status**: {} (Confidence: {:.0}%)\n  * **Evidence Citations**: {}\n\n",
                claim.statement,
                status,
                claim.confidence * 100.0,
                claim.evidence_ids.len()
            ));
        }

        if !self.contradictions.is_empty() {
            out.push_str("## Flagged Contradictions\n\n");
            for c in &self.contradictions {
                let res = if c.resolved_by_live_test {
                    "Resolved by Web Lab Live Experiment"
                } else {
                    "Unresolved Contradiction"
                };
                out.push_str(&format!(
                    "- **Conflict**: {}\n  * **Resolution**: {}\n\n",
                    c.description, res
                ));
            }
        }

        out.push_str("## Primary & Secondary Sources\n\n");
        for src in self.sources.values() {
            out.push_str(&format!(
                "- [{}]({})\n  * **Domain**: {}\n  * **Tier**: {}\n\n",
                src.title.as_deref().unwrap_or(&src.url),
                src.url,
                src.domain,
                src.tier.name()
            ));
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_graph_lifecycle() {
        let mut graph = EvidenceGraph::new();
        let session_id = Uuid::new_v4();

        // 1. Add sources
        let rfc_source = graph.add_source(
            "https://www.rfc-editor.org/rfc/rfc9110".to_string(),
            "rfc-editor.org".to_string(),
            SourceTier::Primary,
        );
        let blog_source = graph.add_source(
            "https://random-dev-blog.net/post".to_string(),
            "random-dev-blog.net".to_string(),
            SourceTier::Untrusted,
        );

        // 2. Add claim
        let claim_id = graph.add_claim(
            session_id,
            "HTTP 429 indicates rate-limiting".to_string(),
            0.5,
        );

        // 3. Attach primary evidence
        graph.attach_evidence(
            claim_id,
            rfc_source,
            "Section 15.5.30: 429 Too Many Requests indicates rate limiting".to_string(),
            false,
        );

        let claim = graph.claims.get(&claim_id).expect("claim should exist");
        assert!(claim.confidence > 0.7);

        // 4. Attach contradiction from untrusted source
        let claim_b = graph.add_claim(
            session_id,
            "HTTP 429 is an internal server error".to_string(),
            0.5,
        );
        graph.attach_evidence(
            claim_b,
            blog_source,
            "429 means server crashed".to_string(),
            true,
        );

        let contra_id = graph.record_contradiction(
            claim_id,
            claim_b,
            "Disagreement on HTTP 429 semantics".to_string(),
        );

        // 5. Resolve via live Web Lab test
        let resolved = graph.resolve_contradiction_with_live_test(contra_id, claim_id);
        assert!(resolved);

        let winner = graph.claims.get(&claim_id).expect("claim should exist");
        assert!(winner.verified);
        assert_eq!(winner.confidence, 1.0);

        let md = graph.format_markdown_summary();
        assert!(md.contains("Evidence Graph"));
        assert!(md.contains("Resolved by Web Lab Live Experiment"));
    }
}
