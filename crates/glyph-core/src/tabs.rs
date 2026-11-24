//! tabs.rs - Multi-File Tab Management and Document System for Glyph.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[cfg(feature = "std")]
use std::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::buffer::PieceTable;
use crate::cursor::CursorManager;
use crate::folding::CodeFoldingManager;

/// Supported language tags in core.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LanguageKind {
    Rust,
    C,
    Python,
    Markdown,
    Shell,
    Json,
    PlainText,
}

impl LanguageKind {
    pub fn from_path(path: &str) -> Self {
        if path.ends_with(".rs") {
            Self::Rust
        } else if path.ends_with(".c") || path.ends_with(".h") || path.ends_with(".cpp") {
            Self::C
        } else if path.ends_with(".py") {
            Self::Python
        } else if path.ends_with(".md") || path.ends_with(".markdown") {
            Self::Markdown
        } else if path.ends_with(".sh") || path.ends_with(".bash") || path.ends_with(".novash") {
            Self::Shell
        } else if path.ends_with(".json") {
            Self::Json
        } else {
            Self::PlainText
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::C => "C/C++",
            Self::Python => "Python",
            Self::Markdown => "Markdown",
            Self::Shell => "Shell",
            Self::Json => "JSON",
            Self::PlainText => "Plain Text",
        }
    }
}

/// An open file tab containing its own piece-table buffer, cursors, and folding map.
#[derive(Clone, Debug)]
pub struct EditorDocument {
    pub path: String,
    pub title: String,
    pub language: LanguageKind,
    pub buffer: PieceTable,
    pub cursors: CursorManager,
    pub folding: CodeFoldingManager,
    pub scroll_line: usize,
    pub scroll_col: usize,
}

impl EditorDocument {
    pub fn new(path: &str, content: &str) -> Self {
        let title = path.split('/').last().unwrap_or(path).to_string();
        let language = LanguageKind::from_path(path);
        let mut buffer = PieceTable::new(content);

        let mut folding = CodeFoldingManager::new();
        folding.calculate_folds(buffer.lines());

        Self {
            path: path.to_string(),
            title,
            language,
            buffer,
            cursors: CursorManager::new(),
            folding,
            scroll_line: 0,
            scroll_col: 0,
        }
    }
}

/// Tab manager coordinating open editor documents.
#[derive(Clone, Debug)]
pub struct TabManager {
    pub tabs: Vec<EditorDocument>,
    pub active_idx: usize,
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TabManager {
    pub fn new() -> Self {
        let default_doc = EditorDocument::new(
            "src/main.rs",
            "//! NovaOS Sovereign Code Studio\nfn main() {\n    println!(\"Hello from Glyph Editor!\");\n}\n",
        );
        Self {
            tabs: vec![default_doc],
            active_idx: 0,
        }
    }

    pub fn active(&self) -> &EditorDocument {
        &self.tabs[self.active_idx]
    }

    pub fn active_mut(&mut self) -> &mut EditorDocument {
        &mut self.tabs[self.active_idx]
    }

    /// Opens a file in a new tab, or switches to it if already open.
    pub fn open_file(&mut self, path: &str, content: &str) {
        if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
            self.active_idx = idx;
        } else {
            let doc = EditorDocument::new(path, content);
            self.tabs.push(doc);
            self.active_idx = self.tabs.len() - 1;
        }
    }

    /// Closes tab at index `idx`. If closed tab was active, adjusts `active_idx`.
    pub fn close_tab(&mut self, idx: usize) -> bool {
        if self.tabs.len() <= 1 {
            // Cannot close the last remaining document tab
            return false;
        }
        if idx < self.tabs.len() {
            self.tabs.remove(idx);
            if self.active_idx >= self.tabs.len() {
                self.active_idx = self.tabs.len() - 1;
            }
            true
        } else {
            false
        }
    }

    /// Selects tab at index `idx`.
    pub fn select_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.active_idx = idx;
        }
    }

    /// Auto-saves all modified documents in the editor.
    pub fn auto_save_all(&mut self) -> usize {
        let mut count = 0;
        for tab in &mut self.tabs {
            if tab.buffer.is_dirty {
                tab.buffer.mark_saved();
                count += 1;
            }
        }
        count
    }
}
