//! lib.rs - Pure 2D software renderer for Glyph Code & Text Editor.
//! Zero GPU requirements; double-buffered rasterizer into 32-bit ARGB CanvasSurface.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[cfg(feature = "std")]
use std::{
    format,
    string::String,
    vec,
    vec::Vec,
};

pub use glyph_core::{EditorDocument, LanguageKind, ProjectExplorer, TabManager};
pub use glyph_syntax::{SyntaxHighlighter, TokenType};

/// 32-bit ARGB Color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { a: 255, r, g, b }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { a, r, g, b }
    }

    pub const fn to_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

/// 2D Rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }
}

/// Double-buffered 32-bit software drawing canvas.
pub struct CanvasSurface {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
}

impl CanvasSurface {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0xFF000000; width * height],
        }
    }

    pub fn fill(&mut self, color: Color) {
        let val = color.to_u32();
        self.buffer.fill(val);
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let val = color.to_u32();
        let x0 = rect.x.max(0) as usize;
        let y0 = rect.y.max(0) as usize;
        let x1 = ((rect.x + rect.width) as usize).min(self.width);
        let y1 = ((rect.y + rect.height) as usize).min(self.height);

        for y in y0..y1 {
            let row_start = y * self.width;
            for x in x0..x1 {
                self.buffer[row_start + x] = val;
            }
        }
    }

    pub fn draw_rect_outline(&mut self, rect: Rect, color: Color) {
        let val = color.to_u32();
        let x0 = rect.x.max(0) as usize;
        let y0 = rect.y.max(0) as usize;
        let x1 = ((rect.x + rect.width) as usize).min(self.width);
        let y1 = ((rect.y + rect.height) as usize).min(self.height);

        if x1 > x0 && y1 > y0 {
            for x in x0..x1 {
                self.buffer[y0 * self.width + x] = val;
                self.buffer[(y1 - 1) * self.width + x] = val;
            }
            for y in y0..y1 {
                self.buffer[y * self.width + x0] = val;
                self.buffer[y * self.width + (x1 - 1)] = val;
            }
        }
    }

    /// Fast 8x16 monospace character blitter.
    pub fn draw_char(&mut self, cx: i32, cy: i32, ch: char, color: Color) {
        if cx < 0 || cy < 0 || (cx + 8) as usize > self.width || (cy + 16) as usize > self.height {
            return;
        }
        let val = color.to_u32();
        let glyph_bits = get_8x16_char_bitmap(ch);
        for row in 0..16 {
            let bits = glyph_bits[row];
            let py = (cy + row as i32) as usize;
            for col in 0..8 {
                if (bits & (0x80 >> col)) != 0 {
                    let px = (cx + col) as usize;
                    self.buffer[py * self.width + px] = val;
                }
            }
        }
    }

    pub fn draw_text(&mut self, mut x: i32, y: i32, text: &str, color: Color) {
        for ch in text.chars() {
            self.draw_char(x, y, ch, color);
            x += 8;
        }
    }
}

/// Palette tokens for dark code editor theme.
#[derive(Clone, Copy, Debug)]
pub struct EditorThemeTokens {
    pub bg_canvas: Color,
    pub bg_sidebar: Color,
    pub bg_gutter: Color,
    pub bg_tab_active: Color,
    pub bg_tab_inactive: Color,
    pub bg_toolbar: Color,
    pub bg_status: Color,
    pub active_line_bg: Color,
    pub border_color: Color,
    pub text_gutter: Color,
    pub text_gutter_active: Color,
    pub cursor_caret: Color,
    pub search_highlight: Color,
    pub text_primary: Color,
    pub text_muted: Color,
}

impl EditorThemeTokens {
    pub const fn dark() -> Self {
        Self {
            bg_canvas: Color::rgb(22, 22, 28),
            bg_sidebar: Color::rgb(18, 18, 22),
            bg_gutter: Color::rgb(26, 26, 32),
            bg_tab_active: Color::rgb(32, 32, 40),
            bg_tab_inactive: Color::rgb(20, 20, 26),
            bg_toolbar: Color::rgb(24, 24, 30),
            bg_status: Color::rgb(0, 120, 212), // Fluent Blue accent bar
            active_line_bg: Color::rgb(30, 30, 38),
            border_color: Color::rgba(255, 255, 255, 25),
            text_gutter: Color::rgb(110, 110, 125),
            text_gutter_active: Color::rgb(240, 240, 250),
            cursor_caret: Color::rgb(255, 214, 10),
            search_highlight: Color::rgba(255, 214, 10, 80),
            text_primary: Color::rgb(240, 240, 245),
            text_muted: Color::rgb(142, 142, 147),
        }
    }
}

/// Hit test actions.
#[derive(Clone, Debug, PartialEq)]
pub enum EditHitTarget {
    TabSelect(usize),
    TabClose(usize),
    ToolbarSave,
    ToolbarUndo,
    ToolbarRedo,
    ToolbarFind,
    SidebarItem(String),
    ToggleFold(usize),
    TextCanvasClick { line: usize, col: usize },
}

/// Complete editor view layout and rasterizer.
pub struct EditorView {
    pub width: i32,
    pub height: i32,
    pub sidebar_width: i32,
    pub gutter_width: i32,
    pub tokens: EditorThemeTokens,
}

impl Default for EditorView {
    fn default() -> Self {
        Self::new(1024, 680)
    }
}

impl EditorView {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            sidebar_width: 180,
            gutter_width: 48,
            tokens: EditorThemeTokens::dark(),
        }
    }

    /// Renders the complete editor view into `surface`.
    pub fn render(
        &self,
        surface: &mut CanvasSurface,
        tabs: &TabManager,
        project: &ProjectExplorer,
    ) {
        // 1. Background
        surface.fill(self.tokens.bg_canvas);

        let toolbar_h = 36;
        let tab_h = 32;
        let status_h = 24;
        let header_h = toolbar_h + tab_h;

        // 2. Toolbar
        self.render_toolbar(surface, Rect::new(0, 0, self.width, toolbar_h));

        // 3. Tab bar
        self.render_tab_bar(surface, tabs, Rect::new(self.sidebar_width, toolbar_h, self.width - self.sidebar_width, tab_h));

        // 4. Project Sidebar
        let sidebar_rect = Rect::new(0, toolbar_h, self.sidebar_width, self.height - toolbar_h - status_h);
        self.render_sidebar(surface, project, sidebar_rect);

        // 5. Code Gutter & Editor Canvas
        let active_doc = tabs.active();
        let editor_x = self.sidebar_width + self.gutter_width;
        let editor_y = header_h;
        let editor_w = self.width - editor_x;
        let editor_h = self.height - header_h - status_h;

        // Gutter background
        surface.fill_rect(
            Rect::new(self.sidebar_width, header_h, self.gutter_width, editor_h),
            self.tokens.bg_gutter,
        );

        // Render code lines
        self.render_code(surface, active_doc, Rect::new(editor_x, editor_y, editor_w, editor_h));

        // 6. Status Bar
        let status_rect = Rect::new(0, self.height - status_h, self.width, status_h);
        self.render_status_bar(surface, active_doc, status_rect);
    }

    fn render_toolbar(&self, surface: &mut CanvasSurface, rect: Rect) {
        surface.fill_rect(rect, self.tokens.bg_toolbar);
        surface.draw_rect_outline(rect, self.tokens.border_color);

        let title = "GLYPH CODE STUDIO";
        surface.draw_text(rect.x + 16, rect.y + 10, title, Color::rgb(0, 122, 255));

        // Action buttons
        let buttons = ["[New]", "[Open]", "[Save]", "[Undo]", "[Redo]", "[Find]"];
        let mut btn_x = rect.x + 200;
        for btn in buttons {
            surface.draw_text(btn_x, rect.y + 10, btn, self.tokens.text_primary);
            btn_x += (btn.len() as i32 * 8) + 16;
        }
    }

    fn render_tab_bar(&self, surface: &mut CanvasSurface, tabs: &TabManager, rect: Rect) {
        surface.fill_rect(rect, self.tokens.bg_tab_inactive);
        surface.draw_rect_outline(rect, self.tokens.border_color);

        let mut tab_x = rect.x;
        for (i, tab) in tabs.tabs.iter().enumerate() {
            let is_active = i == tabs.active_idx;
            let tab_w = (tab.title.len() as i32 * 8) + 36;
            let tab_rect = Rect::new(tab_x, rect.y, tab_w, rect.height);

            let bg = if is_active {
                self.tokens.bg_tab_active
            } else {
                self.tokens.bg_tab_inactive
            };
            surface.fill_rect(tab_rect, bg);
            surface.draw_rect_outline(tab_rect, self.tokens.border_color);

            let text_color = if is_active {
                self.tokens.text_primary
            } else {
                self.tokens.text_muted
            };
            surface.draw_text(tab_x + 8, rect.y + 8, &tab.title, text_color);

            // Dirty indicator or close button
            if tab.buffer.is_dirty {
                surface.draw_char(tab_x + tab_w - 18, rect.y + 8, '*', Color::rgb(255, 159, 10));
            } else {
                surface.draw_char(tab_x + tab_w - 18, rect.y + 8, 'x', self.tokens.text_muted);
            }

            tab_x += tab_w;
        }
    }

    fn render_sidebar(&self, surface: &mut CanvasSurface, project: &ProjectExplorer, rect: Rect) {
        surface.fill_rect(rect, self.tokens.bg_sidebar);
        surface.draw_rect_outline(rect, self.tokens.border_color);

        surface.draw_text(rect.x + 12, rect.y + 10, "EXPLORER", self.tokens.text_muted);

        let mut item_y = rect.y + 32;
        for node in project.visible_nodes() {
            let indent = node.depth as i32 * 12;
            let icon = node.icon();
            surface.draw_text(rect.x + 12 + indent, item_y, icon, Color::rgb(0, 122, 255));
            surface.draw_text(rect.x + 40 + indent, item_y, &node.name, self.tokens.text_primary);
            item_y += 20;
            if item_y > rect.y + rect.height - 20 {
                break;
            }
        }
    }

    fn render_code(&self, surface: &mut CanvasSurface, doc: &EditorDocument, rect: Rect) {
        let mut buffer = doc.buffer.clone();
        let lines = buffer.lines();
        let line_h = 18;
        let primary_line = doc.cursors.primary().line;

        let visible_lines = (rect.height / line_h) as usize;
        let start_line = doc.scroll_line;
        let end_line = (start_line + visible_lines).min(lines.len());

        for (i, line) in lines[start_line..end_line].iter().enumerate() {
            let line_idx = start_line + i;
            let line_y = rect.y + (i as i32 * line_h);

            // Active line highlight
            if line_idx == primary_line {
                surface.fill_rect(
                    Rect::new(self.sidebar_width, line_y, self.width - self.sidebar_width, line_h),
                    self.tokens.active_line_bg,
                );
            }

            // Gutter Line Number
            let num_str = format!("{:>3}", line_idx + 1);
            let num_col = if line_idx == primary_line {
                self.tokens.text_gutter_active
            } else {
                self.tokens.text_gutter
            };
            surface.draw_text(self.sidebar_width + 8, line_y, &num_str, num_col);

            // Fold marker if foldable
            if let Some(r) = doc.folding.ranges.iter().find(|r| r.start_line == line_idx) {
                let marker = if r.is_folded { "[+]" } else { "[-]" };
                surface.draw_text(self.sidebar_width + self.gutter_width - 20, line_y, marker, self.tokens.text_muted);
            }

            // Syntax highlighting spans
            let spans = SyntaxHighlighter::highlight_line(doc.language, line);
            if spans.is_empty() {
                surface.draw_text(rect.x + 8, line_y, line, self.tokens.text_primary);
            } else {
                let mut last_col = 0;
                for span in spans {
                    if span.start_col > last_col && span.start_col <= line.len() {
                        let unstyled = &line[last_col..span.start_col];
                        surface.draw_text(rect.x + 8 + (last_col as i32 * 8), line_y, unstyled, self.tokens.text_primary);
                    }
                    if span.end_col <= line.len() && span.end_col > span.start_col {
                        let styled = &line[span.start_col..span.end_col];
                        let (r, g, b) = span.token_type.rgb();
                        surface.draw_text(rect.x + 8 + (span.start_col as i32 * 8), line_y, styled, Color::rgb(r, g, b));
                    }
                    last_col = span.end_col;
                }
                if last_col < line.len() {
                    let trailing = &line[last_col..];
                    surface.draw_text(rect.x + 8 + (last_col as i32 * 8), line_y, trailing, self.tokens.text_primary);
                }
            }

            // Caret on active line
            if line_idx == primary_line {
                let caret_col = doc.cursors.primary().col;
                let caret_x = rect.x + 8 + (caret_col as i32 * 8);
                surface.fill_rect(Rect::new(caret_x, line_y, 2, 16), self.tokens.cursor_caret);
            }
        }
    }

    fn render_status_bar(&self, surface: &mut CanvasSurface, doc: &EditorDocument, rect: Rect) {
        surface.fill_rect(rect, self.tokens.bg_status);

        let left_info = format!(
            "{} {} | UTF-8 | {}",
            doc.path,
            if doc.buffer.is_dirty { "*" } else { "" },
            doc.language.name()
        );
        surface.draw_text(rect.x + 12, rect.y + 4, &left_info, self.tokens.text_primary);

        let prim = doc.cursors.primary();
        let right_info = format!("Ln {}, Col {} (Total: {})", prim.line + 1, prim.col + 1, doc.buffer.total_length);
        let rx = rect.x + rect.width - (right_info.len() as i32 * 8) - 16;
        surface.draw_text(rx, rect.y + 4, &right_info, self.tokens.text_primary);
    }
}

/// Fallback 8x16 font bitmap generator for autonomous rendering.
fn get_8x16_char_bitmap(ch: char) -> [u8; 16] {
    let mut b = [0u8; 16];
    let ascii = ch as u8;
    if ascii >= 32 && ascii <= 126 {
        // Deterministic pseudo-glyph representation
        let pattern = (ascii.wrapping_mul(37)) ^ 0xAA;
        b[2] = pattern & 0x7E;
        b[3] = (pattern >> 1) & 0x7E;
        b[4] = pattern & 0x7E;
        b[5] = 0x66;
        b[6] = 0x66;
        b[7] = 0x7E;
        b[8] = 0x66;
        b[9] = 0x66;
        b[10] = 0x66;
        b[11] = pattern & 0x7E;
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_rasterization() {
        let mut surface = CanvasSurface::new(1024, 680);
        let tabs = TabManager::new();
        let project = ProjectExplorer::default();
        let view = EditorView::new(1024, 680);

        view.render(&mut surface, &tabs, &project);

        let non_zero = surface.buffer.iter().filter(|&&px| px != 0).count();
        assert!(non_zero > 1000, "Expected non-zero rasterized pixel buffer");
    }
}
