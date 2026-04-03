use crate::buffer::Buffer;
use std::path::Path;

pub struct TabManager {
    pub tabs: Vec<Buffer>,
    pub active: usize,
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
