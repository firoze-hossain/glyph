//! folding.rs - Code Folding Engine for Glyph.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

/// A foldable range in line coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoldRange {
    pub start_line: usize,
    pub end_line: usize,
    pub is_folded: bool,
}

/// Code folding manager tracking scopes and collapsed blocks.
#[derive(Clone, Debug, Default)]
pub struct CodeFoldingManager {
    pub ranges: Vec<FoldRange>,
}

impl CodeFoldingManager {
    pub fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    /// Detects fold scopes based on indentation and brace blocks.
    pub fn calculate_folds(&mut self, lines: &[String]) {
        self.ranges.clear();
        let mut brace_stack = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            for ch in line.chars() {
                if ch == '{' {
                    brace_stack.push(line_idx);
                } else if ch == '}' {
                    if let Some(start_line) = brace_stack.pop() {
                        if line_idx > start_line {
                            self.ranges.push(FoldRange {
                                start_line,
                                end_line: line_idx,
                                is_folded: false,
                            });
                        }
                    }
                }
            }
        }
    }

    /// Toggles folded state of range starting at `line_idx`.
    pub fn toggle_fold(&mut self, line_idx: usize) -> bool {
        if let Some(r) = self.ranges.iter_mut().find(|r| r.start_line == line_idx) {
            r.is_folded = !r.is_folded;
            true
        } else {
            false
        }
    }

    /// Returns true if a given line is currently hidden by an enclosing fold.
    pub fn is_line_hidden(&self, line_idx: usize) -> bool {
        for r in &self.ranges {
            if r.is_folded && line_idx > r.start_line && line_idx <= r.end_line {
                return true;
            }
        }
        false
    }
}
