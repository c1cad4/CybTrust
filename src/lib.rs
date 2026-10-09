//! Reputation is derived from recorded reviews, never directly assigned.
//! Reviewer identity and evidence authenticity must be verified upstream.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewEvent {
    pub event_id: String,
    pub subject: String,
    pub reviewer: String,
    pub evidence_id: String,
    pub accepted: bool,
}

#[derive(Default)]
pub struct TrustLedger {
    events: Vec<ReviewEvent>,
    seen: BTreeSet<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TrustError {
    InvalidEvent,
    SelfReview,
    DuplicateEvent,
}

impl TrustLedger {
    pub fn record(&mut self, event: ReviewEvent) -> Result<(), TrustError> {
        if event.event_id.trim().is_empty()
            || event.subject.trim().is_empty()
            || event.reviewer.trim().is_empty()
            || event.evidence_id.trim().is_empty()
        {
            return Err(TrustError::InvalidEvent);
        }
        if event.subject == event.reviewer {
            return Err(TrustError::SelfReview);
        }
        if !self.seen.insert(event.event_id.clone()) {
            return Err(TrustError::DuplicateEvent);
        }
        self.events.push(event);
        Ok(())
    }

    pub fn score(&self, subject: &str) -> i64 {
        self.events.iter().filter(|e| e.subject == subject).map(|e| {
            if e.accepted { 1 } else { -1 }
        }).sum()
    }

    pub fn scores(&self) -> BTreeMap<String, i64> {
        let mut result = BTreeMap::new();
        for event in &self.events {
            *result.entry(event.subject.clone()).or_insert(0) +=
                if event.accepted { 1 } else { -1 };
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(id: &str, accepted: bool) -> ReviewEvent {
        ReviewEvent {
            event_id: id.into(),
            subject: "agent-a".into(),
            reviewer: "agent-b".into(),
            evidence_id: "proof-1".into(),
            accepted,
        }
    }

    #[test]
    fn computes_scores_from_reviews() {
        let mut ledger = TrustLedger::default();
        ledger.record(event("1", true)).unwrap();
        ledger.record(event("2", false)).unwrap();
        assert_eq!(ledger.score("agent-a"), 0);
    }

    #[test]
    fn rejects_replays() {
        let mut ledger = TrustLedger::default();
        ledger.record(event("1", true)).unwrap();
        assert_eq!(ledger.record(event("1", true)), Err(TrustError::DuplicateEvent));
    }

    #[test]
    fn rejects_self_review() {
        let mut ledger = TrustLedger::default();
        let mut e = event("1", true);
        e.reviewer = e.subject.clone();
        assert_eq!(ledger.record(e), Err(TrustError::SelfReview));
    }
}
