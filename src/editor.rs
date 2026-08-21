#[derive(Debug, Clone)]
pub struct Editor {
    pub lines: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub scroll_y: usize,
    pub is_modified: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            scroll_y: 0,
            is_modified: false,
        }
    }
}

impl Editor {
    pub fn from_string(content: &str) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };

        Self {
            lines,
            cursor_x: 0,
            cursor_y: 0,
            scroll_y: 0,
            is_modified: false,
        }
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    pub fn insert_char(&mut self, ch: char) {
        if self.cursor_y >= self.lines.len() {
            self.lines.resize(self.cursor_y + 1, String::new());
        }
        let line = &mut self.lines[self.cursor_y];
        let idx = clamp_col_byte_index(line, self.cursor_x);
        line.insert(idx, ch);
        self.cursor_x += 1;
        self.is_modified = true;
    }

    pub fn insert_str(&mut self, s: &str) {
        for ch in s.chars() {
            if ch == '\n' {
                self.insert_newline();
            } else {
                self.insert_char(ch);
            }
        }
    }

    pub fn insert_newline(&mut self) {
        if self.cursor_y >= self.lines.len() {
            self.lines.resize(self.cursor_y + 1, String::new());
        }
        let line = &mut self.lines[self.cursor_y];
        let idx = clamp_col_byte_index(line, self.cursor_x);
        let remainder = line.drain(idx..).collect::<String>();
        self.cursor_y += 1;
        self.lines.insert(self.cursor_y, remainder);
        self.cursor_x = 0;
        self.is_modified = true;
    }

    pub fn backspace(&mut self) {
        if self.cursor_y >= self.lines.len() {
            return;
        }

        if self.cursor_x > 0 {
            let line = &mut self.lines[self.cursor_y];
            self.cursor_x -= 1;
            let idx = clamp_col_byte_index(line, self.cursor_x);
            if idx < line.len() {
                line.remove(idx);
            }
            self.is_modified = true;
        } else if self.cursor_y > 0 {
            // Merge with previous line
            let current_line = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            let prev_line = &mut self.lines[self.cursor_y];
            self.cursor_x = prev_line.chars().count();
            prev_line.push_str(&current_line);
            self.is_modified = true;
        }
    }

    pub fn delete_char(&mut self) {
        if self.cursor_y >= self.lines.len() {
            return;
        }

        let line_len = self.lines[self.cursor_y].chars().count();
        if self.cursor_x < line_len {
            let line = &mut self.lines[self.cursor_y];
            let idx = clamp_col_byte_index(line, self.cursor_x);
            line.remove(idx);
            self.is_modified = true;
        } else if self.cursor_y < self.lines.len() - 1 {
            let next_line = self.lines.remove(self.cursor_y + 1);
            self.lines[self.cursor_y].push_str(&next_line);
            self.is_modified = true;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.clamp_cursor_x();
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            self.clamp_cursor_x();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].chars().count();
        }
    }

    pub fn move_right(&mut self) {
        let line_len = self.lines.get(self.cursor_y).map(|l| l.chars().count()).unwrap_or(0);
        if self.cursor_x < line_len {
            self.cursor_x += 1;
        } else if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor_x = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor_x = self.lines.get(self.cursor_y).map(|l| l.chars().count()).unwrap_or(0);
    }

    pub fn insert_color_tag(&mut self, hex_color: &str) {
        let open_tag = format!("<span style=\"color:{}\">", hex_color);
        let close_tag = "</span>";
        self.insert_str(&open_tag);
        self.insert_str("colored text");
        self.insert_str(close_tag);
    }

    pub fn insert_mermaid_template(&mut self) {
        let template = "\n```mermaid\ngraph TD\n    A[Start] --> B[Process]\n    B --> C[Done]\n```\n";
        self.insert_str(template);
    }

    fn clamp_cursor_x(&mut self) {
        let max_x = self.lines.get(self.cursor_y).map(|l| l.chars().count()).unwrap_or(0);
        if self.cursor_x > max_x {
            self.cursor_x = max_x;
        }
    }
}

fn clamp_col_byte_index(line: &str, col_char: usize) -> usize {
    line.char_indices()
        .nth(col_char)
        .map(|(idx, _)| idx)
        .unwrap_or(line.len())
}
