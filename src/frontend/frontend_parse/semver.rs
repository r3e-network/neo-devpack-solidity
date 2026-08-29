//! Solidity version / semver helpers used by the parser.

/// Canonical Solidity version range supported by this compiler.
///
/// Keep this in sync with the TypeScript tooling constant in
/// `tooling/packages/types/src/compiler.ts`. Marked `#[allow(dead_code)]`
/// because it is exported for external tooling and documentation rather than
/// consumed inside this crate; `MIN_SUPPORTED_SOLIDITY_VERSION` and
/// `MAX_EXCLUDED_SOLIDITY_VERSION` are used for the actual range checks.
#[allow(dead_code)]
pub const SUPPORTED_SOLIDITY_RANGE: &str = ">=0.8.19 <0.8.28";

/// Minimum supported Solidity version (inclusive).
pub const MIN_SUPPORTED_SOLIDITY_VERSION: &str = "0.8.19";

/// Maximum excluded Solidity version (exclusive).
pub const MAX_EXCLUDED_SOLIDITY_VERSION: &str = "0.8.28";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Version {
    pub(crate) major: u64,
    pub(crate) minor: u64,
    pub(crate) patch: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Bound {
    Unbounded,
    Inclusive(Version),
    Exclusive(Version),
}

impl Bound {
    /// Combine two bounds. `same_cmp` decides between two bounds of the
    /// same variant (`Inclusive/Inclusive`, `Exclusive/Exclusive`);
    /// `cross_cmp` decides between the mixed pair (`Inclusive` vs
    /// `Exclusive`). The mixed pair's equal case always returns
    /// `Exclusive(a)` since `Exclusive` is more restrictive than
    /// `Inclusive` at the same point (the set `{x | x > a}` is strictly
    /// smaller than `{x | x >= a}`).
    ///
    /// `max` passes `(>=, >)` — picks the more restrictive lower bound
    /// (the side with the LARGER value), used when intersecting two
    /// supported-version ranges.
    /// `min` passes `(<=, <)` — picks the more restrictive upper bound
    /// (the side with the SMALLER value).
    fn combine(
        self,
        other: Self,
        same_cmp: impl Fn(&Version, &Version) -> bool,
        cross_cmp: impl Fn(&Version, &Version) -> bool,
    ) -> Self {
        use Bound::{Exclusive, Inclusive, Unbounded};
        match (self, other) {
            (Unbounded, x) | (x, Unbounded) => x,
            (Inclusive(a), Inclusive(b)) => {
                if same_cmp(&a, &b) {
                    Inclusive(a)
                } else {
                    Inclusive(b)
                }
            }
            (Exclusive(a), Exclusive(b)) => {
                if same_cmp(&a, &b) {
                    Exclusive(a)
                } else {
                    Exclusive(b)
                }
            }
            (Inclusive(a), Exclusive(b)) => {
                if cross_cmp(&a, &b) {
                    Inclusive(a)
                } else if cross_cmp(&b, &a) {
                    Exclusive(b)
                } else {
                    Exclusive(a)
                }
            }
            (Exclusive(a), Inclusive(b)) => {
                if cross_cmp(&a, &b) {
                    Exclusive(a)
                } else if cross_cmp(&b, &a) {
                    Inclusive(b)
                } else {
                    Exclusive(a)
                }
            }
        }
    }

    pub(crate) fn max(self, other: Self) -> Self {
        self.combine(other, |a, b| a >= b, |a, b| a > b)
    }

    pub(crate) fn min(self, other: Self) -> Self {
        self.combine(other, |a, b| a <= b, |a, b| a < b)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ComparatorOp {
    Greater,
    GreaterEq,
    Less,
    LessEq,
    Exact,
}

pub(crate) fn split_comparators(branch: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = branch.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch == ',' {
            i += 1;
            continue;
        }

        if ch == '^' || ch == '~' {
            let mut token = String::new();
            token.push(ch);
            i += 1;
            while i < chars.len() {
                let c = chars[i];
                if c == ',' || c == '^' || c == '~' || c == '<' || c == '>' || c == '=' {
                    break;
                }
                token.push(c);
                i += 1;
            }
            tokens.push(token);
            continue;
        }

        if ch == '<' || ch == '>' || ch == '=' {
            let mut token = String::new();
            token.push(ch);
            i += 1;
            if i < chars.len() && chars[i] == '=' {
                token.push('=');
                i += 1;
            }
            while i < chars.len() {
                let c = chars[i];
                if c == ',' || c == '^' || c == '~' || c == '<' || c == '>' || c == '=' {
                    break;
                }
                token.push(c);
                i += 1;
            }
            tokens.push(token);
            continue;
        }

        // Plain version or hyphen range.
        let mut token = String::new();
        while i < chars.len() {
            let c = chars[i];
            if c == ',' || c == '^' || c == '~' || c == '<' || c == '>' || c == '=' {
                break;
            }
            token.push(c);
            i += 1;
        }
        if !token.is_empty() {
            tokens.push(token);
        }
    }

    tokens
}

pub(crate) fn parse_hyphen_range(comparator: &str) -> Option<(Version, Version)> {
    let (left, right) = comparator.split_once('-')?;
    let start = parse_plain_version(left)?;
    let end = parse_plain_version(right)?;
    Some((start, end))
}

pub(crate) fn parse_caret(comparator: &str) -> Option<(Version, u8)> {
    let raw = comparator.strip_prefix('^')?;
    let dots = raw.matches('.').count() as u8;
    let version = parse_plain_version(raw)?;
    Some((version, dots))
}

pub(crate) fn parse_tilde(comparator: &str) -> Option<Version> {
    let raw = comparator.strip_prefix('~')?;
    parse_plain_version(raw)
}

pub(crate) fn parse_operator_version(comparator: &str) -> Option<(ComparatorOp, Version)> {
    if let Some(raw) = comparator.strip_prefix(">=") {
        return parse_plain_version(raw).map(|v| (ComparatorOp::GreaterEq, v));
    }
    if let Some(raw) = comparator.strip_prefix("<=") {
        return parse_plain_version(raw).map(|v| (ComparatorOp::LessEq, v));
    }
    if let Some(raw) = comparator.strip_prefix('>') {
        return parse_plain_version(raw).map(|v| (ComparatorOp::Greater, v));
    }
    if let Some(raw) = comparator.strip_prefix('<') {
        return parse_plain_version(raw).map(|v| (ComparatorOp::Less, v));
    }
    if let Some(raw) = comparator.strip_prefix('=') {
        return parse_plain_version(raw).map(|v| (ComparatorOp::Exact, v));
    }
    None
}

pub(crate) fn parse_plain_version(raw: &str) -> Option<Version> {
    if raw.is_empty() || raw == "*" {
        return None;
    }

    let mut parts = raw.split('.');
    let major_raw = parts.next()?;
    let minor_raw = parts.next().unwrap_or("0");
    let patch_raw = parts.next().unwrap_or("0");

    if parts.next().is_some() {
        return None;
    }

    // Allow wildcard suffixes like 0.8.* or 0.8.x as "any patch".
    let major = major_raw.parse::<u64>().ok()?;
    let minor = if minor_raw == "*" || minor_raw == "x" {
        0
    } else {
        minor_raw.parse::<u64>().ok()?
    };
    let patch = if patch_raw == "*" || patch_raw == "x" {
        0
    } else {
        patch_raw.parse::<u64>().ok()?
    };

    Some(Version {
        major,
        minor,
        patch,
    })
}

pub(crate) fn intersects_supported_neo_range(lower: Bound, upper: Bound) -> bool {
    let min = parse_plain_version(MIN_SUPPORTED_SOLIDITY_VERSION)
        .expect("MIN_SUPPORTED_SOLIDITY_VERSION must be valid");
    let max = parse_plain_version(MAX_EXCLUDED_SOLIDITY_VERSION)
        .expect("MAX_EXCLUDED_SOLIDITY_VERSION must be valid");
    intersects_semver_window(lower, upper, min, max)
}

pub(crate) fn intersects_semver_window(
    lower: Bound,
    upper: Bound,
    target_start: Version,
    target_end_exclusive: Version,
) -> bool {
    let effective_start = match lower {
        Bound::Unbounded => target_start,
        Bound::Inclusive(v) => v,
        Bound::Exclusive(v) => next_patch(v),
    };

    let effective_end_exclusive = match upper {
        Bound::Unbounded => target_end_exclusive,
        Bound::Inclusive(v) => next_patch(v),
        Bound::Exclusive(v) => v,
    };

    let range_start = if effective_start > target_start {
        effective_start
    } else {
        target_start
    };
    let range_end = if effective_end_exclusive < target_end_exclusive {
        effective_end_exclusive
    } else {
        target_end_exclusive
    };

    range_start < range_end
}

pub(crate) fn next_patch(version: Version) -> Version {
    Version {
        major: version.major,
        minor: version.minor,
        patch: version.patch.saturating_add(1),
    }
}

/// Advance `pos` over ASCII whitespace in `bytes`, returning the index of the
/// next non-whitespace byte (or the end of input).
pub(crate) fn skip_whitespace_forward(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    pos
}
