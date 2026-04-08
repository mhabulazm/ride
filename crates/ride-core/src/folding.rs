use std::collections::BTreeSet;

/// A foldable region identified by its start and end buffer lines (inclusive).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldRegion {
    pub start_line: usize,
    pub end_line: usize,
    pub kind: FoldKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldKind {
    Block, // { ... }
    Function,
    Class,
    Comment,
    Import,
    Section, // Markdown headings
}

/// Manages fold state for a single buffer.
pub struct FoldState {
    /// All detected foldable regions, sorted by start_line.
    pub regions: Vec<FoldRegion>,
    /// Set of start_lines that are currently folded.
    pub folded: BTreeSet<usize>,
}

impl Default for FoldState {
    fn default() -> Self {
        Self::new()
    }
}

impl FoldState {
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            folded: BTreeSet::new(),
        }
    }

    /// Update fold regions from a tree-sitter parse tree.
    pub fn update_regions_from_tree(&mut self, tree: &tree_sitter::Tree, source: &str, lang: &str) {
        let mut regions = Vec::new();
        let root = tree.root_node();
        collect_fold_regions(root, source, lang, &mut regions);
        // Sort by start line, then by size (larger regions first)
        regions.sort_by(|a, b| {
            a.start_line
                .cmp(&b.start_line)
                .then_with(|| b.end_line.cmp(&a.end_line))
        });
        // Remove stale folds that no longer exist
        let valid_starts: BTreeSet<usize> = regions.iter().map(|r| r.start_line).collect();
        self.folded.retain(|s| valid_starts.contains(s));
        self.regions = regions;
    }

    /// Toggle fold at the given buffer line. If the line is the start of a
    /// foldable region, toggle it. Otherwise find the innermost region
    /// containing this line.
    pub fn toggle_fold(&mut self, line: usize) {
        let start = self.region_at(line).map(|r| r.start_line);
        if let Some(s) = start {
            if self.folded.contains(&s) {
                self.folded.remove(&s);
            } else {
                self.folded.insert(s);
            }
        }
    }

    /// Fold the region at the given line.
    pub fn fold(&mut self, line: usize) {
        let start = self.region_at(line).map(|r| r.start_line);
        if let Some(s) = start {
            self.folded.insert(s);
        }
    }

    /// Unfold the region at the given line.
    pub fn unfold(&mut self, line: usize) {
        let start = self.region_at(line).map(|r| r.start_line);
        if let Some(s) = start {
            self.folded.remove(&s);
        }
    }

    /// Unfold all regions.
    pub fn unfold_all(&mut self) {
        self.folded.clear();
    }

    /// Find the innermost fold region that starts at or contains `line`.
    pub fn region_at(&self, line: usize) -> Option<&FoldRegion> {
        // Prefer region starting at this line
        if let Some(r) = self.regions.iter().find(|r| r.start_line == line) {
            return Some(r);
        }
        // Otherwise find innermost containing region
        self.regions
            .iter()
            .rfind(|r| r.start_line <= line && r.end_line >= line)
    }

    /// Check if a given line is hidden (inside a folded region, but not the start line).
    pub fn is_line_hidden(&self, line: usize) -> bool {
        for &fold_start in &self.folded {
            if let Some(region) = self.regions.iter().find(|r| r.start_line == fold_start) {
                if line > region.start_line && line <= region.end_line {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a given line is the start of a foldable region.
    pub fn is_fold_start(&self, line: usize) -> bool {
        self.regions.iter().any(|r| r.start_line == line)
    }

    /// Check if a given line is folded (the region starting here is collapsed).
    pub fn is_folded(&self, line: usize) -> bool {
        self.folded.contains(&line)
    }

    /// Get the fold region starting at this line, if any.
    pub fn get_region_at_start(&self, line: usize) -> Option<&FoldRegion> {
        self.regions.iter().find(|r| r.start_line == line)
    }
}

fn collect_fold_regions(
    node: tree_sitter::Node,
    _source: &str,
    lang: &str,
    regions: &mut Vec<FoldRegion>,
) {
    let start_line = node.start_position().row;
    let end_line = node.end_position().row;

    // Only fold multi-line nodes
    if end_line > start_line {
        let kind = classify_node(node.kind(), lang);
        if let Some(fold_kind) = kind {
            regions.push(FoldRegion {
                start_line,
                end_line,
                kind: fold_kind,
            });
        }
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_fold_regions(child, _source, lang, regions);
    }
}

fn classify_node(node_kind: &str, lang: &str) -> Option<FoldKind> {
    match lang {
        "java" => match node_kind {
            "class_declaration"
            | "interface_declaration"
            | "enum_declaration"
            | "annotation_type_declaration" => Some(FoldKind::Class),
            "method_declaration" | "constructor_declaration" => Some(FoldKind::Function),
            "block" | "switch_block" | "class_body" | "interface_body" | "enum_body"
            | "array_initializer" => Some(FoldKind::Block),
            "block_comment" | "line_comment" => Some(FoldKind::Comment),
            "import_declaration" => Some(FoldKind::Import),
            _ => None,
        },
        "markdown" => match node_kind {
            "section" | "atx_heading" => Some(FoldKind::Section),
            "fenced_code_block" | "indented_code_block" => Some(FoldKind::Block),
            "block_quote" | "list" => Some(FoldKind::Block),
            _ => None,
        },
        "rust" => match node_kind {
            "function_item" | "closure_expression" => Some(FoldKind::Function),
            "struct_item" | "enum_item" | "impl_item" | "trait_item" | "mod_item" => {
                Some(FoldKind::Class)
            }
            "block"
            | "match_block"
            | "declaration_list"
            | "field_declaration_list"
            | "enum_variant_list" => Some(FoldKind::Block),
            "block_comment" | "line_comment" => Some(FoldKind::Comment),
            "use_declaration" => Some(FoldKind::Import),
            _ => None,
        },
        "python" => match node_kind {
            "function_definition" => Some(FoldKind::Function),
            "class_definition" => Some(FoldKind::Class),
            "block" | "if_statement" | "for_statement" | "while_statement" | "try_statement"
            | "with_statement" | "match_statement" => Some(FoldKind::Block),
            "comment" => Some(FoldKind::Comment),
            "import_from_statement" => Some(FoldKind::Import),
            _ => None,
        },
        "typescript" | "javascript" => match node_kind {
            "function_declaration"
            | "method_definition"
            | "arrow_function"
            | "generator_function_declaration" => Some(FoldKind::Function),
            "class_declaration" | "interface_declaration" => Some(FoldKind::Class),
            "statement_block" | "switch_body" | "object" | "array" | "class_body"
            | "if_statement" | "for_statement" | "for_in_statement" | "while_statement"
            | "try_statement" => Some(FoldKind::Block),
            "comment" | "line_comment" | "block_comment" => Some(FoldKind::Comment),
            "import_statement" => Some(FoldKind::Import),
            _ => None,
        },
        "go" => match node_kind {
            "function_declaration" | "method_declaration" | "func_literal" => {
                Some(FoldKind::Function)
            }
            "type_declaration" | "type_spec" => Some(FoldKind::Class),
            "block" | "literal_value" | "interface_type" | "struct_type" | "select_statement"
            | "switch_statement" | "if_statement" | "for_statement" => Some(FoldKind::Block),
            "comment" => Some(FoldKind::Comment),
            "import_declaration" => Some(FoldKind::Import),
            _ => None,
        },
        "c" | "cpp" => match node_kind {
            "function_definition" => Some(FoldKind::Function),
            "struct_specifier"
            | "class_specifier"
            | "enum_specifier"
            | "union_specifier"
            | "namespace_definition" => Some(FoldKind::Class),
            "compound_statement"
            | "field_declaration_list"
            | "enumerator_list"
            | "initializer_list"
            | "declaration_list"
            | "if_statement"
            | "for_statement"
            | "while_statement"
            | "switch_statement" => Some(FoldKind::Block),
            "comment" | "block_comment" | "line_comment" => Some(FoldKind::Comment),
            "preproc_include" | "preproc_def" | "preproc_ifdef" => Some(FoldKind::Import),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_region(start: usize, end: usize, kind: FoldKind) -> FoldRegion {
        FoldRegion {
            start_line: start,
            end_line: end,
            kind,
        }
    }

    #[test]
    fn test_fold_state_new() {
        let state = FoldState::new();
        assert!(state.regions.is_empty());
        assert!(state.folded.is_empty());
    }

    #[test]
    fn test_toggle_fold() {
        let mut state = FoldState::new();
        state.regions = vec![make_region(5, 10, FoldKind::Function)];
        state.toggle_fold(5);
        assert!(state.is_folded(5));
        state.toggle_fold(5);
        assert!(!state.is_folded(5));
    }

    #[test]
    fn test_toggle_fold_from_inside() {
        let mut state = FoldState::new();
        state.regions = vec![make_region(5, 10, FoldKind::Function)];
        state.toggle_fold(7); // inside the region
        assert!(state.is_folded(5)); // folds from start
    }

    #[test]
    fn test_is_line_hidden() {
        let mut state = FoldState::new();
        state.regions = vec![make_region(5, 10, FoldKind::Function)];
        state.fold(5);
        assert!(!state.is_line_hidden(5)); // start line is visible
        assert!(state.is_line_hidden(6));
        assert!(state.is_line_hidden(10));
        assert!(!state.is_line_hidden(11));
        assert!(!state.is_line_hidden(4));
    }

    #[test]
    fn test_unfold_all() {
        let mut state = FoldState::new();
        state.regions = vec![
            make_region(1, 5, FoldKind::Class),
            make_region(10, 20, FoldKind::Function),
        ];
        state.fold(1);
        state.fold(10);
        assert_eq!(state.folded.len(), 2);
        state.unfold_all();
        assert!(state.folded.is_empty());
    }

    #[test]
    fn test_is_fold_start() {
        let mut state = FoldState::new();
        state.regions = vec![make_region(5, 10, FoldKind::Block)];
        assert!(state.is_fold_start(5));
        assert!(!state.is_fold_start(6));
    }

    #[test]
    fn test_nested_regions() {
        let mut state = FoldState::new();
        state.regions = vec![
            make_region(1, 20, FoldKind::Class),
            make_region(3, 8, FoldKind::Function),
            make_region(10, 15, FoldKind::Function),
        ];
        // region_at line 5 should return the function (innermost)
        let r = state.region_at(5).unwrap();
        assert_eq!(r.start_line, 3);
        assert_eq!(r.end_line, 8);
    }

    #[test]
    fn test_fold_hides_nested_content() {
        let mut state = FoldState::new();
        state.regions = vec![
            make_region(1, 20, FoldKind::Class),
            make_region(3, 8, FoldKind::Function),
        ];
        state.fold(1);
        assert!(state.is_line_hidden(3));
        assert!(state.is_line_hidden(8));
        assert!(state.is_line_hidden(15));
        assert!(!state.is_line_hidden(1));
    }

    #[test]
    fn test_region_at_start_line() {
        let mut state = FoldState::new();
        state.regions = vec![make_region(5, 10, FoldKind::Function)];
        let r = state.get_region_at_start(5);
        assert!(r.is_some());
        assert_eq!(r.unwrap().end_line, 10);
        assert!(state.get_region_at_start(6).is_none());
    }

    #[test]
    fn test_update_regions_removes_stale_folds() {
        let mut state = FoldState::new();
        state.regions = vec![make_region(5, 10, FoldKind::Function)];
        state.fold(5);
        assert!(state.is_folded(5));

        // Simulate re-parse with different regions
        state.regions = vec![make_region(7, 12, FoldKind::Function)];
        let valid_starts: BTreeSet<usize> = state.regions.iter().map(|r| r.start_line).collect();
        state.folded.retain(|s| valid_starts.contains(s));
        assert!(!state.is_folded(5)); // stale fold removed
    }
}
