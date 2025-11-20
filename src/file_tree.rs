use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(name: String, path: PathBuf, is_dir: bool, depth: usize) -> Self {
        Self {
            name,
            path,
            is_dir,
            depth,
            expanded: depth == 0, // Root is expanded by default
            children: Vec::new(),
        }
    }

    /// Get all visible items in the tree (for rendering)
    pub fn visible_items(&self) -> Vec<&TreeNode> {
        let mut items = Vec::new();
        self.collect_visible(&mut items);
        items
    }

    fn collect_visible<'a>(&'a self, items: &mut Vec<&'a TreeNode>) {
        items.push(self);
        if self.expanded {
            for child in &self.children {
                child.collect_visible(items);
            }
        }
    }

    /// Toggle expanded state for directories
    pub fn toggle_expanded(&mut self) {
        if self.is_dir {
            self.expanded = !self.expanded;
        }
    }

    /// Find a node by index in the visible items and return mutable reference
    pub fn find_by_index_mut(&mut self, target_idx: usize) -> Option<&mut TreeNode> {
        let mut current_idx = 0;
        self.find_by_index_recursive(&mut current_idx, target_idx)
    }

    fn find_by_index_recursive(
        &mut self,
        current_idx: &mut usize,
        target_idx: usize,
    ) -> Option<&mut TreeNode> {
        if *current_idx == target_idx {
            return Some(self);
        }
        *current_idx += 1;

        if self.expanded {
            for child in &mut self.children {
                if let Some(node) = child.find_by_index_recursive(current_idx, target_idx) {
                    return Some(node);
                }
            }
        }
        None
    }

    /// Find parent index of a node at given index
    pub fn find_parent_index(&self, target_idx: usize) -> Option<usize> {
        let mut current_idx = 0;
        self.find_parent_recursive(&mut current_idx, target_idx, None)
    }

    fn find_parent_recursive(
        &self,
        current_idx: &mut usize,
        target_idx: usize,
        parent_idx: Option<usize>,
    ) -> Option<usize> {
        let my_idx = *current_idx;

        if my_idx == target_idx {
            return parent_idx;
        }
        *current_idx += 1;

        if self.expanded {
            for child in &self.children {
                if let Some(idx) = child.find_parent_recursive(current_idx, target_idx, Some(my_idx)) {
                    return Some(idx);
                }
            }
        }
        None
    }
}

/// Build a file tree from a directory, only including markdown files
pub fn build_tree(root_path: &Path) -> TreeNode {
    let root_name = root_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| root_path.to_string_lossy().to_string());

    let mut root = TreeNode::new(root_name, root_path.to_path_buf(), true, 0);

    // Collect all markdown files and directories
    let mut entries: Vec<_> = WalkDir::new(root_path)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden directories
            !e.file_name()
                .to_str()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
        })
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_dir()
                || e.path()
                    .extension()
                    .map(|ext| ext == "md" || ext == "markdown")
                    .unwrap_or(false)
        })
        .collect();

    // Sort entries by path for consistent ordering
    entries.sort_by(|a, b| a.path().cmp(b.path()));

    // Build nested structure
    for entry in entries {
        let rel_path = entry.path().strip_prefix(root_path).unwrap();
        let components: Vec<_> = rel_path.components().collect();

        insert_path(
            &mut root,
            entry.path(),
            &components,
            0,
            entry.file_type().is_dir(),
        );
    }

    // Remove empty directories
    prune_empty_dirs(&mut root);

    root
}

fn insert_path(
    node: &mut TreeNode,
    full_path: &Path,
    components: &[std::path::Component],
    depth: usize,
    is_dir: bool,
) {
    if components.is_empty() {
        return;
    }

    let name = components[0].as_os_str().to_string_lossy().to_string();

    // Find or create child
    let child_idx = node.children.iter().position(|c| c.name == name);

    if components.len() == 1 {
        // This is the target node
        if child_idx.is_none() {
            let child = TreeNode::new(name, full_path.to_path_buf(), is_dir, depth + 1);
            node.children.push(child);
            // Sort children: directories first, then alphabetically
            node.children.sort_by(|a, b| {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            });
        }
    } else {
        // Need to traverse deeper
        let child = if let Some(idx) = child_idx {
            &mut node.children[idx]
        } else {
            // Create intermediate directory node
            let parent_path = full_path
                .ancestors()
                .nth(components.len() - 1)
                .unwrap();
            let child = TreeNode::new(name, parent_path.to_path_buf(), true, depth + 1);
            node.children.push(child);
            node.children.last_mut().unwrap()
        };

        insert_path(child, full_path, &components[1..], depth + 1, is_dir);
    }
}

fn prune_empty_dirs(node: &mut TreeNode) {
    // Recursively prune children first
    for child in &mut node.children {
        prune_empty_dirs(child);
    }

    // Remove empty directories
    node.children.retain(|child| {
        !child.is_dir || !child.children.is_empty()
    });
}
