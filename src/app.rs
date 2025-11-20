use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::file_tree::{build_tree, TreeNode};
use crate::markdown::{render_markdown, RenderedLine, TextSegment};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Sidebar,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Search,
    Settings,
    DocumentSearch,
    Select, // Mode for text selection (disables mouse capture)
    About,  // About window
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WrapWidth {
    Chars80,
    Chars120,
    NoWrap,
}

impl WrapWidth {
    pub fn to_usize(self) -> Option<usize> {
        match self {
            WrapWidth::Chars80 => Some(80),
            WrapWidth::Chars120 => Some(120),
            WrapWidth::NoWrap => None,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            WrapWidth::Chars80 => "80 characters",
            WrapWidth::Chars120 => "120 characters",
            WrapWidth::NoWrap => "No wrap",
        }
    }

    pub fn next(self) -> Self {
        match self {
            WrapWidth::Chars80 => WrapWidth::Chars120,
            WrapWidth::Chars120 => WrapWidth::NoWrap,
            WrapWidth::NoWrap => WrapWidth::Chars80,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub show_line_numbers: bool,
    pub theme: Theme,
    pub wrap_width: WrapWidth,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            theme: Theme::Dark,
            wrap_width: WrapWidth::Chars120,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("rmd").join("settings.json");
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(settings) = serde_json::from_str(&content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let app_config_dir = config_dir.join("rmd");
            if fs::create_dir_all(&app_config_dir).is_ok() {
                let config_path = app_config_dir.join("settings.json");
                if let Ok(content) = serde_json::to_string_pretty(self) {
                    let _ = fs::write(config_path, content);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub name: String,
    pub match_preview: String,
}

pub struct App {
    pub tree: TreeNode,
    pub selected_index: usize,
    pub content_scroll: usize,
    pub focus: Focus,
    pub current_file: Option<PathBuf>,
    pub rendered_content: Vec<RenderedLine>,
    pub content_height: u16,
    pub mode: AppMode,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub search_selected: usize,
    #[allow(dead_code)]
    pub root_path: PathBuf,
    pub settings: Settings,
    pub settings_selected: usize,
    pub doc_search_query: String,
    pub doc_search_matches: Vec<usize>, // Line indices that match
    pub doc_search_current: usize,      // Current match index
    pub pending_g: bool,                // Track if 'g' was pressed for 'gg' combo
}

impl App {
    pub fn new(root_path: PathBuf) -> Self {
        let tree = build_tree(&root_path);

        // Create welcome message as RenderedLines
        let welcome_content = vec![
            create_heading_line("Welcome to rmd!", 1),
            RenderedLine::new_empty(),
            create_plain_line("Select a Markdown file from the sidebar to view its contents."),
            RenderedLine::new_empty(),
            create_heading_line("Navigation", 2),
            RenderedLine::new_empty(),
            create_plain_line("  j/↓       Move down"),
            create_plain_line("  k/↑       Move up"),
            create_plain_line("  l/→/Enter Open file / Expand directory"),
            create_plain_line("  h/←       Collapse directory / Go to parent"),
            create_plain_line("  Tab       Switch focus between sidebar and content"),
            create_plain_line("  gg        Go to top"),
            create_plain_line("  G         Go to bottom"),
            RenderedLine::new_empty(),
            create_heading_line("Scrolling", 2),
            RenderedLine::new_empty(),
            create_plain_line("  Ctrl+u    Half page up"),
            create_plain_line("  Ctrl+d    Half page down"),
            create_plain_line("  Ctrl+b    Full page up"),
            create_plain_line("  Ctrl+f    Full page down"),
            RenderedLine::new_empty(),
            create_heading_line("Search", 2),
            RenderedLine::new_empty(),
            create_plain_line("  /         Search in document"),
            create_plain_line("  Ctrl+s    Search all files"),
            RenderedLine::new_empty(),
            create_heading_line("General", 2),
            RenderedLine::new_empty(),
            create_plain_line("  v         Select mode (for copying text)"),
            create_plain_line("  q         Quit"),
            create_plain_line("  Ctrl+p    Settings"),
        ];

        let mut app = Self {
            tree,
            selected_index: 0,
            content_scroll: 0,
            focus: Focus::Sidebar,
            current_file: None,
            rendered_content: welcome_content,
            content_height: 20,
            mode: AppMode::Normal,
            search_query: String::new(),
            search_results: Vec::new(),
            search_selected: 0,
            root_path: root_path.clone(),
            settings: Settings::load(),
            settings_selected: 0,
            doc_search_query: String::new(),
            doc_search_matches: Vec::new(),
            doc_search_current: 0,
            pending_g: false,
        };

        // Auto-select first markdown file if available
        app.select_first_file();
        app
    }

    fn select_first_file(&mut self) {
        let items = self.tree.visible_items();

        // First, look for README.md (case-insensitive) in the root
        for (idx, item) in items.iter().enumerate() {
            if !item.is_dir {
                let name_lower = item.name.to_lowercase();
                if name_lower == "readme.md" || name_lower == "readme.markdown" {
                    self.selected_index = idx;
                    self.load_file(&item.path.clone());
                    return;
                }
            }
        }

        // If no README found, select the first file
        for (idx, item) in items.iter().enumerate() {
            if !item.is_dir {
                self.selected_index = idx;
                self.load_file(&item.path.clone());
                break;
            }
        }
    }

    pub fn visible_items(&self) -> Vec<&TreeNode> {
        self.tree.visible_items()
    }

    pub fn next(&mut self) {
        let items = self.visible_items();
        if self.focus == Focus::Sidebar {
            if !items.is_empty() && self.selected_index < items.len() - 1 {
                self.selected_index += 1;
            }
        } else {
            // Scroll content down
            let max_scroll = self.total_wrapped_lines().saturating_sub(self.content_height as usize);
            if self.content_scroll < max_scroll {
                self.content_scroll += 1;
            }
        }
    }

    pub fn previous(&mut self) {
        if self.focus == Focus::Sidebar {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
        } else {
            // Scroll content up
            if self.content_scroll > 0 {
                self.content_scroll -= 1;
            }
        }
    }

    pub fn toggle_or_select(&mut self) {
        let items = self.visible_items();
        if let Some(item) = items.get(self.selected_index) {
            let path = item.path.clone();
            let is_dir = item.is_dir;

            if is_dir {
                // Toggle directory expansion
                if let Some(node) = self.tree.find_by_index_mut(self.selected_index) {
                    node.toggle_expanded();
                }
            } else {
                // Load file
                self.load_file(&path);
            }
        }
    }

    pub fn collapse_or_parent(&mut self) {
        let items = self.visible_items();
        if let Some(item) = items.get(self.selected_index) {
            if item.is_dir && item.expanded {
                // Collapse the directory
                if let Some(node) = self.tree.find_by_index_mut(self.selected_index) {
                    node.toggle_expanded();
                }
            } else {
                // Go to parent
                if let Some(parent_idx) = self.tree.find_parent_index(self.selected_index) {
                    self.selected_index = parent_idx;
                }
            }
        }
    }

    pub fn focus_content_or_select(&mut self) {
        if self.focus == Focus::Sidebar {
            // If on a file, load it and switch to content
            let items = self.visible_items();
            if let Some(item) = items.get(self.selected_index) {
                if item.is_dir {
                    // Expand directory
                    if let Some(node) = self.tree.find_by_index_mut(self.selected_index) {
                        if !node.expanded {
                            node.toggle_expanded();
                        }
                    }
                } else {
                    // Load file and focus content
                    let path = item.path.clone();
                    self.load_file(&path);
                    self.focus = Focus::Content;
                }
            }
        }
    }

    pub fn focus_sidebar_or_collapse(&mut self) {
        if self.focus == Focus::Content {
            self.focus = Focus::Sidebar;
        } else {
            // In sidebar, collapse or go to parent
            self.collapse_or_parent();
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Sidebar => Focus::Content,
            Focus::Content => Focus::Sidebar,
        };
    }

    pub fn scroll_to_top(&mut self) {
        if self.focus == Focus::Sidebar {
            self.selected_index = 0;
        } else {
            self.content_scroll = 0;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        if self.focus == Focus::Sidebar {
            let items = self.visible_items();
            if !items.is_empty() {
                self.selected_index = items.len() - 1;
            }
        } else {
            let max_scroll = self.total_wrapped_lines().saturating_sub(self.content_height as usize);
            self.content_scroll = max_scroll;
        }
    }

    pub fn page_up(&mut self) {
        if self.focus == Focus::Content {
            let page_size = self.content_height as usize;
            self.content_scroll = self.content_scroll.saturating_sub(page_size);
        } else {
            let page_size = 10;
            self.selected_index = self.selected_index.saturating_sub(page_size);
        }
    }

    pub fn page_down(&mut self) {
        if self.focus == Focus::Content {
            let page_size = self.content_height as usize;
            let max_scroll = self.total_wrapped_lines().saturating_sub(self.content_height as usize);
            self.content_scroll = (self.content_scroll + page_size).min(max_scroll);
        } else {
            let items = self.visible_items();
            let page_size = 10;
            self.selected_index = (self.selected_index + page_size).min(items.len().saturating_sub(1));
        }
    }

    pub fn scroll_content_to_top(&mut self) {
        self.content_scroll = 0;
    }

    pub fn scroll_content_to_bottom(&mut self) {
        let max_scroll = self.total_wrapped_lines().saturating_sub(self.content_height as usize);
        self.content_scroll = max_scroll;
    }

    fn load_file(&mut self, path: &PathBuf) {
        self.current_file = Some(path.clone());
        self.content_scroll = 0;

        match fs::read_to_string(path) {
            Ok(content) => {
                self.rendered_content = render_markdown(&content);
            }
            Err(e) => {
                self.rendered_content = vec![
                    create_plain_line(&format!("Error reading file: {}", e)),
                ];
            }
        }
    }

    pub fn set_content_height(&mut self, height: u16) {
        self.content_height = height.saturating_sub(2); // Account for borders
    }

    pub fn enter_search_mode(&mut self) {
        self.mode = AppMode::Search;
        self.search_query.clear();
        self.search_results.clear();
        self.search_selected = 0;
    }

    pub fn exit_search_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.search_query.clear();
        self.search_results.clear();
        self.search_selected = 0;
    }

    pub fn enter_settings_mode(&mut self) {
        self.mode = AppMode::Settings;
        self.settings_selected = 0;
    }

    pub fn exit_settings_mode(&mut self) {
        self.mode = AppMode::Normal;
    }

    pub fn settings_toggle_current(&mut self) {
        match self.settings_selected {
            0 => self.settings.show_line_numbers = !self.settings.show_line_numbers,
            1 => {
                self.settings.theme = match self.settings.theme {
                    Theme::Dark => Theme::Light,
                    Theme::Light => Theme::Dark,
                };
            }
            2 => {
                self.settings.wrap_width = self.settings.wrap_width.next();
            }
            _ => {}
        }
        self.settings.save();
    }

    pub fn settings_next(&mut self) {
        let max_settings = 2; // 0, 1, 2
        if self.settings_selected < max_settings {
            self.settings_selected += 1;
        }
    }

    pub fn settings_previous(&mut self) {
        if self.settings_selected > 0 {
            self.settings_selected -= 1;
        }
    }

    pub fn enter_doc_search_mode(&mut self) {
        self.mode = AppMode::DocumentSearch;
        self.doc_search_query.clear();
        self.doc_search_matches.clear();
        self.doc_search_current = 0;
    }

    pub fn exit_doc_search_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.doc_search_query.clear();
        self.doc_search_matches.clear();
        self.doc_search_current = 0;
    }

    pub fn doc_search_add_char(&mut self, c: char) {
        self.doc_search_query.push(c);
        self.perform_doc_search();
    }

    pub fn doc_search_backspace(&mut self) {
        self.doc_search_query.pop();
        self.perform_doc_search();
    }

    pub fn doc_search_next(&mut self) {
        if !self.doc_search_matches.is_empty() {
            self.doc_search_current = (self.doc_search_current + 1) % self.doc_search_matches.len();
            self.jump_to_current_match();
        }
    }

    pub fn doc_search_previous(&mut self) {
        if !self.doc_search_matches.is_empty() {
            if self.doc_search_current == 0 {
                self.doc_search_current = self.doc_search_matches.len() - 1;
            } else {
                self.doc_search_current -= 1;
            }
            self.jump_to_current_match();
        }
    }

    fn perform_doc_search(&mut self) {
        self.doc_search_matches.clear();
        self.doc_search_current = 0;

        if self.doc_search_query.is_empty() {
            return;
        }

        let query_lower = self.doc_search_query.to_lowercase();

        for (idx, line) in self.rendered_content.iter().enumerate() {
            let line_text: String = line.segments.iter().map(|seg| {
                match seg {
                    TextSegment::Plain(s) => s.clone(),
                    TextSegment::Code(s) => s.clone(),
                    TextSegment::Link { text, .. } => text.clone(),
                    TextSegment::Emphasis(s) => s.clone(),
                    TextSegment::Strong(s) => s.clone(),
                }
            }).collect();

            if line_text.to_lowercase().contains(&query_lower) {
                self.doc_search_matches.push(idx);
            }
        }

        // Jump to first match if found
        if !self.doc_search_matches.is_empty() {
            self.jump_to_current_match();
        }
    }

    fn jump_to_current_match(&mut self) {
        if let Some(&line_idx) = self.doc_search_matches.get(self.doc_search_current) {
            // Calculate wrapped line index from source line index
            let wrapped_idx = self.source_to_wrapped_index(line_idx);
            // Scroll to show the match, centered if possible
            let half_height = (self.content_height / 2) as usize;
            self.content_scroll = wrapped_idx.saturating_sub(half_height);
        }
    }

    /// Convert a source line index to its wrapped line index
    fn source_to_wrapped_index(&self, source_idx: usize) -> usize {
        let max_width = self.settings.wrap_width.to_usize();
        let mut wrapped_idx = 0;

        for (idx, line) in self.rendered_content.iter().enumerate() {
            if idx == source_idx {
                return wrapped_idx;
            }
            // Count how many wrapped lines this source line produces
            wrapped_idx += self.count_wrapped_lines(line, max_width);
        }

        wrapped_idx
    }

    /// Count how many wrapped lines a single RenderedLine produces
    fn count_wrapped_lines(&self, line: &RenderedLine, max_width: Option<usize>) -> usize {
        if line.segments.is_empty() {
            return 1;
        }

        // Don't wrap table rows or separators
        if line.is_table_row || line.is_table_separator {
            return 1;
        }

        // If no wrapping, return 1
        let max_width = match max_width {
            Some(w) => w,
            None => return 1,
        };

        // Get the full text content to check length
        let full_text: String = line.segments.iter().map(|seg| {
            match seg {
                TextSegment::Plain(s) => s.clone(),
                TextSegment::Code(s) => format!("`{}`", s),
                TextSegment::Link { text, .. } => text.clone(),
                TextSegment::Emphasis(s) => s.clone(),
                TextSegment::Strong(s) => s.clone(),
            }
        }).collect();

        // If line fits, no wrapping
        if full_text.len() <= max_width {
            return 1;
        }

        // Count wrapped lines
        let mut count = 0;
        let mut current_len = 0;

        for word in full_text.split_whitespace() {
            if current_len == 0 {
                current_len = word.len();
            } else if current_len + 1 + word.len() <= max_width {
                current_len += 1 + word.len();
            } else {
                count += 1;
                current_len = word.len();
            }
        }

        if current_len > 0 {
            count += 1;
        }

        if count == 0 {
            1
        } else {
            count
        }
    }

    /// Get total number of wrapped lines
    pub fn total_wrapped_lines(&self) -> usize {
        let max_width = self.settings.wrap_width.to_usize();
        self.rendered_content.iter()
            .map(|line| self.count_wrapped_lines(line, max_width))
            .sum()
    }

    pub fn search_add_char(&mut self, c: char) {
        self.search_query.push(c);
        self.perform_search();
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.perform_search();
    }

    pub fn search_next(&mut self) {
        if !self.search_results.is_empty() && self.search_selected < self.search_results.len() - 1 {
            self.search_selected += 1;
        }
    }

    pub fn search_previous(&mut self) {
        if self.search_selected > 0 {
            self.search_selected -= 1;
        }
    }

    pub fn search_select(&mut self) {
        if let Some(result) = self.search_results.get(self.search_selected) {
            let path = result.path.clone();
            let query = self.search_query.clone();
            self.load_file(&path);
            self.exit_search_mode();
            self.focus = Focus::Content;

            // Automatically open document search with the same query
            self.doc_search_query = query;
            self.perform_doc_search();
            self.mode = AppMode::DocumentSearch;
        }
    }

    fn perform_search(&mut self) {
        self.search_results.clear();
        self.search_selected = 0;

        if self.search_query.is_empty() {
            return;
        }

        let query_lower = self.search_query.to_lowercase();

        // Collect all markdown files from the tree
        let files = self.collect_all_files(&self.tree.clone());

        for file_path in files {
            if let Ok(content) = fs::read_to_string(&file_path) {
                // Find first matching line as preview
                let mut match_preview = String::new();
                let mut match_count = 0;

                for line in content.lines() {
                    if line.to_lowercase().contains(&query_lower) {
                        if match_preview.is_empty() {
                            match_preview = line.trim().chars().take(60).collect();
                        }
                        match_count += 1;
                    }
                }

                if match_count > 0 {
                    let name = file_path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let preview = if match_count > 1 {
                        format!("{} ({} matches)", match_preview, match_count)
                    } else {
                        match_preview
                    };

                    self.search_results.push(SearchResult {
                        path: file_path.clone(),
                        name,
                        match_preview: preview,
                    });

                    // Limit results
                    if self.search_results.len() >= 50 {
                        return;
                    }
                }
            }
        }
    }

    fn collect_all_files(&self, node: &TreeNode) -> Vec<PathBuf> {
        let mut files = Vec::new();

        if !node.is_dir {
            files.push(node.path.clone());
        }

        for child in &node.children {
            files.extend(self.collect_all_files(child));
        }

        files
    }
}

// Helper functions
fn create_plain_line(text: &str) -> RenderedLine {
    let mut line = RenderedLine {
        segments: Vec::new(),
        heading_level: 0,
        is_code_block: false,
        is_blockquote: false,
        is_list_item: false,
        is_horizontal_rule: false,
        is_table_row: false,
        is_table_separator: false,
    };
    line.segments.push(TextSegment::Plain(text.to_string()));
    line
}

fn create_heading_line(text: &str, level: u8) -> RenderedLine {
    let mut line = RenderedLine {
        segments: Vec::new(),
        heading_level: level,
        is_code_block: false,
        is_blockquote: false,
        is_list_item: false,
        is_horizontal_rule: false,
        is_table_row: false,
        is_table_separator: false,
    };
    line.segments.push(TextSegment::Plain(text.to_string()));
    line
}

impl RenderedLine {
    pub fn new_empty() -> Self {
        Self {
            segments: Vec::new(),
            heading_level: 0,
            is_code_block: false,
            is_blockquote: false,
            is_list_item: false,
            is_horizontal_rule: false,
            is_table_row: false,
            is_table_separator: false,
        }
    }
}
