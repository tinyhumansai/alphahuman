//! Buffer sizes and string limits for screen intelligence.

pub(crate) const MAX_EPHEMERAL_FRAMES: usize = 120;
pub(crate) const MAX_EPHEMERAL_VISION_SUMMARIES: usize = 120;
pub(crate) const MAX_SCREENSHOT_BYTES: usize = 1_500_000;
pub(crate) const MAX_CONTEXT_CHARS: usize = 256;
pub(crate) const MAX_SUGGESTION_CHARS: usize = 128;
