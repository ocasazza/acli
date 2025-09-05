//! Search functionality for tree navigation

use crate::ui::Ui;
use sublime_fuzzy::{FuzzySearch, Scoring};

/// Search manager for fuzzy finding in tree items
pub struct SearchManager {
    /// Search mode state
    pub search_mode: bool,
    /// Current search query
    pub search_query: String,
    /// Filtered tree items (when searching) - (text, depth, selected, score, match_positions, original_index)
    pub filtered_tree_items: Option<Vec<(String, usize, bool, isize, Vec<usize>, usize)>>,
}

impl Default for SearchManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchManager {
    /// Create a new search manager
    pub fn new() -> Self {
        Self {
            search_mode: false,
            search_query: String::new(),
            filtered_tree_items: None,
        }
    }

    /// Enter search mode
    pub fn enter_search_mode(&mut self, ui: &mut Ui) {
        self.search_mode = true;
        self.search_query.clear();
        self.filtered_tree_items = None;
        ui.set_status("Search mode: type to filter, Esc to exit".to_string());
    }

    /// Exit search mode
    pub fn exit_search_mode(&mut self, ui: &mut Ui) {
        self.search_mode = false;
        self.search_query.clear();
        self.filtered_tree_items = None;
        ui.set_status("Ready".to_string());
    }

    /// Exit search mode but keep filter active
    pub fn exit_search_mode_keep_filter(&mut self, ui: &mut Ui) {
        if !self.search_query.is_empty() {
            self.search_mode = false;
            ui.set_status(format!("Filtered by: '{}'", self.search_query));
        } else {
            self.exit_search_mode(ui);
        }
    }

    /// Add character to search query and update filter
    pub fn add_to_query(&mut self, c: char, tree_items: &[(String, usize, bool)]) -> usize {
        self.search_query.push(c);
        self.update_search_filter(tree_items)
    }

    /// Remove character from search query and update filter
    pub fn remove_from_query(&mut self, tree_items: &[(String, usize, bool)]) -> usize {
        if !self.search_query.is_empty() {
            self.search_query.pop();
            self.update_search_filter(tree_items)
        } else {
            0
        }
    }

    /// Update search filter using fuzzy matching
    fn update_search_filter(&mut self, tree_items: &[(String, usize, bool)]) -> usize {
        if self.search_query.is_empty() {
            self.filtered_tree_items = None;
            return 0;
        }

        // Configure fuzzy matching with fzf-like scoring
        let scoring = Scoring::emphasize_word_starts();

        let mut matches: Vec<(String, usize, bool, isize, Vec<usize>, usize)> = tree_items
            .iter()
            .enumerate()
            .filter_map(|(original_index, (name, depth, selected))| {
                // Clean text for matching by removing icons and formatting
                let clean_text = self.extract_clean_text(name);

                // Use fuzzy search to find matches
                if let Some(fuzzy_match) = FuzzySearch::new(&self.search_query, &clean_text)
                    .case_insensitive()
                    .score_with(&scoring)
                    .best_match()
                {
                    let score = fuzzy_match.score();
                    // Find character positions in the clean text that match our query
                    let positions = self.find_match_positions(&clean_text, &self.search_query);

                    Some((
                        name.clone(),
                        *depth,
                        *selected,
                        score,
                        positions,
                        original_index,
                    ))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first) for fzf-like ranking
        matches.sort_by(|a, b| b.3.cmp(&a.3));

        self.filtered_tree_items = Some(matches);
        0 // Reset selection to top
    }

    /// Extract clean text from tree item name (removing icons and formatting)
    fn extract_clean_text(&self, display_text: &str) -> String {
        // Remove common prefixes and icons to get cleaner text for matching
        let text = display_text
            .replace("▼ ", "")
            .replace("▶ ", "")
            .replace("  ", " ")
            .replace("🌐", "")
            .replace("📊", "")
            .replace("📋", "")
            .replace("🎫", "")
            .replace("⭕", "")
            .replace("📁", "")
            .trim()
            .to_string();

        text
    }

    /// Find character positions that match the query (simple implementation)
    fn find_match_positions(&self, text: &str, query: &str) -> Vec<usize> {
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();
        let mut positions = Vec::new();

        // Simple sequential matching - find each character of query in text
        let text_chars: Vec<char> = text_lower.chars().collect();
        let query_chars: Vec<char> = query_lower.chars().collect();

        let mut text_idx = 0;
        for query_char in query_chars {
            while text_idx < text_chars.len() {
                if text_chars[text_idx] == query_char {
                    positions.push(text_idx);
                    text_idx += 1;
                    break;
                }
                text_idx += 1;
            }
        }

        positions
    }

    /// Get the items to display (either filtered or full tree)
    pub fn get_display_items(
        &self,
        tree_items: &[(String, usize, bool)],
    ) -> Vec<(String, usize, bool)> {
        if let Some(ref filtered) = self.filtered_tree_items {
            // Convert from fuzzy match format to display format
            filtered
                .iter()
                .map(
                    |(name, depth, selected, _score, _positions, _original_index)| {
                        (name.clone(), *depth, *selected)
                    },
                )
                .collect()
        } else {
            tree_items.to_vec()
        }
    }

    /// Get fuzzy search results with highlighting information
    pub fn get_fuzzy_display_items(
        &self,
    ) -> Option<&Vec<(String, usize, bool, isize, Vec<usize>, usize)>> {
        self.filtered_tree_items.as_ref()
    }

    /// Get the original tree index for a filtered result at the given position
    pub fn get_original_index_for_filtered_item(&self, filtered_index: usize) -> Option<usize> {
        if let Some(ref filtered) = self.filtered_tree_items {
            if filtered_index < filtered.len() {
                Some(filtered[filtered_index].5) // Return the original_index (6th element)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Clear search data to prevent memory leaks
    pub fn cleanup(&mut self) {
        self.search_query.clear();
        self.filtered_tree_items = None;
        self.search_mode = false;
    }
}
