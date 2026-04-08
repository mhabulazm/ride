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
            TreeSitterLang::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            TreeSitterLang::Python => Some(tree_sitter_python::LANGUAGE.into()),
            TreeSitterLang::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            TreeSitterLang::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
            TreeSitterLang::Go => Some(tree_sitter_go::LANGUAGE.into()),
            TreeSitterLang::C => Some(tree_sitter_c::LANGUAGE.into()),
            TreeSitterLang::Cpp => Some(tree_sitter_cpp::LANGUAGE.into()),
        }
    }

    pub fn parse(&mut self, source: &str) {
        self.tree = self.parser.parse(source, self.tree.as_ref());
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }

    pub fn lang_name(&self) -> &str {
        match self.lang {
            TreeSitterLang::Java => "java",
            TreeSitterLang::Markdown => "markdown",
            TreeSitterLang::Rust => "rust",
            TreeSitterLang::Python => "python",
            TreeSitterLang::TypeScript => "typescript",
            TreeSitterLang::JavaScript => "javascript",
            TreeSitterLang::Go => "go",
            TreeSitterLang::C => "c",
            TreeSitterLang::Cpp => "cpp",
        }
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
        self.collect_spans(
            root,
            source,
            line_start_byte,
            line_end_byte,
            None,
            &mut spans,
        );
        spans
    }

    fn collect_spans(
        &self,
        node: tree_sitter::Node,
        source: &str,
        line_start: usize,
        line_end: usize,
        parent_kind: Option<&str>,
        spans: &mut Vec<HighlightSpan>,
    ) {
        let node_start = node.start_byte();
        let node_end = node.end_byte();

        // Skip nodes that don't overlap with our line
        if node_end <= line_start || node_start >= line_end {
            return;
        }

        // Check if this is a leaf node
        if node.child_count() == 0 {
            let kind = self.scope_aware_highlight(node.kind(), parent_kind, source, &node);
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

        // Recurse into children with current node as parent context
        let current_kind = node.kind();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_spans(
                child,
                source,
                line_start,
                line_end,
                Some(current_kind),
                spans,
            );
        }
    }

    /// Scope-aware highlighting: uses both the node kind and parent context.
    fn scope_aware_highlight(
        &self,
        node_kind: &str,
        parent_kind: Option<&str>,
        _source: &str,
        _node: &tree_sitter::Node,
    ) -> HighlightKind {
        match self.lang {
            TreeSitterLang::Java => self.java_highlight(node_kind, parent_kind),
            TreeSitterLang::Markdown => self.markdown_highlight(node_kind, parent_kind),
            TreeSitterLang::Rust => self.rust_highlight(node_kind, parent_kind),
            TreeSitterLang::Python => self.python_highlight(node_kind, parent_kind),
            TreeSitterLang::TypeScript | TreeSitterLang::JavaScript => {
                self.js_ts_highlight(node_kind, parent_kind)
            }
            TreeSitterLang::Go => self.go_highlight(node_kind, parent_kind),
            TreeSitterLang::C | TreeSitterLang::Cpp => self.c_cpp_highlight(node_kind, parent_kind),
        }
    }

    fn java_highlight(&self, node_kind: &str, parent_kind: Option<&str>) -> HighlightKind {
        match node_kind {
            // Comments
            "line_comment" | "block_comment" | "comment" => HighlightKind::Comment,

            // Strings
            "string_literal"
            | "character_literal"
            | "string_content"
            | "multiline_string_literal"
            | "string_fragment"
            | "\"" => HighlightKind::String,

            // Numbers
            "decimal_integer_literal"
            | "hex_integer_literal"
            | "octal_integer_literal"
            | "binary_integer_literal"
            | "decimal_floating_point_literal"
            | "hex_floating_point_literal"
            | "integer_literal"
            | "long_literal"
            | "real_literal" => HighlightKind::Number,

            // Keywords
            "public" | "private" | "protected" | "static" | "final" | "abstract" | "class"
            | "interface" | "extends" | "implements" | "return" | "if" | "else" | "for"
            | "while" | "do" | "switch" | "case" | "break" | "continue" | "new" | "try"
            | "catch" | "finally" | "throw" | "throws" | "import" | "package" | "void" | "this"
            | "super" | "null" | "true" | "false" | "default" | "synchronized" | "volatile"
            | "transient" | "native" | "strictfp" | "instanceof" | "enum" | "assert" | "yield"
            | "record" | "sealed" | "permits" | "non-sealed" => HighlightKind::Keyword,

            // Type identifiers — scope-aware
            "type_identifier"
            | "integral_type"
            | "floating_point_type"
            | "boolean_type"
            | "void_type"
            | "generic_type" => HighlightKind::Type,

            // Identifiers — classified by parent context
            "identifier" => match parent_kind {
                // Method name in declaration
                Some("method_declaration") | Some("constructor_declaration") => {
                    HighlightKind::Function
                }
                // Method name in invocation
                Some("method_invocation") => HighlightKind::Function,
                // Class/interface name in declaration
                Some("class_declaration")
                | Some("interface_declaration")
                | Some("enum_declaration") => HighlightKind::Type,
                // Annotation name
                Some("annotation") | Some("marker_annotation") => HighlightKind::Keyword,
                // Field access
                Some("field_access") => HighlightKind::Variable,
                // Formal parameter
                Some("formal_parameter")
                | Some("catch_formal_parameter")
                | Some("spread_parameter") => HighlightKind::Variable,
                // Variable declarator
                Some("variable_declarator") => HighlightKind::Variable,
                // Everything else
                _ => HighlightKind::Normal,
            },

            // Annotation marker
            "@" => HighlightKind::Keyword,

            // Operators
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&"
            | "||" | "!" | "&" | "|" | "^" | "~" | "<<" | ">>" | ">>>" | "+=" | "-=" | "*="
            | "/=" | "->" | "=>" | "++" | "--" | "?" | ":" => HighlightKind::Operator,

            // Punctuation
            "(" | ")" | "{" | "}" | "[" | "]" | ";" | "," | "." => HighlightKind::Punctuation,

            _ => HighlightKind::Normal,
        }
    }

    fn markdown_highlight(&self, node_kind: &str, _parent_kind: Option<&str>) -> HighlightKind {
        match node_kind {
            "atx_heading" | "setext_heading" | "heading_content" | "atx_h1_marker"
            | "atx_h2_marker" | "atx_h3_marker" | "atx_h4_marker" | "atx_h5_marker"
            | "atx_h6_marker" => HighlightKind::Heading,
            "link" | "uri_autolink" | "link_destination" | "link_text" => HighlightKind::Link,
            "emphasis" | "strong_emphasis" => HighlightKind::Emphasis,
            "code_span"
            | "code_fence_content"
            | "fenced_code_block"
            | "info_string"
            | "code_span_delimiter" => HighlightKind::String,
            "list_marker_dot"
            | "list_marker_minus"
            | "list_marker_plus"
            | "list_marker_star"
            | "list_marker_parenthesis" => HighlightKind::Operator,
            "block_quote_marker" => HighlightKind::Comment,
            "thematic_break" => HighlightKind::Punctuation,
            _ => HighlightKind::Normal,
        }
    }

    fn rust_highlight(&self, node_kind: &str, parent_kind: Option<&str>) -> HighlightKind {
        match node_kind {
            "line_comment" | "block_comment" => HighlightKind::Comment,

            "string_literal" | "raw_string_literal" | "char_literal" | "string_content"
            | "escape_sequence" | "\"" => HighlightKind::String,

            "integer_literal" | "float_literal" => HighlightKind::Number,

            "as" | "async" | "await" | "break" | "const" | "continue" | "crate" | "dyn"
            | "else" | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl" | "in"
            | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return"
            | "self" | "Self" | "static" | "struct" | "super" | "trait" | "true" | "type"
            | "unsafe" | "use" | "where" | "while" | "yield" | "macro_rules!" => {
                HighlightKind::Keyword
            }

            "type_identifier" | "primitive_type" => HighlightKind::Type,

            "identifier" => match parent_kind {
                Some("function_item") | Some("call_expression") => HighlightKind::Function,
                Some("struct_item") | Some("enum_item") | Some("type_item")
                | Some("trait_item") | Some("impl_item") => HighlightKind::Type,
                Some("field_expression")
                | Some("field_declaration")
                | Some("field_initializer") => HighlightKind::Variable,
                Some("parameter") | Some("let_declaration") => HighlightKind::Variable,
                _ => HighlightKind::Normal,
            },

            "attribute_item" | "meta_item" => HighlightKind::Keyword,

            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&"
            | "||" | "!" | "&" | "|" | "^" | "~" | "<<" | ">>" | "+=" | "-=" | "*=" | "/="
            | "->" | "=>" | ".." | "..=" | "::" | "?" => HighlightKind::Operator,

            "(" | ")" | "{" | "}" | "[" | "]" | ";" | "," | "." | ":" => HighlightKind::Punctuation,

            _ => HighlightKind::Normal,
        }
    }

    fn python_highlight(&self, node_kind: &str, parent_kind: Option<&str>) -> HighlightKind {
        match node_kind {
            "comment" => HighlightKind::Comment,

            "string" | "string_start" | "string_end" | "string_content" | "escape_sequence"
            | "interpolation" => HighlightKind::String,

            "integer" | "float" => HighlightKind::Number,

            "and" | "as" | "assert" | "async" | "await" | "break" | "class" | "continue"
            | "def" | "del" | "elif" | "else" | "except" | "finally" | "for" | "from"
            | "global" | "if" | "import" | "in" | "is" | "lambda" | "nonlocal" | "not" | "or"
            | "pass" | "raise" | "return" | "try" | "while" | "with" | "yield" | "True"
            | "False" | "None" => HighlightKind::Keyword,

            "type" => HighlightKind::Type,

            "identifier" => match parent_kind {
                Some("function_definition") | Some("call") => HighlightKind::Function,
                Some("class_definition") => HighlightKind::Type,
                Some("decorator") => HighlightKind::Keyword,
                Some("parameter") | Some("keyword_argument") => HighlightKind::Variable,
                _ => HighlightKind::Normal,
            },

            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "**"
            | "//" | "+=" | "-=" | "*=" | "/=" | "@" | "->" | ":=" | "|" | "&" | "^" | "~"
            | "<<" | ">>" => HighlightKind::Operator,

            "(" | ")" | "{" | "}" | "[" | "]" | ":" | "," | "." | ";" => HighlightKind::Punctuation,

            _ => HighlightKind::Normal,
        }
    }

    fn js_ts_highlight(&self, node_kind: &str, parent_kind: Option<&str>) -> HighlightKind {
        match node_kind {
            "comment" | "line_comment" | "block_comment" => HighlightKind::Comment,

            "string" | "string_fragment" | "template_string" | "template_literal"
            | "escape_sequence" | "\"" | "'" | "`" | "regex_pattern" | "regex" | "regex_flags" => {
                HighlightKind::String
            }

            "number" | "integer" | "float" => HighlightKind::Number,

            "as" | "async" | "await" | "break" | "case" | "catch" | "class" | "const"
            | "continue" | "debugger" | "default" | "delete" | "do" | "else" | "enum"
            | "export" | "extends" | "finally" | "for" | "from" | "function" | "if"
            | "implements" | "import" | "in" | "instanceof" | "interface" | "let" | "new"
            | "of" | "return" | "static" | "super" | "switch" | "this" | "throw" | "try"
            | "typeof" | "var" | "void" | "while" | "with" | "yield" | "null" | "undefined"
            | "true" | "false" | "abstract" | "declare" | "type" | "namespace" | "keyof"
            | "readonly" | "private" | "protected" | "public" | "override" => {
                HighlightKind::Keyword
            }

            "type_identifier" | "predefined_type" | "builtin_type" => HighlightKind::Type,

            "identifier"
            | "property_identifier"
            | "shorthand_property_identifier"
            | "shorthand_property_identifier_pattern" => match parent_kind {
                Some("function_declaration")
                | Some("method_definition")
                | Some("call_expression")
                | Some("new_expression") => HighlightKind::Function,
                Some("class_declaration")
                | Some("interface_declaration")
                | Some("type_alias_declaration") => HighlightKind::Type,
                Some("formal_parameters")
                | Some("required_parameter")
                | Some("optional_parameter") => HighlightKind::Variable,
                _ => HighlightKind::Normal,
            },

            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "===" | "!=" | "!==" | "<" | ">" | "<="
            | ">=" | "&&" | "||" | "!" | "&" | "|" | "^" | "~" | "<<" | ">>" | ">>>" | "+="
            | "-=" | "*=" | "/=" | "->" | "=>" | "++" | "--" | "?" | "?." | "??" | "..." => {
                HighlightKind::Operator
            }

            "(" | ")" | "{" | "}" | "[" | "]" | ";" | "," | "." | ":" => HighlightKind::Punctuation,

            _ => HighlightKind::Normal,
        }
    }

    fn go_highlight(&self, node_kind: &str, parent_kind: Option<&str>) -> HighlightKind {
        match node_kind {
            "comment" => HighlightKind::Comment,

            "raw_string_literal"
            | "interpreted_string_literal"
            | "rune_literal"
            | "escape_sequence"
            | "\""
            | "`" => HighlightKind::String,

            "int_literal" | "float_literal" | "imaginary_literal" => HighlightKind::Number,

            "break" | "case" | "chan" | "const" | "continue" | "default" | "defer" | "else"
            | "fallthrough" | "for" | "func" | "go" | "goto" | "if" | "import" | "interface"
            | "map" | "package" | "range" | "return" | "select" | "struct" | "switch" | "type"
            | "var" | "nil" | "true" | "false" | "iota" => HighlightKind::Keyword,

            "type_identifier" => HighlightKind::Type,

            "identifier" | "field_identifier" => match parent_kind {
                Some("function_declaration")
                | Some("method_declaration")
                | Some("call_expression") => HighlightKind::Function,
                Some("type_spec") | Some("type_declaration") => HighlightKind::Type,
                Some("field_declaration")
                | Some("parameter_declaration")
                | Some("short_var_declaration") => HighlightKind::Variable,
                _ => HighlightKind::Normal,
            },

            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&"
            | "||" | "!" | "&" | "|" | "^" | "<<" | ">>" | "&^" | "+=" | "-=" | "*=" | "/="
            | ":=" | "<-" | "++" | "--" => HighlightKind::Operator,

            "(" | ")" | "{" | "}" | "[" | "]" | ";" | "," | "." | ":" => HighlightKind::Punctuation,

            _ => HighlightKind::Normal,
        }
    }

    fn c_cpp_highlight(&self, node_kind: &str, parent_kind: Option<&str>) -> HighlightKind {
        match node_kind {
            "comment" | "line_comment" | "block_comment" => HighlightKind::Comment,

            "string_literal" | "raw_string_literal" | "char_literal" | "string_content"
            | "escape_sequence" | "system_lib_string" | "\"" => HighlightKind::String,

            "number_literal" | "integer_literal" | "float_literal" => HighlightKind::Number,

            "auto" | "break" | "case" | "char" | "const" | "continue" | "default" | "do"
            | "double" | "else" | "enum" | "extern" | "float" | "for" | "goto" | "if"
            | "inline" | "int" | "long" | "register" | "return" | "short" | "signed"
            | "sizeof" | "static" | "struct" | "switch" | "typedef" | "union" | "unsigned"
            | "void" | "volatile" | "while"
            // C++ additions
            | "class" | "namespace" | "template" | "typename" | "public" | "private"
            | "protected" | "virtual" | "override" | "final" | "new" | "delete" | "this"
            | "throw" | "try" | "catch" | "using" | "const_cast" | "dynamic_cast"
            | "static_cast" | "reinterpret_cast" | "nullptr" | "constexpr" | "decltype"
            | "noexcept" | "true" | "false" | "bool" | "explicit" | "friend" | "mutable"
            | "operator" | "alignas" | "alignof" | "concept" | "requires" | "co_await"
            | "co_return" | "co_yield" => HighlightKind::Keyword,

            "#include" | "#define" | "#ifdef" | "#ifndef" | "#endif" | "#if" | "#else"
            | "#elif" | "#pragma" | "#undef" | "#error" | "#warning" | "preproc_include"
            | "preproc_def" | "preproc_ifdef" | "preproc_directive" => HighlightKind::Keyword,

            "type_identifier" | "primitive_type" | "sized_type_specifier" => HighlightKind::Type,

            "identifier" | "field_identifier" => match parent_kind {
                Some("function_declarator") | Some("call_expression") => {
                    HighlightKind::Function
                }
                Some("struct_specifier") | Some("class_specifier")
                | Some("enum_specifier") | Some("type_definition") => HighlightKind::Type,
                Some("field_declaration") | Some("parameter_declaration")
                | Some("init_declarator") => HighlightKind::Variable,
                _ => HighlightKind::Normal,
            },

            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">="
            | "&&" | "||" | "!" | "&" | "|" | "^" | "~" | "<<" | ">>" | "+=" | "-="
            | "*=" | "/=" | "->" | "++" | "--" | "::" | "?" | "..." => {
                HighlightKind::Operator
            }

            "(" | ")" | "{" | "}" | "[" | "]" | ";" | "," | "." | ":" => {
                HighlightKind::Punctuation
            }

            _ => HighlightKind::Normal,
        }
    }
}
