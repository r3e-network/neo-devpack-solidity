use super::*;

/// Build a map from a DECLARATION's start position to the Natspec block that
/// immediately precedes it. Each accumulated doc block is keyed at the first
/// non-whitespace byte AFTER it (i.e. the start of the declaration it
/// documents), so attachment is an exact lookup at the declaration's `start`
/// — independent of distance, and never bleeding across an intervening
/// declaration. A doc run is also broken when a non-whitespace token sits
/// between two doc comments (that earlier doc belongs to the intervening
/// declaration, not the later block).
pub(crate) fn build_comment_map(
    comments: &[Comment],
    source: &str,
) -> HashMap<usize, NatspecDocIR> {
    let mut map = HashMap::new();
    let bytes = source.as_bytes();
    // (end_of_last_doc_line, accumulated_text)
    let mut pending: Option<(usize, String)> = None;

    for comment in comments {
        match comment {
            Comment::DocLine(loc, text) | Comment::DocBlock(loc, text) => {
                if let Loc::File(_, start, end) = loc {
                    let clean_text = clean_doc_comment(text);
                    let continues = match &pending {
                        Some((prev_end, _)) => bytes
                            .get(*prev_end..*start)
                            .is_some_and(|gap| gap.iter().all(u8::is_ascii_whitespace)),
                        None => false,
                    };
                    if continues {
                        if let Some((prev_end, existing)) = pending.as_mut() {
                            *prev_end = *end;
                            existing.push('\n');
                            existing.push_str(&clean_text);
                        }
                    } else {
                        if let Some((prev_end, doc_text)) = pending.take() {
                            map.insert(
                                skip_whitespace_forward(bytes, prev_end),
                                parse_natspec(&doc_text),
                            );
                        }
                        pending = Some((*end, clean_text));
                    }
                }
            }
            Comment::Line(_loc, _) | Comment::Block(_loc, _) => {
                // Regular comments break doc comment sequences.
                if let Some((prev_end, doc_text)) = pending.take() {
                    map.insert(
                        skip_whitespace_forward(bytes, prev_end),
                        parse_natspec(&doc_text),
                    );
                }
            }
        }
    }

    if let Some((prev_end, doc_text)) = pending.take() {
        map.insert(
            skip_whitespace_forward(bytes, prev_end),
            parse_natspec(&doc_text),
        );
    }

    map
}

/// Remove comment delimiters and leading asterisks/slashes
pub(crate) fn clean_doc_comment(text: &str) -> String {
    text.lines()
        .map(|line| {
            let trimmed = line.trim();
            // Remove /// prefix from line doc comments
            if let Some(rest) = trimmed.strip_prefix("///") {
                rest.trim().to_string()
            // Remove /** and */ from block doc comments
            } else if let Some(rest) = trimmed.strip_prefix("/**") {
                rest.trim_end_matches("*/").trim().to_string()
            } else if let Some(rest) = trimmed.strip_suffix("*/") {
                rest.trim().to_string()
            // Remove leading * from block comment lines
            } else if let Some(rest) = trimmed.strip_prefix('*') {
                rest.trim().to_string()
            } else {
                trimmed.to_string()
            }
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse Natspec tags from a documentation comment
pub(crate) fn parse_natspec(text: &str) -> NatspecDocIR {
    let mut doc = NatspecDocIR::default();
    let mut current_tag: Option<&str> = None;
    let mut current_content = String::new();

    for line in text.lines() {
        let trimmed = line.trim();

        // Check for tag at start of line
        if trimmed.starts_with('@') {
            // Save previous tag content
            if let Some(tag) = current_tag {
                save_tag_content(&mut doc, tag, &current_content);
            }

            // Parse new tag
            let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
            current_tag = Some(parts[0]);
            current_content = parts
                .get(1)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
        } else if current_tag.is_some() {
            // Continue previous tag content
            if !current_content.is_empty() {
                current_content.push(' ');
            }
            current_content.push_str(trimmed);
        } else {
            // No tag yet - treat as @notice
            if doc.notice.is_none() && !trimmed.is_empty() {
                doc.notice = Some(trimmed.to_string());
            } else if let Some(ref mut notice) = doc.notice {
                notice.push(' ');
                notice.push_str(trimmed);
            }
        }
    }

    // Save final tag
    if let Some(tag) = current_tag {
        save_tag_content(&mut doc, tag, &current_content);
    }

    doc
}

pub(crate) fn save_tag_content(doc: &mut NatspecDocIR, tag: &str, content: &str) {
    let content = content.trim().to_string();
    if content.is_empty() {
        return;
    }

    match tag {
        "@title" => doc.title = Some(content),
        "@author" => doc.author = Some(content),
        "@notice" => doc.notice = Some(content),
        "@dev" => doc.dev = Some(content),
        "@param" => {
            // Format: @param name description
            let parts: Vec<&str> = content.splitn(2, char::is_whitespace).collect();
            if parts.len() >= 2 {
                doc.params
                    .push((parts[0].to_string(), parts[1].trim().to_string()));
            } else if !parts.is_empty() {
                doc.params.push((parts[0].to_string(), String::new()));
            }
        }
        "@return" => doc.returns.push(content),
        tag if tag.starts_with("@custom:") => {
            let custom_tag = tag.strip_prefix("@custom:").unwrap_or("");
            doc.custom.push((custom_tag.to_string(), content));
        }
        _ => {} // Ignore unknown tags
    }
}

/// Find the doc comment that precedes a given declaration. `build_comment_map`
/// keys each doc block at the start of the declaration it documents, so this is
/// an exact lookup at the declaration's `start` — no fixed-distance backward
/// scan (which mis-attached across intervening tokens or missed docs separated
/// by more than 100 bytes of whitespace).
pub(crate) fn find_preceding_doc(
    loc: &Loc,
    comment_map: &HashMap<usize, NatspecDocIR>,
) -> NatspecDocIR {
    if let Loc::File(_, start, _) = loc {
        if let Some(doc) = comment_map.get(start) {
            return doc.clone();
        }
    }
    NatspecDocIR::default()
}
