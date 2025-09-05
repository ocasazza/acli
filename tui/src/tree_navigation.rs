//! Tree navigation and management functionality

use crate::models::{AtlassianDomain, ProductType, TreeNode, NavigationContext};
use std::error::Error;

/// Tree navigation manager
pub struct TreeNavigationManager {
    /// Tree data for navigation
    pub tree_data: Vec<TreeNode>,
    /// Current tree selection index
    pub tree_selection: usize,
    /// Navigation context for hierarchical selection
    pub navigation_context: NavigationContext,
}

impl Default for TreeNavigationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeNavigationManager {
    /// Create a new tree navigation manager
    pub fn new() -> Self {
        Self {
            tree_data: Vec::new(),
            tree_selection: 0,
            navigation_context: NavigationContext::new(),
        }
    }

    /// Build tree data structure from domain
    pub fn build_tree_data(&mut self, domain: AtlassianDomain) {
        let mut tree_nodes = Vec::new();

        for product in &domain.products {
            let mut product_node = TreeNode::new_product(product.clone());

            // Expand Confluence by default if it has projects
            if product.product_type == ProductType::Confluence && !product.projects.is_empty() {
                product_node.expanded = true;
            }

            for project in &product.projects {
                let project_node = TreeNode::new_project(project.clone());
                product_node.children.push(project_node);
            }

            tree_nodes.push(product_node);
        }

        self.tree_data = tree_nodes;
    }

    /// Get all visible tree items for display (flattened with indentation)
    pub fn get_tree_items(&self) -> Vec<(String, usize, bool)> {
        let mut items = Vec::new();
        for node in &self.tree_data {
            self.flatten_tree_node(node, 0, &mut items);
        }
        items
    }

    /// Recursively flatten tree nodes for display
    fn flatten_tree_node(&self, node: &TreeNode, depth: usize, items: &mut Vec<(String, usize, bool)>) {
        let prefix = "  ".repeat(depth);
        let icon = match &node.node_type {
            crate::models::TreeNodeType::Domain(_) => "🌐",
            crate::models::TreeNodeType::Product(product) => {
                if product.available {
                    match product.product_type {
                        ProductType::Confluence => "📊",
                        ProductType::Jira => "📋",
                        ProductType::Jsm => "🎫",
                    }
                } else {
                    "⭕"
                }
            },
            crate::models::TreeNodeType::Project(_) => "📁",
        };

        let expand_icon = if !node.children.is_empty() {
            if node.expanded { "▼ " } else { "▶ " }
        } else {
            "  "
        };

        let name = format!("{}{}{} {}", prefix, expand_icon, icon, node.name);
        items.push((name, depth, node.selected));

        if node.expanded {
            for child in &node.children {
                self.flatten_tree_node(child, depth + 1, items);
            }
        }
    }

    /// Move tree selection up
    pub fn move_selection_up(&mut self) {
        if self.tree_selection > 0 {
            self.tree_selection -= 1;
        }
    }

    /// Move tree selection down
    pub fn move_selection_down(&mut self) {
        let tree_items = self.get_tree_items();
        if self.tree_selection < tree_items.len().saturating_sub(1) {
            self.tree_selection += 1;
        }
    }

    /// Move selection up by pages (10 items)
    pub fn page_up(&mut self) {
        for _ in 0..10 {
            if self.tree_selection > 0 {
                self.tree_selection -= 1;
            } else {
                break;
            }
        }
    }

    /// Move selection down by pages (10 items)
    pub fn page_down(&mut self) {
        let tree_items = self.get_tree_items();
        for _ in 0..10 {
            if self.tree_selection < tree_items.len().saturating_sub(1) {
                self.tree_selection += 1;
            } else {
                break;
            }
        }
    }

    /// Select the current tree node
    pub fn select_current_node(&mut self, domain: Option<&AtlassianDomain>) -> Result<(), Box<dyn Error>> {
        let tree_items = self.get_tree_items();
        if self.tree_selection < tree_items.len() {
            if let Some(node_path) = self.get_node_path_at_index(self.tree_selection) {
                self.update_navigation_context(&node_path, domain)?;
            }
        }
        Ok(())
    }

    /// Select the current tree node and automatically select/expand parents
    pub fn select_current_node_with_parents(&mut self, domain: Option<&AtlassianDomain>) -> Result<(), Box<dyn Error>> {
        let tree_items = self.get_tree_items();
        if self.tree_selection < tree_items.len() {
            if let Some(node_path) = self.get_node_path_at_index(self.tree_selection) {
                // If this is a child node (project/space), automatically expand and select the parent product
                if node_path.len() > 1 {
                    // Expand the parent product
                    let parent_path = &node_path[0..1];
                    self.set_node_expanded(parent_path, true);

                    // Update navigation context to include both parent and child
                    self.update_navigation_context_with_parents(&node_path, domain)?;
                } else {
                    // This is a root node (product), use normal selection
                    self.update_navigation_context(&node_path, domain)?;
                }
            }
        }
        Ok(())
    }

    /// Expand the current node
    pub fn expand_current_node(&mut self) {
        if let Some(node_path) = self.get_node_path_at_index(self.tree_selection) {
            self.set_node_expanded(&node_path, true);
        }
    }

    /// Collapse the current node
    pub fn collapse_current_node(&mut self) {
        if let Some(node_path) = self.get_node_path_at_index(self.tree_selection) {
            self.set_node_expanded(&node_path, false);
        }
    }

    /// Get the path to a node at the given flattened index
    fn get_node_path_at_index(&self, index: usize) -> Option<Vec<usize>> {
        let mut current_index = 0;
        for (root_index, root_node) in self.tree_data.iter().enumerate() {
            if let Some(path) = self.find_node_path_recursive(root_node, index, &mut current_index, vec![root_index]) {
                return Some(path);
            }
        }
        None
    }

    /// Recursively find the path to a node at the given index
    fn find_node_path_recursive(&self, node: &TreeNode, target_index: usize, current_index: &mut usize, path: Vec<usize>) -> Option<Vec<usize>> {
        if *current_index == target_index {
            return Some(path);
        }
        *current_index += 1;

        if node.expanded {
            for (child_index, child) in node.children.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(child_index);
                if let Some(found_path) = self.find_node_path_recursive(child, target_index, current_index, child_path) {
                    return Some(found_path);
                }
            }
        }
        None
    }

    /// Set expansion state of a node at the given path
    fn set_node_expanded(&mut self, path: &[usize], expanded: bool) {
        if path.is_empty() {
            return;
        }

        let mut current_node = &mut self.tree_data[path[0]];
        for &index in &path[1..] {
            if index < current_node.children.len() {
                current_node = &mut current_node.children[index];
            } else {
                return;
            }
        }
        current_node.expanded = expanded;
    }

    /// Update navigation context based on the selected node path
    fn update_navigation_context(&mut self, path: &[usize], domain: Option<&AtlassianDomain>) -> Result<(), Box<dyn Error>> {
        if path.is_empty() {
            return Ok(());
        }

        // Set domain from stored domain (since products are now root items)
        if let Some(domain) = domain {
            self.navigation_context.domain = Some(domain.clone());
        }

        let mut current_node = &self.tree_data[path[0]];

        // Handle root node (which is now a product)
        if let crate::models::TreeNodeType::Product(product) = &current_node.node_type {
            self.navigation_context.product = Some(product.clone());
            self.navigation_context.project = None; // Reset project when selecting product
        }

        // Navigate to child nodes if any
        for &index in &path[1..] {
            if index < current_node.children.len() {
                current_node = &current_node.children[index];

                if let crate::models::TreeNodeType::Project(project) = &current_node.node_type {
                    self.navigation_context.project = Some(project.clone());
                }
            }
        }

        // Update selected state in tree
        self.clear_all_selections();
        self.set_node_selected(path, true);

        Ok(())
    }

    /// Update navigation context with automatic parent selection for child nodes
    fn update_navigation_context_with_parents(&mut self, path: &[usize], domain: Option<&AtlassianDomain>) -> Result<(), Box<dyn Error>> {
        if path.is_empty() {
            return Ok(());
        }

        // Set domain from stored domain
        if let Some(domain) = domain {
            self.navigation_context.domain = Some(domain.clone());
        }

        // Get the parent product (root node)
        let parent_node = &self.tree_data[path[0]];
        if let crate::models::TreeNodeType::Product(product) = &parent_node.node_type {
            self.navigation_context.product = Some(product.clone());
        }

        // Navigate to the child node
        let mut current_node = parent_node;
        for &index in &path[1..] {
            if index < current_node.children.len() {
                current_node = &current_node.children[index];

                if let crate::models::TreeNodeType::Project(project) = &current_node.node_type {
                    self.navigation_context.project = Some(project.clone());
                }
            }
        }

        // Update selected state in tree - select both parent and child
        self.clear_all_selections();

        // Select the parent product
        let parent_path = &path[0..1];
        self.set_node_selected(parent_path, true);

        // Select the child project/space
        self.set_node_selected(path, true);

        Ok(())
    }

    /// Clear all selections in the tree
    fn clear_all_selections(&mut self) {
        for root_node in &mut self.tree_data {
            Self::clear_selections_recursive(root_node);
        }
    }

    /// Recursively clear selections
    fn clear_selections_recursive(node: &mut TreeNode) {
        node.selected = false;
        for child in &mut node.children {
            Self::clear_selections_recursive(child);
        }
    }

    /// Set selection state of a node at the given path
    fn set_node_selected(&mut self, path: &[usize], selected: bool) {
        if path.is_empty() {
            return;
        }

        let mut current_node = &mut self.tree_data[path[0]];
        for &index in &path[1..] {
            if index < current_node.children.len() {
                current_node = &mut current_node.children[index];
            } else {
                return;
            }
        }
        current_node.selected = selected;
    }

    /// Clear tree data to prevent memory leaks
    pub fn cleanup(&mut self) {
        self.tree_data.clear();
        self.tree_selection = 0;
    }
}
