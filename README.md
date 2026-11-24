# Glyph Code Studio ✍️

**Glyph** (Γλυφή) is the sovereign, lightning-fast Code & Text Editor for **NovaOS** and cross-platform desktop environments. Designed for pure developer ergonomics, distraction-free writing, and instantaneous responsiveness, Glyph couples high-performance data structures with elegant typography.

---

## 🌟 Key Features

### 1. ⚡ Piece-Table Text Buffer Engine
- **O(1) In-Memory Edits**: Uses separate original and append buffers linked through slice descriptors, avoiding large memory copies during intensive edits.
- **Bi-Directional Undo & Redo**: Full transaction history tracking with instantaneous rollback and replay.
- **Line Indexing**: Sub-millisecond mapping between byte offsets and 0-indexed line/column coordinates.

### 2. 🎯 Multi-Cursor Synchronized Editing
- **Concurrent Carets**: Primary caret plus arbitrary secondary cursors for simultaneous multi-point refactoring.
- **Deduplication & Sorting**: Normalizes cursors by sorting and deduping overlapping positions automatically.

### 3. 🔍 Bracket Matching & Delimiter Auto-Closing
- **Nested Scanners**: Forward and backward matching across parentheses `()`, square brackets `[]`, and curly braces `{}`.
- **Smart Quotes**: Pairs single and double quotes with automatic cursor stepping.

### 4. 🗂️ Code Folding & Scope Discovery
- **Indentation & Brace Analysis**: Detects function, class, and block scopes.
- **Interactive Gutter Toggles**: Seamlessly collapse `[-]` and expand `[+]` code blocks.

### 5. 🎨 Multi-Language Syntax Highlighting
- **Polyglot Lexer**: Tokenization across **Rust**, **C/C++**, **Python**, **Markdown**, **Shell**, **JSON**, and **TOML**.
- **Visual Classification**: 8 token classes (Keywords, Types, Strings, Numbers, Comments, Operators, Headings, Text) rendered with vibrant 32-bit colors.

### 6. 🔎 Reverse-Coordinate Find & Replace
- **Bulk Replacement**: Executes reverse-offset substitutions to prevent coordinate drift.
- **Match Navigation**: Seamlessly step through query hits forward and backward.

### 7. 📁 Tabbed Document & Project Explorer
- **Multi-Tab Sessions**: Multiple simultaneous open documents with dirty state (`*`) tracking.
- **Project Directory Tree**: Interactive folder expansion and file switching with icon badges (`[RS]`, `[C]`, `[PY]`, `[MD]`, `[JSON]`, `[SH]`).

### 8. 🖼️ Pure 2D Software Buffer Rasterizer
- Zero GPU or C dynamic runtime requirements.
- High-contrast line numbers gutter, active line accentuation, acrylic header, and status bar telemetry.

---

## 🏗️ Workspace Architecture

```
glyph/
├── Cargo.toml
├── README.md
├── .gitignore
├── crates/
│   ├── glyph-core/        # Piece-Table buffer, multi-cursor, bracket matching, folding, tabs, search
│   │   └── src/lib.rs
│   ├── glyph-syntax/      # Polyglot syntax tokenizers (Rust, C, Python, Markdown, Shell, JSON)
│   │   └── src/lib.rs
│   ├── glyph-ui/          # Pure 2D software buffer renderer (gutter, acrylic toolbar, tabs, status bar)
│   │   └── src/lib.rs
│   ├── glyph-platform/    # Cross-platform filesystem & VFS abstractions (Linux, macOS, Windows, NovaOS)
│   │   └── src/lib.rs
│   └── glyph-app/         # Desktop CLI runner and interactive viewer (--selftest, --version, --info)
│       └── src/main.rs
└── novaos/                # NovaOS bare-metal ring-3 port
    ├── ffi.rs             # Ring-3 system call bindings
    ├── glyph.rs           # Standalone no_std / no_main ELF32 executable
    └── build.sh           # Automated build script creating GLYPH.ELF, EDIT.ELF, and PKG fixtures
```

---

## 🚀 Building & Running

### 1. Cross-Platform Desktop (Linux, macOS, Windows)

```bash
# Run the autonomous qualification suite
cargo run -p glyph-app -- --selftest

# Inspect editor architecture telemetry
cargo run -p glyph-app -- --info

# Open a file for editing
cargo run -p glyph-app -- src/main.rs
```

### 2. Autonomous Unit & Integration Tests

```bash
cargo test
```
*Executes all unit tests across Piece-Table buffer manipulation, multi-cursor synchronization, bracket matching, folding scopes, syntax tokenization, and 2D canvas rasterization.*

### 3. NovaOS Bare-Metal Ring-3 Port

```bash
# Build i686-novaos ELF32 binaries and package fixtures
./novaos/build.sh
```
*Builds `glyph.elf`, copies `GLYPH.ELF` and `EDIT.ELF`, and creates `GLYPH.PKG` and `EDITOR.PKG` metadata files inside `$NOVAOS_ROOT/tools/fixtures/`.*
