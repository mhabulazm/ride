use super::{HighlightKind, HighlightSpan};

/// Basic code keyword highlighting (fallback for tree-sitter languages before full integration)
pub fn highlight_as_code(line: &str) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();
    let keywords = &[
        "public",
        "private",
        "protected",
        "class",
        "interface",
        "abstract",
        "static",
        "final",
        "void",
        "int",
        "long",
        "double",
        "float",
        "boolean",
        "char",
        "byte",
        "short",
        "return",
        "if",
        "else",
        "for",
        "while",
        "do",
        "switch",
        "case",
        "break",
        "continue",
        "new",
        "this",
        "super",
        "try",
        "catch",
        "finally",
        "throw",
        "throws",
        "import",
        "package",
        "fun",
        "val",
        "var",
        "when",
        "object",
        "companion",
        "data",
        "sealed",
        "override",
        "suspend",
        "lateinit",
        "const",
        "null",
        "true",
        "false",
        "syntax",
        "message",
        "service",
        "rpc",
        "returns",
        "repeated",
        "optional",
        "required",
        "string",
        "bool",
        "int32",
        "int64",
        "uint32",
        "uint64",
        "bytes",
        "enum",
        "oneof",
        "map",
    ];

    // Check for single-line comment
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") || trimmed.starts_with('#') {
        let offset = line.len() - trimmed.len();
        spans.push(HighlightSpan {
            start: offset,
            end: line.len(),
            kind: HighlightKind::Comment,
        });
        return spans;
    }

    // Check for string literals
    let mut in_string = false;
    let mut string_start = 0;
    let mut string_char = '"';
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if in_string {
            if chars[i] == string_char && (i == 0 || chars[i - 1] != '\\') {
                let byte_end = chars[..=i].iter().collect::<String>().len();
                spans.push(HighlightSpan {
                    start: string_start,
                    end: byte_end,
                    kind: HighlightKind::String,
                });
                in_string = false;
            }
        } else if chars[i] == '"' || chars[i] == '\'' {
            string_char = chars[i];
            string_start = chars[..i].iter().collect::<String>().len();
            in_string = true;
        }
        i += 1;
    }

    // Highlight keywords (word boundary matching)
    for kw in keywords {
        let mut search_from = 0;
        while let Some(pos) = line[search_from..].find(kw) {
            let abs_pos = search_from + pos;
            let before_ok = abs_pos == 0
                || !line.as_bytes()[abs_pos - 1].is_ascii_alphanumeric()
                    && line.as_bytes()[abs_pos - 1] != b'_';
            let after_pos = abs_pos + kw.len();
            let after_ok = after_pos >= line.len()
                || !line.as_bytes()[after_pos].is_ascii_alphanumeric()
                    && line.as_bytes()[after_pos] != b'_';

            if before_ok && after_ok {
                // Don't overlap with string spans
                let overlaps = spans.iter().any(|s| {
                    s.kind == HighlightKind::String && abs_pos >= s.start && abs_pos < s.end
                });
                if !overlaps {
                    spans.push(HighlightSpan {
                        start: abs_pos,
                        end: after_pos,
                        kind: HighlightKind::Keyword,
                    });
                }
            }
            search_from = abs_pos + 1;
        }
    }

    // Highlight numbers
    let mut num_start = None;
    for (idx, ch) in line.char_indices() {
        if ch.is_ascii_digit() || (ch == '.' && num_start.is_some()) {
            if num_start.is_none() {
                // Don't match digits that are part of identifiers
                if idx > 0
                    && (line.as_bytes()[idx - 1].is_ascii_alphabetic()
                        || line.as_bytes()[idx - 1] == b'_')
                {
                    continue;
                }
                num_start = Some(idx);
            }
        } else if let Some(start) = num_start {
            let overlaps = spans.iter().any(|s| start >= s.start && start < s.end);
            if !overlaps {
                spans.push(HighlightSpan {
                    start,
                    end: idx,
                    kind: HighlightKind::Number,
                });
            }
            num_start = None;
        }
    }

    spans
}

pub fn highlight_log(line: &str) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();
    let upper = line.to_uppercase();

    if upper.contains("ERROR") || upper.contains("FATAL") {
        spans.push(HighlightSpan {
            start: 0,
            end: line.len(),
            kind: HighlightKind::LogError,
        });
    } else if upper.contains("WARN") {
        spans.push(HighlightSpan {
            start: 0,
            end: line.len(),
            kind: HighlightKind::LogWarn,
        });
    } else if upper.contains("INFO") {
        spans.push(HighlightSpan {
            start: 0,
            end: line.len(),
            kind: HighlightKind::LogInfo,
        });
    } else if upper.contains("DEBUG") || upper.contains("TRACE") {
        spans.push(HighlightSpan {
            start: 0,
            end: line.len(),
            kind: HighlightKind::LogDebug,
        });
    }

    // Timestamp pattern: common log formats like 2024-01-01 12:00:00
    if line.len() >= 19 {
        let prefix = &line[..19];
        if prefix.chars().nth(4) == Some('-')
            && prefix.chars().nth(7) == Some('-')
            && prefix.chars().nth(13) == Some(':')
        {
            spans.push(HighlightSpan {
                start: 0,
                end: 19,
                kind: HighlightKind::LogTimestamp,
            });
        }
    }

    spans
}

pub fn highlight_mermaid(line: &str) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();
    let trimmed = line.trim();

    let keywords = &[
        "graph",
        "subgraph",
        "end",
        "flowchart",
        "sequenceDiagram",
        "classDiagram",
        "stateDiagram",
        "erDiagram",
        "gantt",
        "pie",
        "gitGraph",
        "mindmap",
        "timeline",
        "participant",
        "actor",
        "activate",
        "deactivate",
        "note",
        "loop",
        "alt",
        "opt",
        "par",
        "critical",
        "break",
        "rect",
        "title",
        "section",
        "LR",
        "RL",
        "TB",
        "BT",
        "TD",
    ];

    // Check for comments
    if trimmed.starts_with("%%") {
        let offset = line.len() - trimmed.len();
        spans.push(HighlightSpan {
            start: offset,
            end: line.len(),
            kind: HighlightKind::Comment,
        });
        return spans;
    }

    // Highlight keywords
    for kw in keywords {
        if let Some(pos) = line.find(kw) {
            spans.push(HighlightSpan {
                start: pos,
                end: pos + kw.len(),
                kind: HighlightKind::MermaidKeyword,
            });
        }
    }

    // Highlight arrows
    let arrows = &[
        "-->", "---", "-.->", "==>", "-->|", "|", "->", "<--", "<-->",
    ];
    for arrow in arrows {
        let mut search_from = 0;
        while let Some(pos) = line[search_from..].find(arrow) {
            let abs_pos = search_from + pos;
            spans.push(HighlightSpan {
                start: abs_pos,
                end: abs_pos + arrow.len(),
                kind: HighlightKind::MermaidArrow,
            });
            search_from = abs_pos + arrow.len();
        }
    }

    spans
}
