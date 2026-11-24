//! project.rs - Project Directory Sidebar and File Tree Explorer for Glyph.

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

/// Type of node in the project explorer tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectNodeKind {
    Directory,
    File,
}

/// A file or folder item in the project sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectNode {
    pub path: String,
    pub name: String,
    pub kind: ProjectNodeKind,
    pub depth: usize,
    pub is_expanded: bool,
    pub children: Vec<ProjectNode>,
}

impl ProjectNode {
    pub fn icon(&self) -> &'static str {
        if self.kind == ProjectNodeKind::Directory {
            if self.is_expanded { "v" } else { ">" }
        } else if self.name.ends_with(".rs") {
            "[RS]"
        } else if self.name.ends_with(".c") || self.name.ends_with(".h") {
            "[C]"
        } else if self.name.ends_with(".py") {
            "[PY]"
        } else if self.name.ends_with(".md") {
            "[MD]"
        } else if self.name.ends_with(".json") {
            "[JSON]"
        } else if self.name.ends_with(".sh") {
            "[SH]"
        } else {
            "[-]"
        }
    }
}

/// Project directory tree explorer model.
#[derive(Clone, Debug)]
pub struct ProjectExplorer {
    pub root_name: String,
    pub root_nodes: Vec<ProjectNode>,
}

impl Default for ProjectExplorer {
    fn default() -> Self {
        Self::new("novaos-core")
    }
}

impl ProjectExplorer {
    pub fn new(root_name: &str) -> Self {
        let mut explorer = Self {
            root_name: root_name.to_string(),
            root_nodes: Vec::new(),
        };
        explorer.populate_demo_tree();
        explorer
    }

    /// Populates sample directory tree for demonstration and tests.
    fn populate_demo_tree(&mut self) {
        let src_folder = ProjectNode {
            path: "src".into(),
            name: "src".into(),
            kind: ProjectNodeKind::Directory,
            depth: 0,
            is_expanded: true,
            children: vec![
                ProjectNode {
                    path: "src/main.rs".into(),
                    name: "main.rs".into(),
                    kind: ProjectNodeKind::File,
                    depth: 1,
                    is_expanded: false,
                    children: Vec::new(),
                },
                ProjectNode {
                    path: "src/buffer.rs".into(),
                    name: "buffer.rs".into(),
                    kind: ProjectNodeKind::File,
                    depth: 1,
                    is_expanded: false,
                    children: Vec::new(),
                },
                ProjectNode {
                    path: "src/syntax.rs".into(),
                    name: "syntax.rs".into(),
                    kind: ProjectNodeKind::File,
                    depth: 1,
                    is_expanded: false,
                    children: Vec::new(),
                },
            ],
        };

        let docs_folder = ProjectNode {
            path: "docs".into(),
            name: "docs".into(),
            kind: ProjectNodeKind::Directory,
            depth: 0,
            is_expanded: false,
            children: vec![ProjectNode {
                path: "docs/README.md".into(),
                name: "README.md".into(),
                kind: ProjectNodeKind::File,
                depth: 1,
                is_expanded: false,
                children: Vec::new(),
            }],
        };

        let cargo_toml = ProjectNode {
            path: "Cargo.toml".into(),
            name: "Cargo.toml".into(),
            kind: ProjectNodeKind::File,
            depth: 0,
            is_expanded: false,
            children: Vec::new(),
        };

        self.root_nodes = vec![src_folder, docs_folder, cargo_toml];
    }

    /// Returns a flat list of all currently visible (expanded) nodes for rendering.
    pub fn visible_nodes(&self) -> Vec<ProjectNode> {
        let mut out = Vec::new();
        for node in &self.root_nodes {
            Self::flatten_node(node, &mut out);
        }
        out
    }

    fn flatten_node(node: &ProjectNode, out: &mut Vec<ProjectNode>) {
        out.push(node.clone());
        if node.kind == ProjectNodeKind::Directory && node.is_expanded {
            for child in &node.children {
                Self::flatten_node(child, out);
            }
        }
    }

    /// Toggles folder expansion at `target_path`.
    pub fn toggle_path(&mut self, target_path: &str) -> bool {
        for node in &mut self.root_nodes {
            if Self::toggle_node_recursive(node, target_path) {
                return true;
            }
        }
        false
    }

    fn toggle_node_recursive(node: &mut ProjectNode, target_path: &str) -> bool {
        if node.path == target_path {
            if node.kind == ProjectNodeKind::Directory {
                node.is_expanded = !node.is_expanded;
                return true;
            }
        }
        for child in &mut node.children {
            if Self::toggle_node_recursive(child, target_path) {
                return true;
            }
        }
        false
    }
}
