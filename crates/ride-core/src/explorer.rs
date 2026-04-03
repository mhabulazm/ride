use std::path::{Path, PathBuf};

const SUPPORTED_EXTENSIONS: &[&str] = &["java", "kt", "log", "txt", "md", "mmd", "proto"];

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

pub struct Explorer {
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub root: PathBuf,
    pub visible: bool,
}

impl Explorer {
    pub fn new(root: &Path) -> Self {
        let mut explorer = Self {
            entries: Vec::new(),
            selected: 0,
            root: root.to_path_buf(),
            visible: true,
        };
        explorer.refresh();
        explorer
    }

    pub fn refresh(&mut self) {
        self.entries.clear();
        self.build_tree(&self.root.clone(), 0);
    }

    fn build_tree(&mut self, dir: &Path, depth: usize) {
        let mut items: Vec<_> = match std::fs::read_dir(dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .map(|e| (e.path(), e.file_type().ok()))
                .collect(),
            Err(_) => return,
        };

        // Sort: directories first, then alphabetical
        items.sort_by(|(a_path, a_ft), (b_path, b_ft)| {
            let a_dir = a_ft.as_ref().is_some_and(|ft| ft.is_dir());
            let b_dir = b_ft.as_ref().is_some_and(|ft| ft.is_dir());
            b_dir.cmp(&a_dir).then_with(|| a_path.cmp(b_path))
        });

        for (path, ft) in items {
            let is_dir = ft.as_ref().is_some_and(|f| f.is_dir());
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Skip hidden files/dirs
            if name.starts_with('.') {
                continue;
            }

            if is_dir {
                let entry = FileEntry {
                    path: path.clone(),
                    name,
                    is_dir: true,
                    depth,
                    expanded: false,
                };
                self.entries.push(entry);
            } else if Self::is_supported(&path) {
                self.entries.push(FileEntry {
                    path,
                    name,
                    is_dir: false,
                    depth,
                    expanded: false,
                });
            }
        }
    }

    fn is_supported(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| {
                let lower = e.to_lowercase();
                SUPPORTED_EXTENSIONS.contains(&lower.as_str())
            })
            .unwrap_or(false)
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }

    pub fn toggle_or_select(&mut self) -> Option<PathBuf> {
        if let Some(entry) = self.entries.get(self.selected).cloned() {
            if entry.is_dir {
                self.toggle_dir(self.selected);
                None
            } else {
                Some(entry.path)
            }
        } else {
            None
        }
    }

    fn toggle_dir(&mut self, idx: usize) {
        let entry = &self.entries[idx];
        let was_expanded = entry.expanded;
        let dir_path = entry.path.clone();
        let depth = entry.depth;

        if was_expanded {
            // Collapse: remove children
            self.entries[idx].expanded = false;
            let mut remove_count = 0;
            for i in (idx + 1)..self.entries.len() {
                if self.entries[i].depth > depth {
                    remove_count += 1;
                } else {
                    break;
                }
            }
            self.entries.drain(idx + 1..idx + 1 + remove_count);
        } else {
            // Expand: insert children after this entry
            self.entries[idx].expanded = true;
            let mut children = Vec::new();
            Self::collect_children(&dir_path, depth + 1, &mut children);
            // Insert children right after the directory entry
            for (offset, child) in children.into_iter().enumerate() {
                self.entries.insert(idx + 1 + offset, child);
            }
        }
    }

    fn collect_children(dir: &Path, depth: usize, out: &mut Vec<FileEntry>) {
        let mut items: Vec<_> = match std::fs::read_dir(dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .map(|e| (e.path(), e.file_type().ok()))
                .collect(),
            Err(_) => return,
        };

        items.sort_by(|(a_path, a_ft), (b_path, b_ft)| {
            let a_dir = a_ft.as_ref().is_some_and(|ft| ft.is_dir());
            let b_dir = b_ft.as_ref().is_some_and(|ft| ft.is_dir());
            b_dir.cmp(&a_dir).then_with(|| a_path.cmp(b_path))
        });

        for (path, ft) in items {
            let is_dir = ft.as_ref().is_some_and(|f| f.is_dir());
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if name.starts_with('.') {
                continue;
            }

            if is_dir {
                out.push(FileEntry {
                    path,
                    name,
                    is_dir: true,
                    depth,
                    expanded: false,
                });
            } else if Explorer::is_supported(&path) {
                out.push(FileEntry {
                    path,
                    name,
                    is_dir: false,
                    depth,
                    expanded: false,
                });
            }
        }
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected)
    }
}
