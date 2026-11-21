//! The showcase's application model: commands, documents, and a lazy file tree.

use std::path::{Path, PathBuf};

use egui_workbench::app::Document;
use egui_workbench::{RowIcon, ThemeId, TreeNode};

/// Every action the shell can request. Menus, the palette, the keyboard menu, and
/// shortcuts all produce these, and one `handle` function applies them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmd {
    New,
    Save,
    CloseTab,
    CloseOthers,
    NextTab,
    PreviousTab,
    Rename,
    Delete,
    ToggleSidebar,
    SetMode(usize),
    Palette,
    QuickOpen,
    Search,
    Settings,
    SetTheme(ThemeId),
    ToggleFollowSystem,
    About,
    Quit,
}

/// One open document.
pub struct Doc {
    pub path: Option<PathBuf>,
    pub name: String,
    pub text: String,
    pub dirty: bool,
}

impl Document for Doc {
    fn title(&self) -> &str {
        &self.name
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl Doc {
    pub fn untitled(index: usize) -> Self {
        Self {
            path: None,
            name: format!("Untitled-{index}"),
            text: String::new(),
            dirty: false,
        }
    }

    pub fn open(path: &Path) -> Self {
        let text = std::fs::read_to_string(path).unwrap_or_else(|error| format!("Cannot read this file: {error}"));
        Self {
            path: Some(path.to_path_buf()),
            name: file_name(path),
            text,
            dirty: false,
        }
    }
}

/// A directory entry whose children load on first expansion.
pub struct FsNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub expanded: bool,
    pub children: Option<Vec<FsNode>>,
}

impl FsNode {
    pub fn root(path: &Path) -> Self {
        let mut root = Self {
            path: path.to_path_buf(),
            name: file_name(path),
            is_dir: true,
            expanded: true,
            children: None,
        };
        root.load();
        root
    }

    fn load(&mut self) {
        let Ok(entries) = std::fs::read_dir(&self.path) else {
            self.children = Some(Vec::new());
            return;
        };
        let mut children: Vec<Self> = entries
            .flatten()
            .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
            .map(|entry| {
                let path = entry.path();
                Self {
                    name: file_name(&path),
                    is_dir: path.is_dir(),
                    path,
                    expanded: false,
                    children: None,
                }
            })
            .collect();
        // Folders first, then case-insensitive names, as in VS Code.
        children.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        self.children = Some(children);
    }

    /// Expand or collapse the node at `path`, loading its children the first time.
    pub fn toggle(&mut self, path: &Path) -> bool {
        if self.path == path {
            self.expanded = !self.expanded;
            if self.expanded && self.children.is_none() {
                self.load();
            }
            return true;
        }
        self.children
            .iter_mut()
            .flatten()
            .any(|child| path.starts_with(&child.path) && child.toggle(path))
    }

    pub fn collapse_all(&mut self) {
        for child in self.children.iter_mut().flatten() {
            child.expanded = false;
            child.collapse_all();
        }
    }

    pub fn refresh(&mut self) {
        self.load();
    }

    /// The workbench tree nodes for this node's children.
    pub fn view(&self) -> Vec<TreeNode<PathBuf>> {
        self.children.iter().flatten().map(Self::node).collect()
    }

    fn node(&self) -> TreeNode<PathBuf> {
        if self.is_dir {
            let children = self
                .children
                .as_ref()
                .map(|children| children.iter().map(Self::node).collect());
            TreeNode {
                key: self.path.clone(),
                label: self.name.clone(),
                icon: RowIcon::MaterialFolder(self.name.clone()),
                children: Some(children.unwrap_or_default()),
                expanded: self.expanded,
                tooltip: None,
            }
        } else {
            TreeNode::leaf(
                self.path.clone(),
                self.name.clone(),
                RowIcon::MaterialFile(self.path.clone()),
            )
        }
    }

    /// Every file already loaded under this node, for Quick Open.
    pub fn files(&self, out: &mut Vec<PathBuf>) {
        for child in self.children.iter().flatten() {
            if child.is_dir {
                child.files(out);
            } else {
                out.push(child.path.clone());
            }
        }
    }
}

pub fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}
