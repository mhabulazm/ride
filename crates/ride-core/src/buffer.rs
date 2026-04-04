use ropey::Rope;
use std::fs::File;
use std::io::BufReader;
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
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let rope = Rope::from_reader(reader)?;
        Ok(Self {
            rope,
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
            let file = File::create(path)?;
            let writer = std::io::BufWriter::new(file);
            self.rope.write_to(writer)?;
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
        // Capture leading whitespace from current line for auto-indent
        let indent = self
            .get_line(self.cursor_row)
            .map(|line| {
                line.chars()
                    .take_while(|c| *c == ' ' || *c == '\t')
                    .collect::<String>()
            })
            .unwrap_or_default();

        let idx = self.char_index_at_cursor();
        let mut insert_text = String::from('\n');
        insert_text.push_str(&indent);
        self.rope.insert(idx, &insert_text);
        self.cursor_row += 1;
        self.cursor_col = indent.len();
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

    pub fn move_word_left(&mut self) {
        if self.cursor_col == 0 {
            if self.cursor_row > 0 {
                self.cursor_row -= 1;
                self.cursor_col = self.line_len(self.cursor_row);
            }
            return;
        }
        let line = self.get_line(self.cursor_row).unwrap_or_default();
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor_col;
        // Skip whitespace/punctuation going left
        while col > 0 && !chars[col - 1].is_alphanumeric() && chars[col - 1] != '_' {
            col -= 1;
        }
        // Skip word chars going left
        while col > 0 && (chars[col - 1].is_alphanumeric() || chars[col - 1] == '_') {
            col -= 1;
        }
        self.cursor_col = col;
    }

    pub fn move_word_right(&mut self) {
        let line = self.get_line(self.cursor_row).unwrap_or_default();
        let len = self.line_len(self.cursor_row);
        if self.cursor_col >= len {
            if self.cursor_row + 1 < self.line_count() {
                self.cursor_row += 1;
                self.cursor_col = 0;
            }
            return;
        }
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor_col;
        // Skip word chars going right
        while col < len && (chars[col].is_alphanumeric() || chars[col] == '_') {
            col += 1;
        }
        // Skip whitespace/punctuation going right
        while col < len && !chars[col].is_alphanumeric() && chars[col] != '_' {
            col += 1;
        }
        self.cursor_col = col;
    }

    pub fn go_to_line(&mut self, line: usize) {
        let target = line.saturating_sub(1); // 1-based to 0-based
        self.cursor_row = target.min(self.line_count().saturating_sub(1));
        self.cursor_col = 0;
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

    /// Find the matching bracket for the character at the cursor position.
    /// Returns `Some((row, col))` if a match is found.
    pub fn find_matching_bracket(&self) -> Option<(usize, usize)> {
        let line = self.get_line(self.cursor_row)?;
        let ch = line.chars().nth(self.cursor_col)?;

        let (target, forward) = match ch {
            '(' => (')', true),
            '{' => ('}', true),
            '[' => (']', true),
            ')' => ('(', false),
            '}' => ('{', false),
            ']' => ('[', false),
            _ => return None,
        };

        if forward {
            self.search_bracket_forward(ch, target)
        } else {
            self.search_bracket_backward(ch, target)
        }
    }

    fn search_bracket_forward(&self, open: char, close: char) -> Option<(usize, usize)> {
        let mut depth = 0i32;
        for row in self.cursor_row..self.line_count() {
            let line = self.get_line(row)?;
            let start_col = if row == self.cursor_row {
                self.cursor_col
            } else {
                0
            };
            for (col, ch) in line.chars().enumerate().skip(start_col) {
                if ch == open {
                    depth += 1;
                } else if ch == close {
                    depth -= 1;
                    if depth == 0 {
                        return Some((row, col));
                    }
                }
            }
        }
        None
    }

    fn search_bracket_backward(&self, close: char, open: char) -> Option<(usize, usize)> {
        let mut depth = 0i32;
        for row in (0..=self.cursor_row).rev() {
            let line = self.get_line(row)?;
            let chars: Vec<char> = line.chars().collect();
            let end_col = if row == self.cursor_row {
                self.cursor_col
            } else {
                chars.len().saturating_sub(1)
            };
            for col in (0..=end_col).rev() {
                if col >= chars.len() {
                    continue;
                }
                let ch = chars[col];
                if ch == close {
                    depth += 1;
                } else if ch == open {
                    depth -= 1;
                    if depth == 0 {
                        return Some((row, col));
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf_from(text: &str) -> Buffer {
        Buffer {
            rope: Rope::from_str(text),
            file_path: None,
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            dirty: false,
            undo_stack: Vec::new(),
        }
    }

    #[test]
    fn test_empty_buffer() {
        let buf = Buffer::empty();
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.cursor_row, 0);
        assert_eq!(buf.cursor_col, 0);
        assert!(!buf.dirty);
        assert_eq!(buf.file_name(), "[untitled]");
    }

    #[test]
    fn test_insert_char() {
        let mut buf = Buffer::empty();
        buf.insert_char('a');
        buf.insert_char('b');
        assert_eq!(buf.get_line(0).unwrap(), "ab");
        assert_eq!(buf.cursor_col, 2);
        assert!(buf.dirty);
    }

    #[test]
    fn test_insert_newline() {
        let mut buf = buf_from("hello");
        buf.cursor_col = 3;
        buf.insert_newline();
        assert_eq!(buf.get_line(0).unwrap(), "hel\n");
        assert_eq!(buf.get_line(1).unwrap(), "lo");
        assert_eq!(buf.cursor_row, 1);
        assert_eq!(buf.cursor_col, 0);
    }

    #[test]
    fn test_insert_newline_auto_indent_spaces() {
        let mut buf = buf_from("    hello");
        buf.cursor_col = 9;
        buf.insert_newline();
        assert_eq!(buf.get_line(1).unwrap(), "    ");
        assert_eq!(buf.cursor_row, 1);
        assert_eq!(buf.cursor_col, 4);
    }

    #[test]
    fn test_insert_newline_auto_indent_tabs() {
        let mut buf = buf_from("\t\thello");
        buf.cursor_col = 7;
        buf.insert_newline();
        assert_eq!(buf.cursor_row, 1);
        assert_eq!(buf.cursor_col, 2);
    }

    #[test]
    fn test_insert_newline_auto_indent_mid_line() {
        let mut buf = buf_from("  foo bar");
        buf.cursor_col = 5; // between "fo" and "o bar"
        buf.insert_newline();
        assert_eq!(buf.get_line(0).unwrap(), "  foo\n");
        assert_eq!(buf.get_line(1).unwrap(), "   bar");
        assert_eq!(buf.cursor_col, 2);
    }

    #[test]
    fn test_delete_back() {
        let mut buf = buf_from("hello");
        buf.cursor_col = 3;
        buf.delete_back();
        assert_eq!(buf.get_line(0).unwrap(), "helo");
        assert_eq!(buf.cursor_col, 2);
    }

    #[test]
    fn test_delete_back_at_line_start_joins_lines() {
        let mut buf = buf_from("abc\ndef");
        buf.cursor_row = 1;
        buf.cursor_col = 0;
        buf.delete_back();
        assert_eq!(buf.cursor_row, 0);
        assert_eq!(buf.cursor_col, 3);
        assert_eq!(buf.get_line(0).unwrap(), "abcdef");
    }

    #[test]
    fn test_delete_forward() {
        let mut buf = buf_from("hello");
        buf.cursor_col = 1;
        buf.delete_forward();
        assert_eq!(buf.get_line(0).unwrap(), "hllo");
    }

    #[test]
    fn test_undo() {
        let mut buf = buf_from("hello");
        buf.insert_char('!');
        assert_eq!(buf.get_line(0).unwrap(), "!hello");
        buf.undo();
        assert_eq!(buf.get_line(0).unwrap(), "hello");
    }

    #[test]
    fn test_cursor_movement() {
        let mut buf = buf_from("abc\ndef\nghi");
        buf.move_down();
        assert_eq!(buf.cursor_row, 1);
        buf.move_right();
        buf.move_right();
        assert_eq!(buf.cursor_col, 2);
        buf.move_up();
        assert_eq!(buf.cursor_row, 0);
        assert_eq!(buf.cursor_col, 2);
        buf.move_left();
        assert_eq!(buf.cursor_col, 1);
    }

    #[test]
    fn test_move_left_wraps_to_previous_line() {
        let mut buf = buf_from("abc\ndef");
        buf.cursor_row = 1;
        buf.cursor_col = 0;
        buf.move_left();
        assert_eq!(buf.cursor_row, 0);
        assert_eq!(buf.cursor_col, 3);
    }

    #[test]
    fn test_move_right_wraps_to_next_line() {
        let mut buf = buf_from("abc\ndef");
        buf.cursor_col = 3;
        buf.move_right();
        assert_eq!(buf.cursor_row, 1);
        assert_eq!(buf.cursor_col, 0);
    }

    #[test]
    fn test_move_to_line_start_end() {
        let mut buf = buf_from("hello world");
        buf.cursor_col = 5;
        buf.move_to_line_start();
        assert_eq!(buf.cursor_col, 0);
        buf.move_to_line_end();
        assert_eq!(buf.cursor_col, 11);
    }

    #[test]
    fn test_move_to_file_start_end() {
        let mut buf = buf_from("line1\nline2\nline3");
        buf.cursor_row = 1;
        buf.cursor_col = 3;
        buf.move_to_file_start();
        assert_eq!(buf.cursor_row, 0);
        assert_eq!(buf.cursor_col, 0);
        buf.move_to_file_end();
        assert_eq!(buf.cursor_row, 2);
        assert_eq!(buf.cursor_col, 5);
    }

    #[test]
    fn test_page_up_down() {
        let mut buf = buf_from(&"line\n".repeat(100));
        buf.page_down(20);
        assert_eq!(buf.cursor_row, 20);
        buf.page_up(10);
        assert_eq!(buf.cursor_row, 10);
        buf.page_up(100);
        assert_eq!(buf.cursor_row, 0);
    }

    #[test]
    fn test_clamp_cursor_col() {
        let mut buf = buf_from("long line\nhi");
        buf.cursor_col = 8;
        buf.move_down();
        // "hi" has length 2, cursor should clamp
        assert_eq!(buf.cursor_col, 2);
    }

    // --- Bracket matching tests ---

    #[test]
    fn test_bracket_match_forward_parens() {
        let mut buf = buf_from("fn(a, b)");
        buf.cursor_col = 2; // on '('
        assert_eq!(buf.find_matching_bracket(), Some((0, 7)));
    }

    #[test]
    fn test_bracket_match_backward_parens() {
        let mut buf = buf_from("fn(a, b)");
        buf.cursor_col = 7; // on ')'
        assert_eq!(buf.find_matching_bracket(), Some((0, 2)));
    }

    #[test]
    fn test_bracket_match_nested() {
        let mut buf = buf_from("{a{b}c}");
        buf.cursor_col = 0; // on outer '{'
        assert_eq!(buf.find_matching_bracket(), Some((0, 6)));
    }

    #[test]
    fn test_bracket_match_multiline() {
        let mut buf = buf_from("{\n  foo\n}");
        buf.cursor_col = 0; // on '{'
        assert_eq!(buf.find_matching_bracket(), Some((2, 0)));
    }

    #[test]
    fn test_bracket_match_multiline_backward() {
        let mut buf = buf_from("{\n  foo\n}");
        buf.cursor_row = 2;
        buf.cursor_col = 0; // on '}'
        assert_eq!(buf.find_matching_bracket(), Some((0, 0)));
    }

    #[test]
    fn test_bracket_no_match() {
        let mut buf = buf_from("(unclosed");
        buf.cursor_col = 0;
        assert_eq!(buf.find_matching_bracket(), None);
    }

    #[test]
    fn test_bracket_not_a_bracket() {
        let mut buf = buf_from("hello");
        buf.cursor_col = 0;
        assert_eq!(buf.find_matching_bracket(), None);
    }

    #[test]
    fn test_bracket_square() {
        let mut buf = buf_from("a[b[c]]");
        buf.cursor_col = 1; // on first '['
        assert_eq!(buf.find_matching_bracket(), Some((0, 6)));
    }

    #[test]
    fn test_update_scroll() {
        let mut buf = buf_from(&"line\n".repeat(100));
        buf.cursor_row = 50;
        buf.update_scroll(20, 80);
        assert!(buf.scroll_row <= 50);
        assert!(buf.scroll_row + 20 > 50);
    }

    #[test]
    fn test_file_name_with_path() {
        let buf = Buffer {
            rope: Rope::new(),
            file_path: Some(PathBuf::from("/tmp/test.txt")),
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            dirty: false,
            undo_stack: Vec::new(),
        };
        assert_eq!(buf.file_name(), "test.txt");
    }

    // --- Word movement tests ---

    #[test]
    fn test_move_word_right_basic() {
        let mut buf = buf_from("hello world foo");
        buf.move_word_right();
        assert_eq!(buf.cursor_col, 6); // after "hello "
    }

    #[test]
    fn test_move_word_right_from_middle() {
        let mut buf = buf_from("hello world");
        buf.cursor_col = 2;
        buf.move_word_right();
        assert_eq!(buf.cursor_col, 6); // after "hello "
    }

    #[test]
    fn test_move_word_left_basic() {
        let mut buf = buf_from("hello world");
        buf.cursor_col = 11;
        buf.move_word_left();
        assert_eq!(buf.cursor_col, 6); // start of "world"
    }

    #[test]
    fn test_move_word_left_at_start() {
        let mut buf = buf_from("hello world");
        buf.cursor_col = 0;
        buf.move_word_left(); // no-op at start of first line
        assert_eq!(buf.cursor_col, 0);
        assert_eq!(buf.cursor_row, 0);
    }

    #[test]
    fn test_move_word_right_at_end_wraps() {
        let mut buf = buf_from("hello\nworld");
        buf.cursor_col = 5;
        buf.move_word_right();
        assert_eq!(buf.cursor_row, 1);
        assert_eq!(buf.cursor_col, 0);
    }

    #[test]
    fn test_move_word_left_at_start_wraps() {
        let mut buf = buf_from("hello\nworld");
        buf.cursor_row = 1;
        buf.cursor_col = 0;
        buf.move_word_left();
        assert_eq!(buf.cursor_row, 0);
        assert_eq!(buf.cursor_col, 5);
    }

    #[test]
    fn test_move_word_right_with_punctuation() {
        let mut buf = buf_from("foo(bar)");
        buf.move_word_right();
        assert_eq!(buf.cursor_col, 4); // skips "foo(" to start of "bar"
    }

    // --- Go-to-line tests ---

    #[test]
    fn test_go_to_line() {
        let mut buf = buf_from("line1\nline2\nline3\nline4");
        buf.go_to_line(3);
        assert_eq!(buf.cursor_row, 2);
        assert_eq!(buf.cursor_col, 0);
    }

    #[test]
    fn test_go_to_line_zero_clamps() {
        let mut buf = buf_from("line1\nline2");
        buf.go_to_line(0);
        assert_eq!(buf.cursor_row, 0);
    }

    #[test]
    fn test_go_to_line_past_end_clamps() {
        let mut buf = buf_from("line1\nline2");
        buf.go_to_line(999);
        assert_eq!(buf.cursor_row, 1); // last line
    }

    // --- Large file streaming tests ---

    #[test]
    fn test_from_file_and_save_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "hello\nworld\n").unwrap();

        let mut buf = Buffer::from_file(&path).unwrap();
        assert_eq!(buf.line_count(), 3); // "hello\n", "world\n", ""
        assert_eq!(buf.get_line(0).unwrap(), "hello\n");

        buf.cursor_col = 5;
        buf.insert_char('!');
        buf.save().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("hello!"));
    }
}
