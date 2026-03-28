use crate::commands::delete_word_backward_pos;
use crate::state::{AppState, FocusPane};

/// Paste block marker character. Each occurrence in `composer` corresponds
/// to an entry in `paste_blocks` at the same ordinal position.
pub const PASTE_MARKER: char = '\u{FFFC}';

impl AppState {
    pub(crate) fn apply_input_char(&mut self, ch: char) {
        if self.focus == FocusPane::Composer {
            self.composer.insert(self.composer_cursor, ch);
            self.composer_cursor += ch.len_utf8();
        }
    }

    pub(crate) fn apply_backspace(&mut self) {
        if self.focus == FocusPane::Composer && self.composer_cursor > 0 {
            let prev = self.composer[..self.composer_cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            let deleted_char = self.composer[prev..].chars().next().unwrap();
            if deleted_char == PASTE_MARKER {
                let marker_index = self.composer[..prev]
                    .chars()
                    .filter(|&c| c == PASTE_MARKER)
                    .count();
                if marker_index < self.paste_blocks.len() {
                    self.paste_blocks.remove(marker_index);
                }
            }
            self.composer.remove(prev);
            self.composer_cursor = prev;
        }
    }

    pub(crate) fn apply_cursor_left(&mut self) {
        if self.focus == FocusPane::Composer && self.composer_cursor > 0 {
            self.composer_cursor = self.composer[..self.composer_cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    pub(crate) fn apply_cursor_right(&mut self) {
        if self.focus == FocusPane::Composer && self.composer_cursor < self.composer.len() {
            self.composer_cursor = self.composer[self.composer_cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.composer_cursor + i)
                .unwrap_or(self.composer.len());
        }
    }

    pub(crate) fn apply_cursor_home(&mut self) {
        if self.focus == FocusPane::Composer {
            self.composer_cursor = 0;
        }
    }

    pub(crate) fn apply_cursor_end(&mut self) {
        if self.focus == FocusPane::Composer {
            self.composer_cursor = self.composer.len();
        }
    }

    pub(crate) fn apply_delete_forward(&mut self) {
        if self.focus == FocusPane::Composer && self.composer_cursor < self.composer.len() {
            let deleted_char = self.composer[self.composer_cursor..].chars().next().unwrap();
            if deleted_char == PASTE_MARKER {
                let marker_index = self.composer[..self.composer_cursor]
                    .chars()
                    .filter(|&c| c == PASTE_MARKER)
                    .count();
                if marker_index < self.paste_blocks.len() {
                    self.paste_blocks.remove(marker_index);
                }
            }
            self.composer.remove(self.composer_cursor);
        }
    }

    pub(crate) fn apply_delete_word_backward(&mut self) {
        if self.focus == FocusPane::Composer && self.composer_cursor > 0 {
            let new_pos = delete_word_backward_pos(&self.composer, self.composer_cursor);
            self.remove_paste_blocks_in_range(new_pos, self.composer_cursor);
            self.composer.drain(new_pos..self.composer_cursor);
            self.composer_cursor = new_pos;
        }
    }

    pub(crate) fn apply_clear_to_start(&mut self) {
        if self.focus == FocusPane::Composer {
            self.remove_paste_blocks_in_range(0, self.composer_cursor);
            self.composer.drain(..self.composer_cursor);
            self.composer_cursor = 0;
        }
    }

    pub(crate) fn apply_paste(&mut self, text: String) {
        if self.focus != FocusPane::Composer {
            return;
        }
        let line_count = text.lines().count();
        if line_count < 2 {
            // Single-line paste: inline directly.
            self.composer.insert_str(self.composer_cursor, &text);
            self.composer_cursor += text.len();
        } else {
            // Multi-line paste: store as a collapsed block.
            let marker_index = self.composer[..self.composer_cursor]
                .chars()
                .filter(|&c| c == PASTE_MARKER)
                .count();
            self.paste_blocks.insert(marker_index, text);
            self.composer.insert(self.composer_cursor, PASTE_MARKER);
            self.composer_cursor += PASTE_MARKER.len_utf8();
        }
    }

    /// Remove paste block entries for any `\u{FFFC}` markers in
    /// `composer[start..end]`.
    fn remove_paste_blocks_in_range(&mut self, start: usize, end: usize) {
        let base = self.composer[..start]
            .chars()
            .filter(|&c| c == PASTE_MARKER)
            .count();
        let mut to_remove = Vec::new();
        let mut idx = base;
        for ch in self.composer[start..end].chars() {
            if ch == PASTE_MARKER {
                to_remove.push(idx);
                idx += 1;
            }
        }
        for i in to_remove.into_iter().rev() {
            if i < self.paste_blocks.len() {
                self.paste_blocks.remove(i);
            }
        }
    }

    /// Expand all paste block markers into the actual pasted text,
    /// returning the full submission string.
    pub fn expand_paste_blocks(&self) -> String {
        let mut expanded = String::new();
        let mut block_idx = 0usize;
        for ch in self.composer.chars() {
            if ch == PASTE_MARKER {
                if let Some(content) = self.paste_blocks.get(block_idx) {
                    expanded.push_str(content);
                }
                block_idx += 1;
            } else {
                expanded.push(ch);
            }
        }
        expanded
    }
}
