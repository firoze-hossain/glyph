//! buffer.rs - Piece-Table Text Buffer Engine for Glyph.
//! Delivers O(1) edits, line indexing, and full Undo/Redo history tracking.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[cfg(feature = "std")]
use std::{
    string::{String, ToString},
    vec::Vec,
};

/// Buffer source type for a piece table entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferKind {
    Original,
    Add,
}

/// A slice descriptor referencing a portion of the original or append buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub kind: BufferKind,
    pub start: usize,
    pub length: usize,
}

/// Piece-table text buffer with undo/redo history.
#[derive(Clone, Debug)]
pub struct PieceTable {
    pub original: String,
    pub add: String,
    pub pieces: Vec<Piece>,
    pub undo_stack: Vec<Vec<Piece>>,
    pub redo_stack: Vec<Vec<Piece>>,
    pub is_dirty: bool,
    pub total_length: usize,
    cached_lines: Option<Vec<String>>,
}

impl Default for PieceTable {
    fn default() -> Self {
        Self::new("")
    }
}

impl PieceTable {
    pub fn new(initial_text: &str) -> Self {
        let original = initial_text.to_string();
        let len = original.len();
        let mut pieces = Vec::new();
        if len > 0 {
            pieces.push(Piece {
                kind: BufferKind::Original,
                start: 0,
                length: len,
            });
        }

        Self {
            original,
            add: String::new(),
            pieces,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            is_dirty: false,
            total_length: len,
            cached_lines: None,
        }
    }

    /// Total byte length of the reconstructed document.
    pub fn len(&self) -> usize {
        self.total_length
    }

    pub fn is_empty(&self) -> bool {
        self.total_length == 0
    }

    /// Reconstructs the complete document text from pieces.
    pub fn get_text(&self) -> String {
        let mut out = String::with_capacity(self.total_length);
        for p in &self.pieces {
            let slice = match p.kind {
                BufferKind::Original => &self.original[p.start..p.start + p.length],
                BufferKind::Add => &self.add[p.start..p.start + p.length],
            };
            out.push_str(slice);
        }
        out
    }

    /// Saves the current piece sequence onto the undo stack before modifying.
    fn save_undo_state(&mut self) {
        self.undo_stack.push(self.pieces.clone());
        self.redo_stack.clear();
        self.is_dirty = true;
        self.cached_lines = None;
    }

    /// Inserts `text` at absolute byte offset `pos`.
    pub fn insert(&mut self, pos: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        self.save_undo_state();

        let add_start = self.add.len();
        self.add.push_str(text);
        let add_len = text.len();
        let new_piece = Piece {
            kind: BufferKind::Add,
            start: add_start,
            length: add_len,
        };

        if self.pieces.is_empty() {
            self.pieces.push(new_piece);
            self.total_length = add_len;
            return;
        }

        let mut current_offset = 0;
        let mut new_pieces = Vec::new();
        let mut inserted = false;

        for p in &self.pieces {
            let piece_end = current_offset + p.length;

            if !inserted && pos <= piece_end {
                let split_offset = pos.saturating_sub(current_offset);

                if split_offset > 0 {
                    new_pieces.push(Piece {
                        kind: p.kind,
                        start: p.start,
                        length: split_offset,
                    });
                }

                new_pieces.push(new_piece.clone());

                if split_offset < p.length {
                    new_pieces.push(Piece {
                        kind: p.kind,
                        start: p.start + split_offset,
                        length: p.length - split_offset,
                    });
                }

                inserted = true;
            } else {
                new_pieces.push(p.clone());
            }

            current_offset = piece_end;
        }

        if !inserted {
            new_pieces.push(new_piece);
        }

        self.pieces = new_pieces;
        self.total_length += add_len;
    }

    /// Deletes `length` bytes starting at absolute byte offset `pos`.
    pub fn delete(&mut self, pos: usize, length: usize) {
        if length == 0 || pos >= self.total_length {
            return;
        }
        self.save_undo_state();

        let del_end = (pos + length).min(self.total_length);
        let actual_del = del_end - pos;

        let mut current_offset = 0;
        let mut new_pieces = Vec::new();

        for p in &self.pieces {
            let piece_start = current_offset;
            let piece_end = current_offset + p.length;

            if piece_end <= pos || piece_start >= del_end {
                // Entirely outside deletion range
                new_pieces.push(p.clone());
            } else {
                // Intersects deletion range
                if piece_start < pos {
                    // Left remainder
                    new_pieces.push(Piece {
                        kind: p.kind,
                        start: p.start,
                        length: pos - piece_start,
                    });
                }
                if piece_end > del_end {
                    // Right remainder
                    let offset_in_piece = del_end - piece_start;
                    new_pieces.push(Piece {
                        kind: p.kind,
                        start: p.start + offset_in_piece,
                        length: p.length - offset_in_piece,
                    });
                }
            }

            current_offset = piece_end;
        }

        self.pieces = new_pieces;
        self.total_length = self.total_length.saturating_sub(actual_del);
    }

    /// Reverts to the previous piece-table state.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.pieces.clone());
            self.pieces = prev;
            self.recalc_total_length();
            self.cached_lines = None;
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    /// Re-applies the most recently undone edit.
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.pieces.clone());
            self.pieces = next;
            self.recalc_total_length();
            self.cached_lines = None;
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    fn recalc_total_length(&mut self) {
        self.total_length = self.pieces.iter().map(|p| p.length).sum();
    }

    /// Rebuilds or retrieves cached lines of text for fast display.
    pub fn lines(&mut self) -> &[String] {
        if self.cached_lines.is_none() {
            let text = self.get_text();
            let mut lines = Vec::new();
            for l in text.split('\n') {
                lines.push(l.to_string());
            }
            if lines.is_empty() {
                lines.push(String::new());
            }
            self.cached_lines = Some(lines);
        }
        self.cached_lines.as_ref().unwrap()
    }

    /// Line count of the document.
    pub fn line_count(&mut self) -> usize {
        self.lines().len()
    }

    /// Converts 0-indexed (line, col) to absolute byte position.
    pub fn line_col_to_pos(&mut self, target_line: usize, target_col: usize) -> usize {
        let lines = self.lines();
        let mut pos = 0;
        for (i, line) in lines.iter().enumerate() {
            if i == target_line {
                return pos + target_col.min(line.len());
            }
            pos += line.len() + 1; // +1 for '\n'
        }
        pos
    }

    /// Converts absolute byte position to 0-indexed (line, col).
    pub fn pos_to_line_col(&mut self, target_pos: usize) -> (usize, usize) {
        let lines = self.lines();
        let mut current_pos = 0;
        for (i, line) in lines.iter().enumerate() {
            let next_pos = current_pos + line.len() + 1;
            if target_pos < next_pos || i == lines.len() - 1 {
                let col = target_pos.saturating_sub(current_pos).min(line.len());
                return (i, col);
            }
            current_pos = next_pos;
        }
        (0, 0)
    }

    /// Marks the document as saved (not dirty).
    pub fn mark_saved(&mut self) {
        self.is_dirty = false;
    }
}
