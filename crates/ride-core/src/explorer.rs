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

    pub fn create_file(&mut self, name: &str) -> std::io::Result<PathBuf> {
        let parent = self.selected_parent_dir();
        let path = parent.join(name);
        std::fs::File::create(&path)?;
        self.refresh_keeping_selection();
        Ok(path)
    }

    pub fn create_folder(&mut self, name: &str) -> std::io::Result<PathBuf> {
        let parent = self.selected_parent_dir();
        let path = parent.join(name);
        std::fs::create_dir_all(&path)?;
        self.refresh_keeping_selection();
        Ok(path)
    }

    pub fn rename_selected(&mut self, new_name: &str) -> std::io::Result<PathBuf> {
        if let Some(entry) = self.entries.get(self.selected) {
            let old_path = entry.path.clone();
            let new_path = old_path.parent().unwrap_or(&self.root).join(new_name);
            std::fs::rename(&old_path, &new_path)?;
            self.refresh_keeping_selection();
            Ok(new_path)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No selection",
            ))
        }
    }

    pub fn delete_selected(&mut self) -> std::io::Result<()> {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                std::fs::remove_dir_all(&entry.path)?;
            } else {
                std::fs::remove_file(&entry.path)?;
            }
            self.refresh_keeping_selection();
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No selection",
            ))
        }
    }

    fn selected_parent_dir(&self) -> PathBuf {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                entry.path.clone()
            } else {
                entry.path.parent().unwrap_or(&self.root).to_path_buf()
            }
        } else {
            self.root.clone()
        }
    }

    fn refresh_keeping_selection(&mut self) {
        let old_selected_path = self.entries.get(self.selected).map(|e| e.path.clone());
        self.refresh();
        if let Some(old_path) = old_selected_path {
            if let Some(idx) = self.entries.iter().position(|e| e.path == old_path) {
                self.selected = idx;
            } else {
                self.selected = self.selected.min(self.entries.len().saturating_sub(1));
            }
        }
    }
}
