use super::*;

pub(crate) fn enforce_supported_pragma(
    pragma: &solang_parser::pt::PragmaDirective,
) -> Result<Option<Version>, FrontendError> {
    use solang_parser::pt::PragmaDirective;

    let PragmaDirective::Version(_, ident, comparators) = pragma else {
        return Ok(None);
    };

    if ident.name != "solidity" {
        return Ok(None);
    }

    let spec = comparators
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ");

    // Compiler compatibility targets mainstream modern Solidity ranges used by
    // upstream protocols. We accept pragmas that intersect 0.5.x through 0.8.x.
    if pragma_supports_neo_devpack_solidity(spec.as_str()) {
        Ok(pragma_min_version(spec.as_str()))
    } else {
        Err(FrontendError::UnsupportedVersion(spec))
    }
}

/// Compute the pragma's lowest-allowed concrete Solidity version.
///
/// Used to enforce per-feature version gates (e.g. `string.concat` requires
/// `>= 0.8.12`). When the pragma admits multiple OR-branches, we pick the
/// smallest lower bound since any of those versions may be used at compile
/// time. Returns `None` for unbounded or unparseable ranges.
pub(crate) fn pragma_min_version(spec: &str) -> Option<Version> {
    let normalized = spec.replace(' ', "").to_lowercase();
    if normalized.is_empty() {
        return None;
    }

    let mut best: Option<Version> = None;
    for branch in normalized.split("||") {
        let Some(v) = branch_min_version(branch) else {
            continue;
        };
        best = match best {
            Some(existing) if existing <= v => Some(existing),
            _ => Some(v),
        };
    }
    best
}

pub(crate) fn branch_min_version(branch: &str) -> Option<Version> {
    let comparators = split_comparators(branch);
    let mut lower: Option<Version> = None;
    let mut update = |candidate: Version| {
        lower = match lower {
            Some(existing) if existing >= candidate => Some(existing),
            _ => Some(candidate),
        };
    };

    for comparator in comparators {
        if comparator == "*" {
            continue;
        }
        if let Some((start, _)) = parse_hyphen_range(&comparator) {
            update(start);
            continue;
        }
        if let Some((version, _)) = parse_caret(&comparator) {
            update(version);
            continue;
        }
        if let Some(version) = parse_tilde(&comparator) {
            update(version);
            continue;
        }
        if let Some((op, version)) = parse_operator_version(&comparator) {
            match op {
                ComparatorOp::Greater => update(next_patch(version)),
                ComparatorOp::GreaterEq | ComparatorOp::Exact => update(version),
                _ => {}
            }
            continue;
        }
        if let Some(version) = parse_plain_version(&comparator) {
            update(version);
        }
    }
    lower
}

/// Feature registry: features that require a minimum Solidity version.
///
/// POC covers `string.concat`/`bytes.concat` (both introduced in 0.8.12 /
/// 0.8.4 respectively). Extend this as more features are gated.
const FEATURE_STRING_CONCAT_MIN: Version = Version {
    major: 0,
    minor: 8,
    patch: 12,
};
const FEATURE_BYTES_CONCAT_MIN: Version = Version {
    major: 0,
    minor: 8,
    patch: 4,
};

/// Reject feature uses that predate the pragma's declared minimum version.
///
/// Scans `source` for `string.concat(` / `bytes.concat(` tokens (outside of
/// comments and string literals) and errors when `pragma_min` is below the
/// feature's introduction version. Matches solc's "feature unavailable in
/// declared pragma range" behavior for the most common gap.
pub(crate) fn enforce_feature_version_gates(
    source: &str,
    pragma_min: Option<Version>,
) -> Result<(), FrontendError> {
    let Some(min) = pragma_min else {
        return Ok(());
    };

    let stripped = strip_comments_and_strings(source);

    if min < FEATURE_STRING_CONCAT_MIN && contains_builtin_call(&stripped, "string.concat(") {
        return Err(FrontendError::Parse(format!(
            "feature `string.concat` requires pragma >= 0.8.12; declared pragma allows {}.{}.{}",
            min.major, min.minor, min.patch
        )));
    }
    if min < FEATURE_BYTES_CONCAT_MIN && contains_builtin_call(&stripped, "bytes.concat(") {
        return Err(FrontendError::Parse(format!(
            "feature `bytes.concat` requires pragma >= 0.8.4; declared pragma allows {}.{}.{}",
            min.major, min.minor, min.patch
        )));
    }
    Ok(())
}

/// True when `needle` (e.g. `"string.concat("`) appears as a real builtin call
/// rather than as a suffix of a larger identifier — i.e. the character
/// immediately before it is not part of an identifier (`a-zA-Z0-9_`). Without
/// this a user variable like `myString.concat(...)` would falsely trip the
/// pragma-feature gate.
pub(crate) fn contains_builtin_call(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    let mut start = 0usize;
    while let Some(pos) = haystack[start..].find(needle) {
        let abs = start + pos;
        let boundary_ok = abs == 0 || {
            let prev = bytes[abs - 1];
            !(prev.is_ascii_alphanumeric() || prev == b'_')
        };
        if boundary_ok {
            return true;
        }
        start = abs + 1;
    }
    false
}

/// Lightweight lexer-aware strip: replaces string-literal and comment bodies
/// with spaces so they cannot produce false positives for feature scans.
/// Handles `//` line comments, `/* */` block comments, `"..."` and `'...'`
/// string literals with backslash escapes. Adequate for identifier-level
/// feature probing (not a full Solidity lexer).
pub(crate) fn strip_comments_and_strings(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            out.push_str("  ");
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                out.push(' ');
                i += 1;
            }
            if i + 1 < bytes.len() {
                out.push_str("  ");
                i += 2;
            }
            continue;
        }
        if c == b'"' || c == b'\'' {
            let quote = c;
            out.push(' ');
            i += 1;
            while i < bytes.len() && bytes[i] != quote {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    out.push_str("  ");
                    i += 2;
                    continue;
                }
                out.push(' ');
                i += 1;
            }
            if i < bytes.len() {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        out.push(c as char);
        i += 1;
    }
    out
}

pub(crate) fn pragma_supports_neo_devpack_solidity(spec: &str) -> bool {
    let normalized = spec.replace(' ', "").to_lowercase();

    if normalized.is_empty() {
        return true;
    }

    // Accept if any OR-branch can include a supported compiler range.
    normalized
        .split("||")
        .any(branch_supports_neo_devpack_solidity)
}

pub(crate) fn branch_supports_neo_devpack_solidity(branch: &str) -> bool {
    if branch.is_empty() {
        return false;
    }

    let comparators = split_comparators(branch);
    if comparators.is_empty() {
        return false;
    }

    let mut lower = Bound::Unbounded;
    let mut upper = Bound::Unbounded;

    for comparator in comparators {
        if comparator == "*" {
            continue;
        }

        if let Some((start, end)) = parse_hyphen_range(&comparator) {
            lower = lower.max(Bound::Inclusive(start));
            upper = upper.min(Bound::Inclusive(end));
            continue;
        }

        if let Some((version, level)) = parse_caret(&comparator) {
            let upper_version = match level {
                0 => Version {
                    major: version.major.saturating_add(1),
                    minor: 0,
                    patch: 0,
                },
                _ => Version {
                    major: version.major,
                    minor: version.minor.saturating_add(1),
                    patch: 0,
                },
            };
            lower = lower.max(Bound::Inclusive(version));
            upper = upper.min(Bound::Exclusive(upper_version));
            continue;
        }

        if let Some(version) = parse_tilde(&comparator) {
            let upper_version = Version {
                major: version.major,
                minor: version.minor.saturating_add(1),
                patch: 0,
            };
            lower = lower.max(Bound::Inclusive(version));
            upper = upper.min(Bound::Exclusive(upper_version));
            continue;
        }

        if let Some((op, version)) = parse_operator_version(&comparator) {
            match op {
                ComparatorOp::Greater => lower = lower.max(Bound::Exclusive(version)),
                ComparatorOp::GreaterEq => lower = lower.max(Bound::Inclusive(version)),
                ComparatorOp::Less => upper = upper.min(Bound::Exclusive(version)),
                ComparatorOp::LessEq => upper = upper.min(Bound::Inclusive(version)),
                ComparatorOp::Exact => {
                    lower = lower.max(Bound::Inclusive(version));
                    upper = upper.min(Bound::Inclusive(version));
                }
            }
            continue;
        }

        if let Some(version) = parse_plain_version(&comparator) {
            lower = lower.max(Bound::Inclusive(version));
            upper = upper.min(Bound::Inclusive(version));
            continue;
        }

        // Unknown comparator format: reject conservatively.
        return false;
    }

    intersects_supported_neo_range(lower, upper)
}
