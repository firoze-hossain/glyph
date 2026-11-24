//! lib.rs - Core domain logic for Glyph Code & Text Editor.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod bracket;
pub mod buffer;
pub mod cursor;
pub mod folding;
pub mod project;
pub mod search;
pub mod tabs;

pub use bracket::BracketMatcher;
pub use buffer::{BufferKind, Piece, PieceTable};
pub use cursor::{Cursor, CursorManager};
pub use folding::{CodeFoldingManager, FoldRange};
pub use project::{ProjectExplorer, ProjectNode, ProjectNodeKind};
pub use search::{FindReplaceEngine, SearchMatch};
pub use tabs::{EditorDocument, LanguageKind, TabManager};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_table_basic() {
        let mut table = PieceTable::new("Hello World");
        assert_eq!(table.get_text(), "Hello World");
        assert_eq!(table.len(), 11);

        table.insert(5, ", Beautiful");
        assert_eq!(table.get_text(), "Hello, Beautiful World");

        table.delete(5, 11);
        assert_eq!(table.get_text(), "Hello World");
    }

    #[test]
    fn test_piece_table_undo_redo() {
        let mut table = PieceTable::new("Initial");
        table.insert(7, " Text");
        assert_eq!(table.get_text(), "Initial Text");
        assert!(table.undo());
        assert_eq!(table.get_text(), "Initial");
        assert!(table.redo());
        assert_eq!(table.get_text(), "Initial Text");
    }

    #[test]
    fn test_multi_cursor_normalize() {
        let mut cm = CursorManager::new();
        cm.add_cursor(2, 4);
        cm.add_cursor(1, 10);
        cm.add_cursor(2, 4); // Duplicate
        assert_eq!(cm.cursors.len(), 3); // (0,0), (1,10), (2,4)
    }

    #[test]
    fn test_bracket_matching_nested() {
        let code = "{ a: [1, (2 + 3)] }";
        // '[' at index 5 matches ']' at 16
        assert_eq!(BracketMatcher::find_matching_bracket(code, 5), Some(16));
        // '(' at index 9 matches ')' at 15
        assert_eq!(BracketMatcher::find_matching_bracket(code, 9), Some(15));
    }

    #[test]
    fn test_code_folding() {
        let lines = vec![
            "fn compute() {".to_string(),
            "    let a = 1;".to_string(),
            "}".to_string(),
        ];
        let mut folding = CodeFoldingManager::new();
        folding.calculate_folds(&lines);
        assert_eq!(folding.ranges.len(), 1);
        assert_eq!(folding.ranges[0].start_line, 0);
        assert_eq!(folding.ranges[0].end_line, 2);
    }

    #[test]
    fn test_find_and_replace() {
        let mut table = PieceTable::new("foo bar foo baz");
        let count = FindReplaceEngine::replace_all(&mut table, "foo", "qux", true);
        assert_eq!(count, 2);
        assert_eq!(table.get_text(), "qux bar qux baz");
    }

    #[test]
    fn test_project_explorer_visible() {
        let project = ProjectExplorer::default();
        let visible = project.visible_nodes();
        assert!(visible.iter().any(|n| n.name == "src"));
        assert!(visible.iter().any(|n| n.name == "main.rs"));
    }

    #[test]
    fn test_tab_manager_auto_save() {
        let mut tabs = TabManager::new();
        tabs.open_file("test.txt", "content");
        assert_eq!(tabs.tabs.len(), 2);
        tabs.active_mut().buffer.insert(0, "new ");
        assert!(tabs.active().buffer.is_dirty);
        assert_eq!(tabs.auto_save_all(), 1);
        assert!(!tabs.active().buffer.is_dirty);
    }
}
