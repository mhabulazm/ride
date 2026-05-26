use std::path::Path;

pub mod regex_hl;
pub mod treesitter_hl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightKind {
    Normal,
    Keyword,
    Type,
    String,
    Comment,
    Number,
    Function,
    Operator,
    Punctuation,
    Variable,
    Heading,
    Link,
    Emphasis,
    // Mermaid specific
    MermaidKeyword,
    MermaidArrow,
    // LOG specific
    LogError,
    LogWarn,
    LogInfo,
    LogDebug,
    LogTimestamp,
}

#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub kind: HighlightKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlighterType {
    TreeSitter(TreeSitterLang),
    Regex(RegexLang),
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeSitterLang {
    Java,
    Markdown,
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
    C,
    Cpp,
    Html,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegexLang {
    Code, // generic code keywords (used for Kotlin, Proto)
    Log,
    Mermaid,
}

pub fn detect_highlighter(path: &Path) -> HighlighterType {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("java") => HighlighterType::TreeSitter(TreeSitterLang::Java),
        Some("rs") => HighlighterType::TreeSitter(TreeSitterLang::Rust),
        Some("py") => HighlighterType::TreeSitter(TreeSitterLang::Python),
        Some("ts" | "tsx") => HighlighterType::TreeSitter(TreeSitterLang::TypeScript),
        Some("js" | "jsx") => HighlighterType::TreeSitter(TreeSitterLang::JavaScript),
        Some("go") => HighlighterType::TreeSitter(TreeSitterLang::Go),
        Some("c" | "h") => HighlighterType::TreeSitter(TreeSitterLang::C),
        Some("cpp" | "cc" | "hpp" | "cxx" | "hxx") => {
            HighlighterType::TreeSitter(TreeSitterLang::Cpp)
        }
        Some("html" | "htm") => HighlighterType::TreeSitter(TreeSitterLang::Html),
        Some("kt") => HighlighterType::Regex(RegexLang::Code),
        Some("md") => HighlighterType::TreeSitter(TreeSitterLang::Markdown),
        Some("proto") => HighlighterType::Regex(RegexLang::Code),
        Some("log") => HighlighterType::Regex(RegexLang::Log),
        Some("mmd") => HighlighterType::Regex(RegexLang::Mermaid),
        Some("txt") => HighlighterType::Plain,
        _ => HighlighterType::Plain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_html_extension_detected() {
        assert_eq!(
            detect_highlighter(Path::new("index.html")),
            HighlighterType::TreeSitter(TreeSitterLang::Html)
        );
        assert_eq!(
            detect_highlighter(Path::new("page.htm")),
            HighlighterType::TreeSitter(TreeSitterLang::Html)
        );
    }

    #[test]
    fn test_html_highlights_tag_name() {
        use crate::highlight::treesitter_hl::TreeSitterHighlighter;
        let mut hl = TreeSitterHighlighter::new(TreeSitterLang::Html).unwrap();
        let src = "<p class=\"x\">hi</p>";
        hl.parse(src);
        let spans = hl.highlight_line(src, 0);
        // At least one Type span (the tag name) and one String span (the attr value).
        assert!(spans.iter().any(|s| s.kind == HighlightKind::Type));
        assert!(spans.iter().any(|s| s.kind == HighlightKind::String));
    }
}

/// Get highlight spans for a single line of text.
/// This is the main entry point used by the UI.
pub fn highlight_line(
    highlighter_type: HighlighterType,
    line: &str,
    _line_idx: usize,
) -> Vec<HighlightSpan> {
    match highlighter_type {
        HighlighterType::TreeSitter(_lang) => {
            // Tree-sitter operates on full source, so per-line highlighting
            // will be handled by the treesitter module with cached parse trees.
            // For now, fall back to basic keyword highlighting.
            regex_hl::highlight_as_code(line)
        }
        HighlighterType::Regex(RegexLang::Code) => regex_hl::highlight_as_code(line),
        HighlighterType::Regex(RegexLang::Log) => regex_hl::highlight_log(line),
        HighlighterType::Regex(RegexLang::Mermaid) => regex_hl::highlight_mermaid(line),
        HighlighterType::Plain => vec![],
    }
}
