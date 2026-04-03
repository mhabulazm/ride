use super::{HighlightKind, HighlightSpan, TreeSitterLang};
use tree_sitter::{Language, Parser, Tree};

pub struct TreeSitterHighlighter {
    parser: Parser,
    tree: Option<Tree>,
    lang: TreeSitterLang,
}

impl TreeSitterHighlighter {
    pub fn new(lang: TreeSitterLang) -> Option<Self> {
        let language = Self::get_language(lang)?;
        let mut parser = Parser::new();
        parser.set_language(&language).ok()?;
        Some(Self {
            parser,
            tree: None,
            lang,
        })
    }

    fn get_language(lang: TreeSitterLang) -> Option<Language> {
        match lang {
            TreeSitterLang::Java => Some(tree_sitter_java::LANGUAGE.into()),
            TreeSitterLang::Markdown => Some(tree_sitter_md::LANGUAGE.into()),
        }
    }

    pub fn parse(&mut self, source: &str) {
        self.tree = self.parser.parse(source, self.tree.as_ref());
    }

    pub fn highlight_line(&self, source: &str, line_idx: usize) -> Vec<HighlightSpan> {
        let tree = match &self.tree {
            Some(t) => t,
            None => return vec![],
        };

        let root = tree.root_node();
        let line_start_byte = source
            .lines()
            .take(line_idx)
            .map(|l| l.len() + 1) // +1 for newline
            .sum::<usize>();

        let line_text = source.lines().nth(line_idx).unwrap_or("");
        let line_end_byte = line_start_byte + line_text.len();

        let mut spans = Vec::new();
        self.collect_spans(root, source, line_start_byte, line_end_byte, &mut spans);
        spans
    }

    fn collect_spans(
        &self,
        node: tree_sitter::Node,
        _source: &str,
        line_start: usize,
        line_end: usize,
        spans: &mut Vec<HighlightSpan>,
    ) {
        let node_start = node.start_byte();
        let node_end = node.end_byte();

        // Skip nodes that don't overlap with our line
        if node_end <= line_start || node_start >= line_end {
            return;
        }

        // Check if this is a leaf node or a relevant node
        if node.child_count() == 0 {
            let kind = self.node_to_highlight_kind(node.kind());
            if kind != HighlightKind::Normal {
                let span_start = node_start.max(line_start) - line_start;
                let span_end = node_end.min(line_end) - line_start;
                spans.push(HighlightSpan {
                    start: span_start,
                    end: span_end,
                    kind,
                });
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_spans(child, _source, line_start, line_end, spans);
        }
    }

    fn node_to_highlight_kind(&self, node_kind: &str) -> HighlightKind {
        match self.lang {
            TreeSitterLang::Java => match node_kind {
                "line_comment" | "block_comment" | "comment" => HighlightKind::Comment,
                "string_literal" | "character_literal" | "string_content"
                | "multiline_string_literal" => HighlightKind::String,
                "decimal_integer_literal" | "hex_integer_literal" | "octal_integer_literal"
                | "binary_integer_literal" | "decimal_floating_point_literal"
                | "hex_floating_point_literal" | "integer_literal" | "long_literal"
                | "real_literal" => HighlightKind::Number,
                "public" | "private" | "protected" | "static" | "final" | "abstract"
                | "class" | "interface" | "extends" | "implements" | "return" | "if" | "else"
                | "for" | "while" | "do" | "switch" | "case" | "break" | "continue" | "new"
                | "try" | "catch" | "finally" | "throw" | "throws" | "import" | "package"
                | "void" | "this" | "super" | "null" | "true" | "false" => {
                    HighlightKind::Keyword
                }
                "type_identifier" | "integral_type" | "floating_point_type"
                | "boolean_type" | "void_type" => HighlightKind::Type,
                "method_invocation" => HighlightKind::Function,
                "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">="
                | "&&" | "||" | "!" | "&" | "|" | "^" | "~" | "<<" | ">>" | "+=" | "-="
                | "*=" | "/=" | "->" | "=>" => HighlightKind::Operator,
                _ => HighlightKind::Normal,
            },
            TreeSitterLang::Markdown => match node_kind {
                "atx_heading" | "setext_heading" | "heading_content" => HighlightKind::Heading,
                "link" | "uri_autolink" | "link_destination" => HighlightKind::Link,
                "emphasis" | "strong_emphasis" => HighlightKind::Emphasis,
                "code_span" | "code_fence_content" | "fenced_code_block" => {
                    HighlightKind::String
                }
                _ => HighlightKind::Normal,
            },
        }
    }
}
