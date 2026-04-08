use regex::RegexBuilder;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub file: Option<PathBuf>,
    pub line: usize,
    pub col: usize,
    pub text: String,
}

pub struct SearchState {
    pub query: String,
    pub matches: Vec<SearchMatch>,
    pub current: usize,
    pub active: bool,
    pub across_files: bool,
    pub case_insensitive: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            matches: Vec::new(),
            current: 0,
            active: false,
            across_files: false,
            case_insensitive: true,
        }
    }

    pub fn search_in_buffer(&mut self, lines: &[String]) {
        self.matches.clear();
        self.current = 0;

        if self.query.is_empty() {
            return;
        }

        let re = match RegexBuilder::new(&regex::escape(&self.query))
            .case_insensitive(self.case_insensitive)
            .build()
        {
            Ok(re) => re,
            Err(_) => return,
        };

        for (line_idx, line) in lines.iter().enumerate() {
            for m in re.find_iter(line) {
                self.matches.push(SearchMatch {
                    file: None,
                    line: line_idx,
                    col: m.start(),
                    text: line.trim_end().to_string(),
                });
            }
        }
    }

    pub fn search_across_files(&mut self, dir: &Path) {
        self.matches.clear();
        self.current = 0;

        if self.query.is_empty() {
            return;
        }

        let re = match RegexBuilder::new(&regex::escape(&self.query))
            .case_insensitive(self.case_insensitive)
            .build()
        {
            Ok(re) => re,
            Err(_) => return,
        };

        let supported = &["java", "kt", "log", "txt", "md", "mmd", "proto"];

        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            if !ext
                .as_ref()
                .is_some_and(|e| supported.contains(&e.as_str()))
            {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(path) {
                for (line_idx, line) in content.lines().enumerate() {
                    for m in re.find_iter(line) {
                        self.matches.push(SearchMatch {
                            file: Some(path.to_path_buf()),
                            line: line_idx,
                            col: m.start(),
                            text: line.trim_end().to_string(),
                        });
                        // Limit results per file
                        if self.matches.len() > 1000 {
                            return;
                        }
                    }
                }
            }
        }
    }

    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current = (self.current + 1) % self.matches.len();
        }
    }

    pub fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            self.current = if self.current == 0 {
                self.matches.len() - 1
            } else {
                self.current - 1
            };
        }
    }

    pub fn current_match(&self) -> Option<&SearchMatch> {
        self.matches.get(self.current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_empty_query() {
        let mut state = SearchState::new();
        let lines = vec!["hello".to_string(), "world".to_string()];
        state.search_in_buffer(&lines);
        assert!(state.matches.is_empty());
    }

    #[test]
    fn test_search_finds_matches() {
        let mut state = SearchState::new();
        state.query = "lo".to_string();
        let lines = vec!["hello".to_string(), "world".to_string()];
        state.search_in_buffer(&lines);
        assert_eq!(state.matches.len(), 1);
        assert_eq!(state.matches[0].line, 0);
        assert_eq!(state.matches[0].col, 3);
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut state = SearchState::new();
        state.query = "HELLO".to_string();
        state.case_insensitive = true;
        let lines = vec!["Hello World".to_string()];
        state.search_in_buffer(&lines);
        assert_eq!(state.matches.len(), 1);
    }

    #[test]
    fn test_search_multiple_matches_per_line() {
        let mut state = SearchState::new();
        state.query = "ab".to_string();
        let lines = vec!["ab cd ab ef ab".to_string()];
        state.search_in_buffer(&lines);
        assert_eq!(state.matches.len(), 3);
    }

    #[test]
    fn test_next_prev_match() {
        let mut state = SearchState::new();
        state.query = "a".to_string();
        let lines = vec!["a b a c a".to_string()];
        state.search_in_buffer(&lines);
        assert_eq!(state.matches.len(), 3);
        assert_eq!(state.current, 0);
        state.next_match();
        assert_eq!(state.current, 1);
        state.next_match();
        assert_eq!(state.current, 2);
        state.next_match();
        assert_eq!(state.current, 0); // wraps
        state.prev_match();
        assert_eq!(state.current, 2); // wraps back
    }

    #[test]
    fn test_search_across_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test.txt"), "hello world\nfoo bar").unwrap();
        std::fs::write(dir.path().join("other.txt"), "hello again").unwrap();

        let mut state = SearchState::new();
        state.query = "hello".to_string();
        state.search_across_files(dir.path());
        assert_eq!(state.matches.len(), 2);
        assert!(state.matches.iter().all(|m| m.file.is_some()));
    }

    #[test]
    fn test_search_no_results() {
        let mut state = SearchState::new();
        state.query = "xyz".to_string();
        let lines = vec!["hello".to_string()];
        state.search_in_buffer(&lines);
        assert!(state.matches.is_empty());
        assert!(state.current_match().is_none());
    }
}
