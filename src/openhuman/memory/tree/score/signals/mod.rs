//! Score signals + weighted combine (Phase 2 / #708).
//!
//! Each submodule computes one scoring signal in `[0.0, 1.0]`. [`combine`]
//! aggregates them into a total score using per-signal weights. The output
//! is still `[0.0, 1.0]` after normalisation by total weight.
//!
//! Storing per-signal values alongside the total (via [`ScoreSignals`]) is
//! what makes admission decisions debuggable — when a chunk is dropped, we
//! persist *which* signals fired at what values.

pub mod interaction;
pub mod metadata_weight;
pub mod source_weight;
pub mod token_count;
pub mod unique_words;

use serde::{Deserialize, Serialize};

use crate::openhuman::memory::tree::score::extract::ExtractedEntities;
use crate::openhuman::memory::tree::types::Metadata;

/// Per-signal score breakdown for one chunk. Persisted alongside the total
/// for diagnostics.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScoreSignals {
    pub token_count: f32,
    pub unique_words: f32,
    pub metadata_weight: f32,
    pub source_weight: f32,
    pub interaction: f32,
    pub entity_density: f32,
}

/// Default weights applied to each signal in [`combine`].
#[derive(Clone, Debug)]
pub struct SignalWeights {
    pub token_count: f32,
    pub unique_words: f32,
    pub metadata_weight: f32,
    pub source_weight: f32,
    pub interaction: f32,
    pub entity_density: f32,
}

impl Default for SignalWeights {
    fn default() -> Self {
        Self {
            token_count: 1.0,
            unique_words: 1.0,
            metadata_weight: 1.5,
            source_weight: 1.5,
            interaction: 3.0, // strongest signal — direct user engagement
            entity_density: 1.0,
        }
    }
}

/// Compute all signals for a chunk.
pub fn compute(
    meta: &Metadata,
    content: &str,
    token_count: u32,
    ex: &ExtractedEntities,
) -> ScoreSignals {
    ScoreSignals {
        token_count: token_count::score(token_count),
        unique_words: unique_words::score(content),
        metadata_weight: metadata_weight::score(meta),
        source_weight: source_weight::score(meta),
        interaction: interaction::score(meta),
        entity_density: entity_density_score(token_count, ex),
    }
}

/// Entity-density signal: entities per token, capped.
///
/// More distinct entities per unit of content → more substantive. Calibrated
/// so ~1 entity per 100 tokens maxes out the signal.
pub fn entity_density_score(token_count: u32, ex: &ExtractedEntities) -> f32 {
    let unique = ex.unique_entity_count() as f32;
    if token_count == 0 {
        return 0.0;
    }
    let per_token = unique / token_count as f32;
    // cap at 0.01 entities/token = 1 entity per 100 tokens
    (per_token / 0.01).min(1.0)
}

/// Weighted sum of signals, normalised to `[0.0, 1.0]`.
pub fn combine(signals: &ScoreSignals, w: &SignalWeights) -> f32 {
    let total_weight = w.token_count
        + w.unique_words
        + w.metadata_weight
        + w.source_weight
        + w.interaction
        + w.entity_density;
    if total_weight <= 0.0 {
        return 0.0;
    }
    let weighted = signals.token_count * w.token_count
        + signals.unique_words * w.unique_words
        + signals.metadata_weight * w.metadata_weight
        + signals.source_weight * w.source_weight
        + signals.interaction * w.interaction
        + signals.entity_density * w.entity_density;
    (weighted / total_weight).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openhuman::memory::tree::score::extract::{
        EntityKind, ExtractedEntities, ExtractedEntity,
    };
    use crate::openhuman::memory::tree::types::SourceKind;
    use chrono::Utc;

    fn meta(tags: &[&str], kind: SourceKind) -> Metadata {
        let mut m = Metadata::point_in_time(kind, "x", "owner", Utc::now());
        m.tags = tags.iter().map(|s| s.to_string()).collect();
        m
    }

    fn make_entities(n: usize) -> ExtractedEntities {
        ExtractedEntities {
            entities: (0..n)
                .map(|i| ExtractedEntity {
                    kind: EntityKind::Email,
                    text: format!("user{i}@example.com"),
                    span_start: 0,
                    span_end: 10,
                    score: 1.0,
                })
                .collect(),
            topics: vec![],
        }
    }

    #[test]
    fn combine_all_zeros_is_zero() {
        let s = ScoreSignals::default();
        assert!(combine(&s, &SignalWeights::default()) < 0.01);
    }

    #[test]
    fn combine_all_ones_is_one() {
        let s = ScoreSignals {
            token_count: 1.0,
            unique_words: 1.0,
            metadata_weight: 1.0,
            source_weight: 1.0,
            interaction: 1.0,
            entity_density: 1.0,
        };
        assert!((combine(&s, &SignalWeights::default()) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn weights_influence_total() {
        let s = ScoreSignals {
            token_count: 0.0,
            unique_words: 0.0,
            metadata_weight: 0.0,
            source_weight: 0.0,
            interaction: 1.0, // only interaction fires
            entity_density: 0.0,
        };
        // Default interaction weight = 3.0 out of (1+1+1.5+1.5+3+1) = 9.0
        let total = combine(&s, &SignalWeights::default());
        assert!((total - (3.0 / 9.0)).abs() < 1e-6);
    }

    #[test]
    fn compute_wires_all_signals() {
        let m = meta(&["reply"], SourceKind::Email);
        let ex = make_entities(3);
        let s = compute(
            &m,
            "Some substantive text about Phoenix launch planning.",
            12,
            &ex,
        );
        assert!(s.interaction > 0.0);
        assert!(s.metadata_weight > 0.0);
        assert!(s.source_weight > 0.0);
    }

    #[test]
    fn entity_density_scales() {
        // 1 entity / 100 tokens = max
        let ex = make_entities(1);
        assert!((entity_density_score(100, &ex) - 1.0).abs() < 1e-6);
        // 1 entity / 1000 tokens = 0.1
        assert!((entity_density_score(1000, &ex) - 0.1).abs() < 1e-6);
        // zero-token input is safely 0
        assert_eq!(entity_density_score(0, &ex), 0.0);
    }
}
