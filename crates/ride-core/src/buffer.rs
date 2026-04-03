use ropey::Rope;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Buffer {
    pub rope: Rope,
    pub file_path: Option<PathBuf>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub dirty: bool,
    undo_stack: Vec<(Rope, usize, usize)>,
}

impl Buffer {
    pub fn empty() -> Self {
        Self {
            rope: Rope::new(),
            file_path: None,
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            dirty: false,
            undo_stack: Vec::new(),
        }
    }

    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        let text = fs::read_to_string(path)?;
        Ok(Self {
            rope: Rope::from_str(&text),
            file_path: Some(path.to_path_buf()),
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            dirty: false,
            undo_stack: Vec::new(),
        })
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(ref path) = self.file_path {
            fs::write(path, self.rope.to_string())?;
            self.dirty = false;
        }
        Ok(())
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn get_line(&self, idx: usize) -> Option<String> {
        if idx < self.rope.len_lines() {
            Some(self.rope.line(idx).to_string())
        } else {
            None
        }
    }

    fn save_undo(&mut self) {
        self.undo_stack
            .push((self.rope.clone(), self.cursor_row, self.cursor_col));
    }

    pub fn undo(&mut self) {
        if let Some((rope, row, col)) = self.undo_stack.pop() {
            self.rope = rope;
            self.cursor_row = row;
            self.cursor_col = col;
            self.dirty = true;
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.save_undo();
        let idx = self.char_index_at_cursor();
        self.rope.insert_char(idx, ch);
        self.cursor_col += 1;
        self.dirty = true;
    }

    pub fn insert_newline(&mut self) {
        self.save_undo();
        let idx = self.char_index_at_cursor();
        self.rope.insert_char(idx, '\n');
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.dirty = true;
    }

    pub fn delete_back(&mut self) {
        self.save_undo();
        if self.cursor_col > 0 {
            let idx = self.char_index_at_cursor();
            self.rope.remove(idx - 1..idx);
            self.cursor_col -= 1;
            self.dirty = true;
        } else if self.cursor_row > 0 {
            // Join with previous line
            let prev_line_len = self.line_len(self.cursor_row - 1);
            let idx = self.char_index_at_cursor();
            self.rope.remove(idx - 1..idx);
            self.cursor_row -= 1;
            self.cursor_col = prev_line_len;
            self.dirty = true;
        }
    }

    pub fn delete_forward(&mut self) {
        self.save_undo();
        let idx = self.char_index_at_cursor();
        if idx < self.rope.len_chars() {
            self.rope.remove(idx..idx + 1);
            self.dirty = true;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.clamp_cursor_col();
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_row + 1 < self.line_count() {
            self.cursor_row += 1;
            self.clamp_cursor_col();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.line_len(self.cursor_row);
        }
    }

    pub fn move_right(&mut self) {
        let len = self.line_len(self.cursor_row);
        if self.cursor_col < len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.line_count() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_to_line_start(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_to_line_end(&mut self) {
        self.cursor_col = self.line_len(self.cursor_row);
    }

    pub fn move_to_file_start(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    pub fn move_to_file_end(&mut self) {
        self.cursor_row = self.line_count().saturating_sub(1);
        self.cursor_col = self.line_len(self.cursor_row);
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.cursor_row = self.cursor_row.saturating_sub(page_size);
        self.clamp_cursor_col();
    }

    pub fn page_down(&mut self, page_size: usize) {
        self.cursor_row = (self.cursor_row + page_size).min(self.line_count().saturating_sub(1));
        self.clamp_cursor_col();
    }

    pub fn update_scroll(&mut self, viewport_height: usize, viewport_width: usize) {
        // Vertical scroll
        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        }
        if self.cursor_row >= self.scroll_row + viewport_height {
            self.scroll_row = self.cursor_row - viewport_height + 1;
        }
        // Horizontal scroll
        if self.cursor_col < self.scroll_col {
            self.scroll_col = self.cursor_col;
        }
        if self.cursor_col >= self.scroll_col + viewport_width {
            self.scroll_col = self.cursor_col - viewport_width + 1;
        }
    }

    fn line_len(&self, row: usize) -> usize {
        if row >= self.rope.len_lines() {
            return 0;
        }
        let line = self.rope.line(row);
        let len = line.len_chars();
        // Exclude trailing newline
        if len > 0 && line.char(len - 1) == '\n' {
            len - 1
        } else {
            len
        }
    }

    fn clamp_cursor_col(&mut self) {
        let len = self.line_len(self.cursor_row);
        if self.cursor_col > len {
            self.cursor_col = len;
        }
    }

    fn char_index_at_cursor(&self) -> usize {
        let line_start = self.rope.line_to_char(self.cursor_row);
        line_start + self.cursor_col
    }

    pub fn file_name(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "[untitled]".to_string())
    }
}
