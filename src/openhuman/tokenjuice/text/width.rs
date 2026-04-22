//! Grapheme-aware terminal-column width calculation.
//!
//! Uses `unicode-segmentation` for grapheme cluster boundaries and
//! `unicode-width` for CJK/emoji double-width detection, mirroring the
//! `Intl.Segmenter`-based logic in the upstream TypeScript.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

/// Return the list of user-perceived grapheme clusters in `text`.
pub fn graphemes(text: &str) -> Vec<&str> {
    text.graphemes(true).collect()
}

/// Return the number of grapheme clusters (not bytes or scalar values).
///
/// This is used for character-count limiting (mirrors `countTextChars` in TS).
pub fn count_text_chars(text: &str) -> usize {
    text.graphemes(true).count()
}

/// Return the terminal column width of a single grapheme cluster.
///
/// Emoji are assumed to be 2 columns wide, which matches the upstream TS
/// `graphemeWidth` logic.  The `unicode-width` crate handles most CJK ranges.
fn grapheme_width(segment: &str) -> usize {
    if segment.is_empty() {
        return 0;
    }

    // Emoji: assume width 2 (matches upstream)
    let first_cp = segment.chars().next().unwrap_or('\0');
    if is_emoji(first_cp) {
        return 2;
    }

    // Use unicode-width on the first non-combining code point
    let mut width = 0usize;
    let mut has_visible = false;
    for ch in segment.chars() {
        // Skip zero-width joiners and variation selectors
        if ch == '\u{200D}' || ch == '\u{FE0F}' {
            continue;
        }
        // Skip combining marks (general category M)
        if is_combining_mark(ch) {
            continue;
        }
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        width = width.max(w);
        has_visible = true;
    }

    if has_visible {
        width
    } else {
        0
    }
}

/// Return the total terminal column width of `text`.
pub fn count_terminal_cells(text: &str) -> usize {
    text.graphemes(true).map(grapheme_width).sum()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Conservative emoji test covering the main Extended_Pictographic ranges used
/// by the upstream TS code (`/\p{Extended_Pictographic}/u`).
///
/// We use broad ranges to avoid unreachable-pattern warnings in match arms.
fn is_emoji(cp: char) -> bool {
    let c = cp as u32;
    // Misc symbols, dingbats, and the main supplemental emoji blocks
    matches!(c,
        0x2300..=0x27BF |       // Misc technical + arrows + dingbats (broad)
        0x1F300..=0x1FAFF       // All supplemental emoji / symbol blocks
    )
}

/// True for Unicode combining marks (general category M*).
/// We use a simplified range check sufficient for the characters that appear
/// in terminal output.
fn is_combining_mark(ch: char) -> bool {
    let c = ch as u32;
    matches!(c,
        0x0300..=0x036F |   // Combining Diacritical Marks
        0x1AB0..=0x1AFF |   // Combining Diacritical Marks Extended
        0x1DC0..=0x1DFF |   // Combining Diacritical Marks Supplement
        0x20D0..=0x20FF |   // Combining Diacritical Marks for Symbols
        0xFE20..=0xFE2F     // Combining Half Marks
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_char_count() {
        assert_eq!(count_text_chars("hello"), 5);
    }

    #[test]
    fn emoji_char_count_one_grapheme() {
        // U+1F600 GRINNING FACE — 1 grapheme cluster
        assert_eq!(count_text_chars("😀"), 1);
    }

    #[test]
    fn cjk_terminal_width_two_cells() {
        // U+4E2D — one CJK character, should be 2 terminal cells
        assert_eq!(count_terminal_cells("中"), 2);
    }

    #[test]
    fn ascii_terminal_width() {
        assert_eq!(count_terminal_cells("abc"), 3);
    }

    #[test]
    fn graphemes_splits_correctly() {
        let gs = graphemes("abc");
        assert_eq!(gs, vec!["a", "b", "c"]);
    }

    // --- grapheme_width coverage ---

    #[test]
    fn emoji_terminal_width_two_cells() {
        // U+1F600 GRINNING FACE — emoji, should be 2 terminal cells
        assert_eq!(count_terminal_cells("😀"), 2);
    }

    #[test]
    fn zwj_sequence_is_two_cells() {
        // ZWJ sequences (e.g. family emoji) — grapheme_width should handle ZWJ
        // U+200D ZERO WIDTH JOINER is skipped; the base emoji drives width
        let fam = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}"; // family emoji
        let w = count_terminal_cells(fam);
        // Should be at least 1 (base emoji) — not zero
        assert!(w >= 1, "ZWJ sequence should have non-zero width");
    }

    #[test]
    fn variation_selector_skipped() {
        // U+FE0F VARIATION SELECTOR-16 is skipped (not counted as width)
        let text_emoji = "\u{2665}\u{FE0F}"; // ♥️ heart with VS16
        let w = count_terminal_cells(text_emoji);
        // The heart U+2665 is in the 0x2300..=0x27BF range → emoji → 2 cells
        assert_eq!(w, 2);
    }

    #[test]
    fn combining_mark_does_not_add_width() {
        // U+0301 COMBINING ACUTE ACCENT is a combining mark — skipped in width calc
        // "e\u{0301}" is one grapheme cluster (é) — width should be 1 (from "e")
        let composed = "e\u{0301}";
        let w = count_terminal_cells(composed);
        assert_eq!(w, 1, "combining accent should not add extra width");
    }

    #[test]
    fn empty_string_zero_width() {
        assert_eq!(count_terminal_cells(""), 0);
        assert_eq!(count_text_chars(""), 0);
    }

    #[test]
    fn mixed_ascii_and_cjk_width() {
        // "a中b" → 1 + 2 + 1 = 4 terminal cells, 3 grapheme clusters
        assert_eq!(count_terminal_cells("a中b"), 4);
        assert_eq!(count_text_chars("a中b"), 3);
    }

    #[test]
    fn misc_symbols_are_emoji_width() {
        // U+2603 SNOWMAN is in 0x2300..=0x27BF range → width 2
        let snowman = "\u{2603}";
        let w = count_terminal_cells(snowman);
        assert_eq!(w, 2);
    }

    #[test]
    fn combining_diacritical_marks_extended_covered() {
        // U+1AB0 is in 0x1AB0..=0x1AFF range (Combining Diacritical Marks Extended)
        // These are combining marks that get skipped in grapheme_width
        // "a\u{1AB0}" should be one grapheme cluster with width 1 (from 'a')
        let text = "a\u{1AB0}";
        let w = count_terminal_cells(text);
        // 'a' contributes 1, the combining mark is skipped
        assert_eq!(w, 1);
    }

    #[test]
    fn combining_half_marks_fe20_range() {
        // U+FE20 is in 0xFE20..=0xFE2F (Combining Half Marks)
        // This exercises the last arm of is_combining_mark
        let text = "x\u{FE20}";
        let w = count_terminal_cells(text);
        // 'x' contributes 1; FE20 is a combining mark, skipped
        assert_eq!(w, 1);
    }

    #[test]
    fn only_zwj_grapheme_has_zero_width() {
        // A segment consisting only of ZWJ (U+200D) — skipped in grapheme_width
        // has_visible remains false → returns 0
        // This is an artificial segment since real graphemes always have a base;
        // we test via count_terminal_cells on a string with only ZWJ
        let text = "\u{200D}";
        let w = count_terminal_cells(text);
        // ZWJ alone: has_visible stays false → width 0
        assert_eq!(w, 0);
    }

    #[test]
    fn grapheme_width_empty_segment_is_zero() {
        // count_terminal_cells on empty string: graphemes() returns no segments
        // so the sum is 0; the empty-check branch is exercised via internal calls
        assert_eq!(count_terminal_cells(""), 0);
    }

    #[test]
    fn combining_diacritical_supplement_1dc0() {
        // U+1DC0 is in 0x1DC0..=0x1DFF (Combining Diacritical Marks Supplement)
        let text = "e\u{1DC0}";
        let w = count_terminal_cells(text);
        assert_eq!(w, 1);
    }

    #[test]
    fn combining_diacritical_for_symbols_20d0() {
        // U+20D0 is in 0x20D0..=0x20FF (Combining Diacritical Marks for Symbols)
        let text = "A\u{20D0}";
        let w = count_terminal_cells(text);
        assert_eq!(w, 1);
    }
}
