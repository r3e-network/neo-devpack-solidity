use super::constants::MAX_SOURCE_LENGTH;
use std::borrow::Cow;

fn clamp_utf8<'a>(value: &'a str, max_len: usize) -> Cow<'a, str> {
    if value.len() <= max_len {
        return Cow::Borrowed(value);
    }

    let mut end = max_len.min(value.len());
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    Cow::Owned(value[..end].to_string())
}

/// Clamp a NEF source string to the maximum allowed byte length, preserving
/// UTF-8 boundaries.
pub fn clamp_nef_source<'a>(value: &'a str) -> Cow<'a, str> {
    clamp_utf8(value, MAX_SOURCE_LENGTH)
}

/// Clamp a NEF source string and report whether truncation occurred.
pub fn clamp_nef_source_with_flag<'a>(value: &'a str) -> (Cow<'a, str>, bool) {
    let clamped = clamp_nef_source(value);
    let truncated = clamped.len() < value.len();
    (clamped, truncated)
}
