//! Line-level text processing utilities.
//!
//! Port of the processing functions in `src/core/text.ts`.

use super::width::count_text_chars;
use unicode_segmentation::UnicodeSegmentation;

const TRUNCATION_SUFFIX: &str = "\n... truncated ...";
const MIDDLE_TRUNCATION_MARKER: &str = "\n... omitted ...\n";

// ---------------------------------------------------------------------------
// Line normalization
// ---------------------------------------------------------------------------

/// Split text into lines, normalising CRLF and stripping trailing whitespace
/// per line (mirrors `normalizeLines` in TS).
pub fn normalize_lines(text: &str) -> Vec<String> {
    text.replace("\r\n", "\n")
        .split('\n')
        .map(|line| line.trim_end().to_owned())
        .collect()
}

// ---------------------------------------------------------------------------
// Edge trimming
// ---------------------------------------------------------------------------

/// Remove empty lines from the start and end of a line slice.
pub fn trim_empty_edges(lines: &[String]) -> Vec<String> {
    let start = lines
        .iter()
        .position(|l| !l.trim().is_empty())
        .unwrap_or(lines.len());
    let end = lines
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .map(|i| i + 1)
        .unwrap_or(0);
    if start >= end {
        return Vec::new();
    }
    lines[start..end].to_vec()
}

// ---------------------------------------------------------------------------
// Deduplication
// ---------------------------------------------------------------------------

/// Remove adjacent duplicate lines (keeps first occurrence).
pub fn dedupe_adjacent(lines: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    for line in lines {
        if out.last().map(|l: &String| l.as_str()) != Some(line.as_str()) {
            out.push(line.clone());
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Head / tail summarisation
// ---------------------------------------------------------------------------

/// Keep the first `head` lines, an omission marker, and the last `tail` lines.
/// If `lines.len() <= head + tail`, returns `lines` unchanged.
pub fn head_tail(lines: &[String], head: usize, tail: usize) -> Vec<String> {
    if lines.len() <= head + tail {
        return lines.to_vec();
    }
    let omitted = lines.len() - head - tail;
    let mut out = Vec::with_capacity(head + 1 + tail);
    out.extend_from_slice(&lines[..head]);
    out.push(format!("... {} lines omitted ...", omitted));
    out.extend_from_slice(&lines[lines.len() - tail..]);
    out
}

// ---------------------------------------------------------------------------
// Clamping
// ---------------------------------------------------------------------------

/// Trim `text` at the last newline that is at or before position 50% through
/// the text (mirrors `trimHeadToLineBoundary` in TS).
fn trim_head_to_line_boundary(text: &str) -> &str {
    let last_nl = text.rfind('\n');
    match last_nl {
        None => text,
        Some(pos) => {
            if pos < text.len() / 2 {
                text
            } else {
                &text[..pos]
            }
        }
    }
}

/// Trim `text` at the first newline that is at or after position 50% through
/// (mirrors `trimTailToLineBoundary` in TS).
fn trim_tail_to_line_boundary(text: &str) -> &str {
    let first_nl = text.find('\n');
    match first_nl {
        None => text,
        Some(pos) => {
            if pos > text.len().div_ceil(2) {
                text
            } else {
                &text[pos + 1..]
            }
        }
    }
}

/// Clamp `text` to at most `max_chars` grapheme clusters (tail-truncate).
pub fn clamp_text(text: &str, max_chars: usize) -> String {
    if count_text_chars(text) <= max_chars {
        return text.to_owned();
    }
    let suffix_chars = count_text_chars(TRUNCATION_SUFFIX);
    let body_chars = max_chars.saturating_sub(suffix_chars);
    let segs: Vec<&str> = text.graphemes(true).collect();
    let head: String = segs[..body_chars.min(segs.len())].concat();
    let head = trim_head_to_line_boundary(&head);
    format!("{}{}", head, TRUNCATION_SUFFIX)
}

/// Clamp `text` to at most `max_chars` grapheme clusters using middle-truncation.
/// Keeps 70% from the head and 30% from the tail.
pub fn clamp_text_middle(text: &str, max_chars: usize) -> String {
    if count_text_chars(text) <= max_chars {
        return text.to_owned();
    }
    let marker_chars = count_text_chars(MIDDLE_TRUNCATION_MARKER);
    let body_chars = max_chars.saturating_sub(marker_chars);
    let head_chars = (body_chars as f64 * 0.7).ceil() as usize;
    let tail_chars = body_chars.saturating_sub(head_chars);

    let segs: Vec<&str> = text.graphemes(true).collect();
    let total = segs.len();

    let head_raw: String = segs[..head_chars.min(total)].concat();
    let head = trim_head_to_line_boundary(&head_raw).to_owned();

    let tail_raw: String = segs[total.saturating_sub(tail_chars)..].concat();
    let tail = trim_tail_to_line_boundary(&tail_raw).to_owned();

    format!("{}{}{}", head, MIDDLE_TRUNCATION_MARKER, tail)
}

// ---------------------------------------------------------------------------
// Pluralize
// ---------------------------------------------------------------------------

/// English pluralization matching the upstream `pluralize` function exactly.
pub fn pluralize(count: usize, noun: &str) -> String {
    // If noun already ends in "passed", "failed", "skipped" — no change
    if noun.ends_with("passed") || noun.ends_with("failed") || noun.ends_with("skipped") {
        return format!("{} {}", count, noun);
    }
    if count == 1 {
        return format!("{} {}", count, noun);
    }
    if noun.ends_with('s')
        || noun.ends_with('x')
        || noun.ends_with('z')
        || noun.ends_with("sh")
        || noun.ends_with("ch")
    {
        return format!("{} {}es", count, noun);
    }
    // [^aeiou]y → -ies
    let ends_consonant_y = noun.ends_with('y')
        && noun.len() >= 2
        && !matches!(
            noun.chars().nth(noun.len() - 2),
            Some('a' | 'e' | 'i' | 'o' | 'u')
        );
    if ends_consonant_y {
        let stem = &noun[..noun.len() - 1];
        return format!("{} {}ies", count, stem);
    }
    format!("{} {}s", count, noun)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- normalize_lines ---

    #[test]
    fn normalize_crlf() {
        assert_eq!(normalize_lines("a\r\nb"), vec!["a", "b"]);
    }

    #[test]
    fn normalize_strips_trailing_space() {
        assert_eq!(normalize_lines("a   "), vec!["a"]);
    }

    // --- trim_empty_edges ---

    #[test]
    fn trim_edges_removes_blanks() {
        let lines: Vec<String> = vec!["", "a", "b", ""]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(trim_empty_edges(&lines), vec!["a", "b"]);
    }

    #[test]
    fn trim_edges_all_blank() {
        let lines: Vec<String> = vec!["", ""].iter().map(|s| s.to_string()).collect();
        assert!(trim_empty_edges(&lines).is_empty());
    }

    // --- dedupe_adjacent ---

    #[test]
    fn dedupe_keeps_non_adjacent() {
        let lines = vec!["a", "a", "b", "a"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        assert_eq!(dedupe_adjacent(&lines), vec!["a", "b", "a"]);
    }

    // --- head_tail ---

    #[test]
    fn head_tail_short_passthrough() {
        let lines: Vec<String> = (0..5).map(|i| format!("{}", i)).collect();
        assert_eq!(head_tail(&lines, 3, 3), lines);
    }

    #[test]
    fn head_tail_omits_middle() {
        let lines: Vec<String> = (0..10).map(|i| format!("{}", i)).collect();
        let result = head_tail(&lines, 3, 3);
        assert_eq!(result.len(), 7); // 3 + marker + 3
        assert!(result[3].contains("4 lines omitted"));
    }

    // --- clamp_text ---

    #[test]
    fn clamp_text_passthrough_short() {
        assert_eq!(clamp_text("hi", 100), "hi");
    }

    #[test]
    fn clamp_text_truncates() {
        let long_text = "a".repeat(2000);
        let clamped = clamp_text(&long_text, 100);
        assert!(count_text_chars(&clamped) <= 100 + count_text_chars(TRUNCATION_SUFFIX));
        assert!(clamped.ends_with("... truncated ..."));
    }

    // --- clamp_text_middle ---

    #[test]
    fn clamp_middle_passthrough_short() {
        assert_eq!(clamp_text_middle("hi", 100), "hi");
    }

    #[test]
    fn clamp_middle_contains_marker() {
        let long_text = "a\n".repeat(200);
        let clamped = clamp_text_middle(&long_text, 50);
        assert!(
            clamped.contains("... omitted ..."),
            "missing marker in: {}",
            clamped
        );
    }

    // --- pluralize ---

    #[test]
    fn pluralize_regular() {
        assert_eq!(pluralize(2, "error"), "2 errors");
    }

    #[test]
    fn pluralize_singular() {
        assert_eq!(pluralize(1, "error"), "1 error");
    }

    #[test]
    fn pluralize_sibilant() {
        assert_eq!(pluralize(2, "match"), "2 matches");
    }

    #[test]
    fn pluralize_y_ending() {
        assert_eq!(pluralize(2, "entry"), "2 entries");
    }

    #[test]
    fn pluralize_already_ended() {
        assert_eq!(pluralize(3, "passed"), "3 passed");
    }

    #[test]
    fn pluralize_failed_noun() {
        assert_eq!(pluralize(2, "failed"), "2 failed");
    }

    #[test]
    fn pluralize_skipped_noun() {
        assert_eq!(pluralize(0, "skipped"), "0 skipped");
    }

    // --- trim_head_to_line_boundary edge cases ---

    #[test]
    fn clamp_text_no_newline_in_head() {
        // When there's no newline in the head portion, clamp_text still truncates
        // This exercises the "None" branch of trim_head_to_line_boundary
        let text = "a".repeat(200); // no newlines
        let clamped = clamp_text(&text, 50);
        assert!(clamped.ends_with("... truncated ..."));
    }

    #[test]
    fn clamp_text_newline_at_early_position() {
        // Newline at position < len/2 → trim_head_to_line_boundary returns text as-is
        // (the newline is too early to use as a boundary)
        let text = "ab\n".to_owned() + &"x".repeat(200);
        let clamped = clamp_text(&text, 100);
        assert!(clamped.ends_with("... truncated ..."));
    }

    #[test]
    fn clamp_middle_no_newline_in_tail() {
        // tail portion has no newline → trim_tail_to_line_boundary returns text as-is
        // This exercises the "None" branch of trim_tail_to_line_boundary
        let text = "line1\nline2\n".to_owned() + &"x".repeat(300);
        let clamped = clamp_text_middle(&text, 40);
        assert!(clamped.contains("... omitted ..."));
    }

    #[test]
    fn clamp_middle_newline_at_late_position() {
        // Newline at position > len.div_ceil(2) → returns text as-is in trim_tail
        // Build tail where the first newline is very late
        let text = "line1\nline2\nline3\n".repeat(50);
        let clamped = clamp_text_middle(&text, 80);
        assert!(clamped.contains("... omitted ..."));
    }

    #[test]
    fn clamp_middle_tail_newline_in_second_half() {
        // Force trim_tail_to_line_boundary to hit the "pos > len/2" branch:
        // The tail raw string must have its first newline past the midpoint.
        // We need a large body so the tail portion (30%) starts with many chars
        // before the first newline.
        // "xxxxxxxx\nyyyyyyy" where \n is at position > midpoint
        // Construct text with many lines; the last chunk has no early newline
        let many_lines: String = "head-line\n".repeat(100);
        // Tail segment ends with long non-newline text followed by newline at end
        let text = many_lines + &"z".repeat(200) + "\nlast";
        let clamped = clamp_text_middle(&text, 300);
        // Should produce output with the marker
        assert!(clamped.contains("... omitted ..."));
    }

    // --- head_tail edge cases ---

    #[test]
    fn head_tail_exact_boundary() {
        // lines.len() == head + tail → passthrough (not truncated)
        let lines: Vec<String> = (0..6).map(|i| format!("line{}", i)).collect();
        let result = head_tail(&lines, 3, 3);
        assert_eq!(result, lines, "exact head+tail should not truncate");
    }

    // --- dedupe_adjacent empty input ---

    #[test]
    fn dedupe_adjacent_empty() {
        assert!(dedupe_adjacent(&[]).is_empty());
    }

    // --- normalize_lines with no trailing whitespace ---

    #[test]
    fn normalize_lines_no_crlf() {
        let lines = normalize_lines("a\nb\nc");
        assert_eq!(lines, vec!["a", "b", "c"]);
    }
}
