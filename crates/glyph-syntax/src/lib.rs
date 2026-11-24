//! lib.rs - Multi-Language Syntax Highlighting Engine for Glyph.
//! Supports Rust, C/C++, Python, Markdown, Shell, JSON, and TOML.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

#[cfg(feature = "std")]
use std::{vec, vec::Vec};

pub use glyph_core::LanguageKind;

/// Token classification for syntax rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    Type,
    StringLiteral,
    NumberLiteral,
    Comment,
    Operator,
    Heading,
    Text,
}

impl TokenType {
    /// 32-bit ARGB / 24-bit RGB representation.
    pub const fn rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Keyword => (191, 90, 242),      // Accent Purple
            Self::Type => (90, 200, 250),         // Bright Cyan
            Self::StringLiteral => (48, 209, 88), // Emerald Green
            Self::NumberLiteral => (255, 159, 10),// Amber Orange
            Self::Comment => (142, 142, 147),     // Dim Muted Gray
            Self::Operator => (255, 55, 95),      // Coral Pink
            Self::Heading => (255, 214, 10),      // Golden Yellow
            Self::Text => (240, 240, 245),        // Crisp White
        }
    }
}

/// A highlighted range in a line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxSpan {
    pub start_col: usize,
    pub end_col: usize,
    pub token_type: TokenType,
}

pub struct SyntaxHighlighter;

impl SyntaxHighlighter {
    /// Tokenizes a single line into highlighted syntax spans.
    pub fn highlight_line(lang: LanguageKind, line: &str) -> Vec<SyntaxSpan> {
        match lang {
            LanguageKind::Rust => Self::highlight_rust(line),
            LanguageKind::C => Self::highlight_c(line),
            LanguageKind::Python => Self::highlight_python(line),
            LanguageKind::Markdown => Self::highlight_markdown(line),
            LanguageKind::Shell => Self::highlight_shell(line),
            LanguageKind::Json => Self::highlight_json(line),
            LanguageKind::PlainText => Self::highlight_plain(line),
        }
    }

    fn highlight_rust(line: &str) -> Vec<SyntaxSpan> {
        let mut spans = Vec::new();
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        if trimmed.starts_with("//") {
            spans.push(SyntaxSpan {
                start_col: indent,
                end_col: line.len(),
                token_type: TokenType::Comment,
            });
            return spans;
        }

        Self::tokenize_words(
            line,
            &[
                "fn", "let", "mut", "pub", "struct", "enum", "impl", "use", "mod",
                "match", "if", "else", "for", "while", "loop", "return", "true",
                "false", "const", "static", "type", "where", "async", "await",
                "trait", "self", "Self",
            ],
            &[
                "u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64",
                "isize", "f32", "f64", "bool", "char", "str", "String", "Vec",
                "Option", "Result", "Some", "None", "Ok", "Err",
            ],
            &mut spans,
        );
        spans
    }

    fn highlight_c(line: &str) -> Vec<SyntaxSpan> {
        let mut spans = Vec::new();
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            spans.push(SyntaxSpan {
                start_col: indent,
                end_col: line.len(),
                token_type: TokenType::Comment,
            });
            return spans;
        }

        Self::tokenize_words(
            line,
            &[
                "int", "char", "void", "float", "double", "struct", "union",
                "typedef", "enum", "return", "if", "else", "for", "while",
                "do", "switch", "case", "break", "continue", "static", "const",
                "extern", "sizeof",
            ],
            &["uint8_t", "uint16_t", "uint32_t", "uint64_t", "size_t", "ssize_t", "bool"],
            &mut spans,
        );
        spans
    }

    fn highlight_python(line: &str) -> Vec<SyntaxSpan> {
        let mut spans = Vec::new();
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        if trimmed.starts_with('#') {
            spans.push(SyntaxSpan {
                start_col: indent,
                end_col: line.len(),
                token_type: TokenType::Comment,
            });
            return spans;
        }

        Self::tokenize_words(
            line,
            &[
                "def", "class", "import", "from", "as", "return", "if", "elif",
                "else", "for", "while", "try", "except", "finally", "with", "lambda",
                "yield", "True", "False", "None", "and", "or", "not", "in", "is",
                "self",
            ],
            &["int", "str", "float", "bool", "list", "dict", "set", "tuple"],
            &mut spans,
        );
        spans
    }

    fn highlight_markdown(line: &str) -> Vec<SyntaxSpan> {
        let mut spans = Vec::new();
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        if trimmed.starts_with('#') {
            spans.push(SyntaxSpan {
                start_col: indent,
                end_col: line.len(),
                token_type: TokenType::Heading,
            });
        } else if trimmed.starts_with("> ") {
            spans.push(SyntaxSpan {
                start_col: indent,
                end_col: line.len(),
                token_type: TokenType::Comment,
            });
        } else {
            spans.push(SyntaxSpan {
                start_col: 0,
                end_col: line.len(),
                token_type: TokenType::Text,
            });
        }
        spans
    }

    fn highlight_shell(line: &str) -> Vec<SyntaxSpan> {
        let mut spans = Vec::new();
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        if trimmed.starts_with('#') {
            spans.push(SyntaxSpan {
                start_col: indent,
                end_col: line.len(),
                token_type: TokenType::Comment,
            });
            return spans;
        }

        Self::tokenize_words(
            line,
            &[
                "if", "then", "else", "elif", "fi", "case", "esac", "for", "while",
                "do", "done", "in", "function", "return", "exit", "echo", "cd",
                "export", "local",
            ],
            &[],
            &mut spans,
        );
        spans
    }

    fn highlight_json(line: &str) -> Vec<SyntaxSpan> {
        let mut spans = Vec::new();
        let mut in_string = false;
        let mut str_start = 0;

        for (i, ch) in line.char_indices() {
            if ch == '"' {
                if in_string {
                    spans.push(SyntaxSpan {
                        start_col: str_start,
                        end_col: i + 1,
                        token_type: TokenType::StringLiteral,
                    });
                    in_string = false;
                } else {
                    str_start = i;
                    in_string = true;
                }
            } else if !in_string && (ch.is_ascii_digit() || ch == '-') {
                spans.push(SyntaxSpan {
                    start_col: i,
                    end_col: i + 1,
                    token_type: TokenType::NumberLiteral,
                });
            }
        }
        if spans.is_empty() {
            spans.push(SyntaxSpan {
                start_col: 0,
                end_col: line.len(),
                token_type: TokenType::Text,
            });
        }
        spans
    }

    fn highlight_plain(line: &str) -> Vec<SyntaxSpan> {
        if line.is_empty() {
            Vec::new()
        } else {
            vec![SyntaxSpan {
                start_col: 0,
                end_col: line.len(),
                token_type: TokenType::Text,
            }]
        }
    }

    fn tokenize_words(
        line: &str,
        keywords: &[&str],
        types: &[&str],
        spans: &mut Vec<SyntaxSpan>,
    ) {
        let mut word_start: Option<usize> = None;

        for (i, ch) in line.char_indices() {
            let is_ident_char = ch.is_alphanumeric() || ch == '_';

            if is_ident_char && word_start.is_none() {
                word_start = Some(i);
            } else if !is_ident_char && word_start.is_some() {
                let start = word_start.unwrap();
                let word = &line[start..i];
                if keywords.contains(&word) {
                    spans.push(SyntaxSpan {
                        start_col: start,
                        end_col: i,
                        token_type: TokenType::Keyword,
                    });
                } else if types.contains(&word) {
                    spans.push(SyntaxSpan {
                        start_col: start,
                        end_col: i,
                        token_type: TokenType::Type,
                    });
                }
                word_start = None;
            }
        }

        if let Some(start) = word_start {
            let word = &line[start..];
            let end = line.len();
            if keywords.contains(&word) {
                spans.push(SyntaxSpan {
                    start_col: start,
                    end_col: end,
                    token_type: TokenType::Keyword,
                });
            } else if types.contains(&word) {
                spans.push(SyntaxSpan {
                    start_col: start,
                    end_col: end,
                    token_type: TokenType::Type,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_highlighting_all_languages() {
        assert!(!SyntaxHighlighter::highlight_line(LanguageKind::Rust, "fn test() -> bool {").is_empty());
        assert!(!SyntaxHighlighter::highlight_line(LanguageKind::C, "int main() {").is_empty());
        assert!(!SyntaxHighlighter::highlight_line(LanguageKind::Python, "def run():").is_empty());
        assert!(!SyntaxHighlighter::highlight_line(LanguageKind::Markdown, "# Header").is_empty());
        assert!(!SyntaxHighlighter::highlight_line(LanguageKind::Shell, "export PATH=$PATH").is_empty());
        assert!(!SyntaxHighlighter::highlight_line(LanguageKind::Json, "{\"key\": 123}").is_empty());
    }
}
