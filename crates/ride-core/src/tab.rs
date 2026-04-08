use crate::buffer::Buffer;
use std::path::Path;

pub struct TabManager {
    pub tabs: Vec<Buffer>,
    pub active: usize,
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
        }
    }

    pub fn open_file(&mut self, path: &Path) -> std::io::Result<()> {
        // Check if already open
        for (i, tab) in self.tabs.iter().enumerate() {
            if tab.file_path.as_deref() == Some(path) {
                self.active = i;
                return Ok(());
            }
        }
        let buf = Buffer::from_file(path)?;
        self.tabs.push(buf);
        self.active = self.tabs.len() - 1;
        Ok(())
    }

    pub fn open_empty(&mut self) {
        self.tabs.push(Buffer::empty());
        self.active = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self) -> bool {
        if self.tabs.is_empty() {
            return false;
        }
        self.tabs.remove(self.active);
        if self.active >= self.tabs.len() && self.active > 0 {
            self.active -= 1;
        }
        !self.tabs.is_empty()
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active = (self.active + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active = if self.active == 0 {
                self.tabs.len() - 1
            } else {
                self.active - 1
            };
        }
    }

    pub fn active_buffer(&self) -> Option<&Buffer> {
        self.tabs.get(self.active)
    }

    pub fn active_buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.tabs.get_mut(self.active)
    }

    pub fn has_unsaved(&self) -> bool {
        self.tabs.iter().any(|t| t.dirty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_file(name: &str, content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        (dir, path)
    }

    #[test]
    fn test_new_tab_manager_is_empty() {
        let tm = TabManager::new();
        assert!(tm.tabs.is_empty());
        assert_eq!(tm.active, 0);
        assert!(tm.active_buffer().is_none());
    }

    #[test]
    fn test_open_file() {
        let (_dir, path) = temp_file("test.txt", "hello");
        let mut tm = TabManager::new();
        tm.open_file(&path).unwrap();
        assert_eq!(tm.tabs.len(), 1);
        assert_eq!(tm.active, 0);
        assert!(tm.active_buffer().is_some());
    }

    #[test]
    fn test_open_same_file_twice_reuses_tab() {
        let (_dir, path) = temp_file("test.txt", "hello");
        let mut tm = TabManager::new();
        tm.open_file(&path).unwrap();
        tm.open_file(&path).unwrap();
        assert_eq!(tm.tabs.len(), 1);
    }

    #[test]
    fn test_open_empty() {
        let mut tm = TabManager::new();
        tm.open_empty();
        assert_eq!(tm.tabs.len(), 1);
        assert_eq!(tm.active_buffer().unwrap().file_name(), "[untitled]");
    }

    #[test]
    fn test_next_prev_tab() {
        let mut tm = TabManager::new();
        tm.open_empty();
        tm.open_empty();
        tm.open_empty();
        assert_eq!(tm.active, 2);
        tm.next_tab();
        assert_eq!(tm.active, 0); // wraps around
        tm.prev_tab();
        assert_eq!(tm.active, 2); // wraps back
        tm.prev_tab();
        assert_eq!(tm.active, 1);
    }

    #[test]
    fn test_close_tab() {
        let mut tm = TabManager::new();
        tm.open_empty();
        tm.open_empty();
        assert_eq!(tm.tabs.len(), 2);
        tm.close_tab();
        assert_eq!(tm.tabs.len(), 1);
        assert_eq!(tm.active, 0);
    }

    #[test]
    fn test_close_last_tab_returns_false() {
        let mut tm = TabManager::new();
        tm.open_empty();
        let has_more = tm.close_tab();
        assert!(!has_more);
    }

    #[test]
    fn test_has_unsaved() {
        let mut tm = TabManager::new();
        tm.open_empty();
        assert!(!tm.has_unsaved());
        tm.active_buffer_mut().unwrap().insert_char('x');
        assert!(tm.has_unsaved());
    }
}
