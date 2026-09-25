use super::*;

/// Convert solang parser diagnostics into structured [`ParseDiagnostic`]s,
/// preserving each diagnostic's byte range and prefixing the message with
/// `line:column:` so standard-JSON output can emit one error per diagnostic
/// with a precise `sourceLocation`.
pub(crate) fn collect_parse_diagnostics(
    source: &str,
    diagnostics: &[Diagnostic],
) -> Vec<ParseDiagnostic> {
    diagnostics
        .iter()
        .map(|diag| {
            if let solang_parser::pt::Loc::File(_, start, end) = diag.loc {
                let (line, column) = offset_to_line_column(source, start);
                ParseDiagnostic {
                    message: format!("{}:{}: {}", line, column, diag.message),
                    start,
                    end,
                }
            } else {
                ParseDiagnostic {
                    message: diag.message.clone(),
                    start: 0,
                    end: 0,
                }
            }
        })
        .collect()
}

fn offset_to_line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut column = 1usize;
    let mut current = 0usize;

    for ch in source.chars() {
        if current >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }

        current += ch.len_utf8();
    }

    (line, column)
}
