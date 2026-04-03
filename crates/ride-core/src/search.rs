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

            if !ext.as_ref().is_some_and(|e| supported.contains(&e.as_str())) {
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
