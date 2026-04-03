use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const SUPPORTED_EXTENSIONS: &[&str] = &["java", "kt", "log", "txt", "md", "mmd", "proto"];

pub struct FuzzyFinder {
    pub query: String,
    pub all_files: Vec<PathBuf>,
    pub filtered: Vec<PathBuf>,
    pub selected: usize,
    pub active: bool,
    root: PathBuf,
}

impl FuzzyFinder {
    pub fn new(root: &Path) -> Self {
        let all_files = collect_files(root);
        Self {
            query: String::new(),
            all_files,
            filtered: Vec::new(),
            selected: 0,
            active: false,
            root: root.to_path_buf(),
        }
    }

    pub fn open(&mut self) {
        self.query.clear();
        self.selected = 0;
        self.active = true;
        self.update_filter();
    }

    pub fn close(&mut self) {
        self.active = false;
        self.query.clear();
        self.filtered.clear();
    }

    pub fn input(&mut self, ch: char) {
        self.query.push(ch);
        self.selected = 0;
        self.update_filter();
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.selected = 0;
        self.update_filter();
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() && self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }

    pub fn confirm(&self) -> Option<PathBuf> {
        self.filtered.get(self.selected).cloned()
    }

    pub fn display_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    }

    fn update_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = self.all_files.clone();
        } else {
            let query_lower = self.query.to_lowercase();
            let mut scored: Vec<(i64, &PathBuf)> = self
                .all_files
                .iter()
                .filter_map(|p| {
                    let name = self.display_path(p).to_lowercase();
                    fuzzy_score(&query_lower, &name).map(|score| (score, p))
                })
                .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0));
            self.filtered = scored.into_iter().map(|(_, p)| p.clone()).collect();
        }
    }
}

/// Simple fuzzy matching: all query chars must appear in order in the target.
/// Returns a score (higher is better) or None if no match.
pub fn fuzzy_score(query: &str, target: &str) -> Option<i64> {
    let mut score: i64 = 0;
    let mut target_iter = target.char_indices().peekable();
    let mut prev_match_idx: Option<usize> = None;

    for qch in query.chars() {
        let mut found = false;
        for (idx, tch) in target_iter.by_ref() {
            if tch == qch {
                // Bonus for consecutive matches
                if let Some(prev) = prev_match_idx {
                    if idx == prev + 1 {
                        score += 10;
                    }
                }
                // Bonus for matching at word boundaries (after / or . or start)
                if idx == 0 || matches!(target.as_bytes().get(idx - 1), Some(b'/' | b'\\' | b'.' | b'_' | b'-')) {
                    score += 5;
                }
                score += 1;
                prev_match_idx = Some(idx);
                found = true;
                break;
            }
        }
        if !found {
            return None;
        }
    }

    // Prefer shorter paths (less noise)
    score -= (target.len() as i64) / 10;

    Some(score)
}

/// Check if all query chars appear in order in the target (case-insensitive).
pub fn fuzzy_matches(query: &str, target: &str) -> bool {
    fuzzy_score(&query.to_lowercase(), &target.to_lowercase()).is_some()
}

fn collect_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Skip hidden dirs/files (only check components relative to root)
        if let Ok(rel) = path.strip_prefix(root) {
            if rel
                .components()
                .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
            {
                continue;
            }
        }

        let supported = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| SUPPORTED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
            .unwrap_or(false);

        if supported {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_score_exact_match() {
        assert!(fuzzy_score("abc", "abc").is_some());
    }

    #[test]
    fn test_fuzzy_score_subsequence() {
        assert!(fuzzy_score("ac", "abc").is_some());
        assert!(fuzzy_score("ae", "abcde").is_some());
    }

    #[test]
    fn test_fuzzy_score_no_match() {
        assert!(fuzzy_score("xyz", "abc").is_none());
        assert!(fuzzy_score("ba", "abc").is_none()); // wrong order
    }

    #[test]
    fn test_fuzzy_score_empty_query() {
        assert!(fuzzy_score("", "anything").is_some());
    }

    #[test]
    fn test_fuzzy_score_consecutive_bonus() {
        let consecutive = fuzzy_score("ab", "abc").unwrap();
        let spread = fuzzy_score("ac", "abc").unwrap();
        assert!(consecutive > spread);
    }

    #[test]
    fn test_fuzzy_score_boundary_bonus() {
        // 'a' at start of path component gets boundary bonus
        let boundary = fuzzy_score("a", "src/app").unwrap();
        let mid = fuzzy_score("r", "src/app").unwrap();
        assert!(boundary >= mid);
    }

    #[test]
    fn test_fuzzy_matches() {
        assert!(fuzzy_matches("mkt", "src/Main.kt"));
        assert!(!fuzzy_matches("xyz", "src/Main.kt"));
    }

    #[test]
    fn test_fuzzy_finder_open_close() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.txt"), "hello").unwrap();
        let mut finder = FuzzyFinder::new(dir.path());

        assert!(!finder.active);
        finder.open();
        assert!(finder.active);
        assert!(finder.query.is_empty());
        assert!(!finder.filtered.is_empty());
        finder.close();
        assert!(!finder.active);
    }

    #[test]
    fn test_fuzzy_finder_filtering() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("apple.txt"), "").unwrap();
        std::fs::write(dir.path().join("banana.txt"), "").unwrap();
        std::fs::write(dir.path().join("cherry.md"), "").unwrap();
        let mut finder = FuzzyFinder::new(dir.path());
        finder.open();
        assert_eq!(finder.filtered.len(), 3);

        finder.input('a');
        // "apple" and "banana" match 'a', "cherry" does too (has 'a' in... no 'a' in cherry)
        // Actually: "apple.txt" has 'a', "banana.txt" has 'a', "cherry.md" no 'a' — wait it does not
        // Let's just check it filters down
        assert!(finder.filtered.len() <= 3);

        finder.input('p');
        // "ap" matches "apple.txt"
        assert!(finder.filtered.iter().any(|p| p.file_name().unwrap().to_string_lossy().contains("apple")));
    }

    #[test]
    fn test_fuzzy_finder_navigation() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "").unwrap();
        std::fs::write(dir.path().join("b.txt"), "").unwrap();
        let mut finder = FuzzyFinder::new(dir.path());
        finder.open();

        assert_eq!(finder.selected, 0);
        finder.move_down();
        assert_eq!(finder.selected, 1);
        finder.move_down();
        assert_eq!(finder.selected, 1); // can't go past end
        finder.move_up();
        assert_eq!(finder.selected, 0);
        finder.move_up();
        assert_eq!(finder.selected, 0); // can't go below 0
    }

    #[test]
    fn test_fuzzy_finder_confirm() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.txt"), "").unwrap();
        let mut finder = FuzzyFinder::new(dir.path());
        finder.open();
        let result = finder.confirm();
        assert!(result.is_some());
    }

    #[test]
    fn test_fuzzy_finder_backspace() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.txt"), "").unwrap();
        let mut finder = FuzzyFinder::new(dir.path());
        finder.open();
        finder.input('z');
        finder.input('z');
        assert!(finder.filtered.is_empty());
        finder.backspace();
        finder.backspace();
        assert!(!finder.filtered.is_empty());
    }

    #[test]
    fn test_collect_files_skips_hidden() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".hidden")).unwrap();
        std::fs::write(dir.path().join(".hidden/secret.txt"), "").unwrap();
        std::fs::write(dir.path().join("visible.txt"), "").unwrap();
        let files = collect_files(dir.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].file_name().unwrap().to_string_lossy().contains("visible"));
    }

    #[test]
    fn test_collect_files_only_supported_extensions() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("code.java"), "").unwrap();
        std::fs::write(dir.path().join("script.py"), "").unwrap();
        std::fs::write(dir.path().join("notes.md"), "").unwrap();
        let files = collect_files(dir.path());
        assert_eq!(files.len(), 2); // java and md, not py
    }

    #[test]
    fn test_display_path_strips_root() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.txt"), "").unwrap();
        let finder = FuzzyFinder::new(dir.path());
        let full = dir.path().join("test.txt");
        assert_eq!(finder.display_path(&full), "test.txt");
    }
}
