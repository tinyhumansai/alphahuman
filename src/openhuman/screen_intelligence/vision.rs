//! Vision query methods — recent summaries, flush, and analyze-and-persist.

use super::helpers::push_ephemeral_vision_summary;
use super::state::AccessibilityEngine;
use super::types::{CaptureFrame, VisionFlushResult, VisionRecentResult, VisionSummary};

impl AccessibilityEngine {
    pub async fn vision_recent(&self, limit: Option<usize>) -> VisionRecentResult {
        let state = self.inner.lock().await;
        let max_items = limit.unwrap_or(10).clamp(1, 120);

        let summaries = state
            .session
            .as_ref()
            .map(|session| {
                session
                    .vision_summaries
                    .iter()
                    .rev()
                    .take(max_items)
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        VisionRecentResult { summaries }
    }

    pub async fn vision_flush(&self) -> Result<VisionFlushResult, String> {
        let candidate = {
            let mut state = self.inner.lock().await;
            let Some(session) = state.session.as_mut() else {
                return Ok(VisionFlushResult {
                    accepted: false,
                    summary: None,
                });
            };

            let latest = session
                .frames
                .iter()
                .rev()
                .find(|f| f.image_ref.is_some())
                .cloned();
            if let Some(frame) = latest.clone() {
                session.vision_state = "queued".to_string();
                session.vision_queue_depth = session.vision_queue_depth.saturating_add(1);
                Some(frame)
            } else {
                None
            }
        };

        let Some(frame) = candidate else {
            return Ok(VisionFlushResult {
                accepted: false,
                summary: None,
            });
        };

        let summary = match super::processing_worker::analyze_frame(self, frame).await {
            Ok(summary) => summary,
            Err(err) => {
                let mut state = self.inner.lock().await;
                if let Some(session) = state.session.as_mut() {
                    session.vision_queue_depth = session.vision_queue_depth.saturating_sub(1);
                    session.vision_state = "error".to_string();
                }
                state.last_error = Some(format!("vision_flush_analysis_failed: {err}"));
                return Err(format!("vision flush failed: {err}"));
            }
        };

        let persist = super::helpers::persist_vision_summary(summary.clone())
            .await
            .map_err(|err| format!("vision summary persistence failed: {err}"));

        {
            let mut state = self.inner.lock().await;
            if let Some(session) = state.session.as_mut() {
                session.vision_queue_depth = session.vision_queue_depth.saturating_sub(1);
                push_ephemeral_vision_summary(&mut session.vision_summaries, summary.clone());
                session.last_vision_at_ms = Some(summary.captured_at_ms);
                session.last_vision_summary = Some(summary.key_text.clone());
                match &persist {
                    Ok(result) => {
                        session.vision_state = "ready".to_string();
                        session.vision_persist_count =
                            session.vision_persist_count.saturating_add(1);
                        session.last_vision_persisted_key = Some(result.key.clone());
                        session.last_vision_persist_error = None;
                    }
                    Err(err) => {
                        session.vision_state = "error".to_string();
                        session.last_vision_persist_error = Some(err.clone());
                        state.last_error = Some(format!("vision_flush_persist_failed: {err}"));
                    }
                }
            }
        }

        if let Err(err) = persist {
            return Err(format!("vision flush failed: {err}"));
        }

        Ok(VisionFlushResult {
            accepted: true,
            summary: Some(summary),
        })
    }

    /// Deterministic pipeline hook used by tests and diagnostics:
    /// analyze one frame with the local vision model and persist the summary to memory.
    pub async fn analyze_and_persist_frame(
        &self,
        frame: CaptureFrame,
    ) -> Result<VisionSummary, String> {
        let summary = super::processing_worker::analyze_frame(self, frame).await?;
        let persisted = super::helpers::persist_vision_summary(summary.clone())
            .await
            .map_err(|err| format!("vision summary persistence failed: {err}"))?;
        tracing::debug!(
            "[screen_intelligence] analyze_and_persist_frame completed (namespace={} key={})",
            persisted.namespace,
            persisted.key
        );
        Ok(summary)
    }
}
