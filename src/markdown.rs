use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd, CodeBlockKind, Options};

/// Represents a segment of styled text
#[derive(Debug, Clone)]
pub enum TextSegment {
    Plain(String),
    Code(String),
    #[allow(dead_code)]
    Link { text: String, url: String },
    Emphasis(String),
    Strong(String),
}

/// Represents a rendered line with its heading level (0 = not a heading)
#[derive(Debug, Clone, Default)]
pub struct RenderedLine {
    pub segments: Vec<TextSegment>,
    pub heading_level: u8,
    pub is_code_block: bool,
    pub is_blockquote: bool,
    pub is_list_item: bool,
    pub is_horizontal_rule: bool,
    pub is_table_row: bool,
    pub is_table_separator: bool,
}

impl RenderedLine {
    fn new() -> Self {
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

    fn push_plain(&mut self, text: String) {
        if !text.is_empty() {
            self.segments.push(TextSegment::Plain(text));
        }
    }

    fn push_code(&mut self, text: String) {
        self.segments.push(TextSegment::Code(text));
    }

    fn push_link(&mut self, text: String, url: String) {
        self.segments.push(TextSegment::Link { text, url });
    }

    fn push_emphasis(&mut self, text: String) {
        self.segments.push(TextSegment::Emphasis(text));
    }

    fn push_strong(&mut self, text: String) {
        self.segments.push(TextSegment::Strong(text));
    }

    #[allow(dead_code)]
    pub fn to_plain_string(&self) -> String {
        self.segments
            .iter()
            .map(|seg| match seg {
                TextSegment::Plain(s) => s.clone(),
                TextSegment::Code(s) => format!("`{}`", s),
                TextSegment::Link { text, .. } => text.clone(),
                TextSegment::Emphasis(s) => s.clone(),
                TextSegment::Strong(s) => s.clone(),
            })
            .collect()
    }
}

/// Render markdown to styled lines
pub fn render_markdown(content: &str) -> Vec<RenderedLine> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(content, options);
    let mut lines: Vec<RenderedLine> = Vec::new();
    let mut current_line = RenderedLine::new();
    let mut current_text = String::new();

    let mut in_code_block = false;
    let mut list_depth: usize = 0;
    let mut ordered_list_index: Vec<u64> = Vec::new();
    let mut in_blockquote = false;
    let mut in_link = false;
    let mut link_url = String::new();
    let mut link_text = String::new();
    let mut in_emphasis = false;
    let mut emphasis_text = String::new();
    let mut in_strong = false;
    let mut strong_text = String::new();

    // Table state
    let mut in_table = false;
    let mut table_row_cells: Vec<String> = Vec::new();
    let mut table_cell_text = String::new();
    let mut table_column_widths: Vec<usize> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut is_first_row = false;

    for event in parser {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Heading { level, .. } => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        if !current_line.segments.is_empty() {
                            lines.push(std::mem::take(&mut current_line));
                            current_line = RenderedLine::new();
                        }
                        let heading_level = match level {
                            HeadingLevel::H1 => 1,
                            HeadingLevel::H2 => 2,
                            HeadingLevel::H3 => 3,
                            HeadingLevel::H4 => 4,
                            HeadingLevel::H5 => 5,
                            HeadingLevel::H6 => 6,
                        };
                        current_line.heading_level = heading_level;
                        // Add # prefix to show heading level
                        current_text.push_str(&"#".repeat(heading_level as usize));
                        current_text.push(' ');
                    }
                    Tag::Paragraph => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        if !current_line.segments.is_empty() {
                            lines.push(std::mem::take(&mut current_line));
                            current_line = RenderedLine::new();
                        }
                    }
                    Tag::CodeBlock(kind) => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        if !current_line.segments.is_empty() {
                            lines.push(std::mem::take(&mut current_line));
                            current_line = RenderedLine::new();
                        }
                        in_code_block = true;
                        let lang = match kind {
                            CodeBlockKind::Fenced(lang) => lang.to_string(),
                            CodeBlockKind::Indented => String::new(),
                        };
                        let mut marker_line = RenderedLine::new();
                        marker_line.is_code_block = true;
                        if lang.is_empty() {
                            marker_line.push_plain("```".to_string());
                        } else {
                            marker_line.push_plain(format!("```{}", lang));
                        }
                        lines.push(marker_line);
                    }
                    Tag::List(start) => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        if !current_line.segments.is_empty() {
                            lines.push(std::mem::take(&mut current_line));
                            current_line = RenderedLine::new();
                        }
                        list_depth += 1;
                        if let Some(n) = start {
                            ordered_list_index.push(n);
                        } else {
                            ordered_list_index.push(0);
                        }
                    }
                    Tag::Item => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        if !current_line.segments.is_empty() {
                            lines.push(std::mem::take(&mut current_line));
                            current_line = RenderedLine::new();
                        }
                        current_line.is_list_item = true;
                        let indent = "  ".repeat(list_depth.saturating_sub(1));
                        if let Some(&idx) = ordered_list_index.last() {
                            if idx > 0 {
                                current_text.push_str(&format!("{}{}. ", indent, idx));
                                if let Some(last) = ordered_list_index.last_mut() {
                                    *last += 1;
                                }
                            } else {
                                current_text.push_str(&format!("{}• ", indent));
                            }
                        }
                    }
                    Tag::BlockQuote(_) => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        if !current_line.segments.is_empty() {
                            lines.push(std::mem::take(&mut current_line));
                            current_line = RenderedLine::new();
                        }
                        in_blockquote = true;
                    }
                    Tag::Emphasis => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        in_emphasis = true;
                        emphasis_text.clear();
                    }
                    Tag::Strong => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        in_strong = true;
                        strong_text.clear();
                    }
                    Tag::Strikethrough => {}
                    Tag::Link { dest_url, .. } => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        in_link = true;
                        link_url = dest_url.to_string();
                        link_text.clear();
                    }
                    Tag::Image { dest_url, title, .. } => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        let display = if !title.is_empty() {
                            title.to_string()
                        } else {
                            "[image]".to_string()
                        };
                        current_line.push_link(display, dest_url.to_string());
                    }
                    Tag::Table(_alignments) => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        if !current_line.segments.is_empty() {
                            lines.push(std::mem::take(&mut current_line));
                            current_line = RenderedLine::new();
                        }
                        in_table = true;
                        table_rows.clear();
                        table_column_widths.clear();
                        is_first_row = true;
                    }
                    Tag::TableHead => {
                        table_row_cells.clear();
                    }
                    Tag::TableRow => {
                        table_row_cells.clear();
                    }
                    Tag::TableCell => {
                        table_cell_text.clear();
                    }
                    _ => {}
                }
            }
            Event::End(tag) => {
                match tag {
                    TagEnd::Heading(_) => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        lines.push(std::mem::take(&mut current_line));
                        current_line = RenderedLine::new();

                        // Add blank line after headings
                        lines.push(RenderedLine::new());
                    }
                    TagEnd::Paragraph => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        lines.push(std::mem::take(&mut current_line));
                        current_line = RenderedLine::new();
                        lines.push(RenderedLine::new());
                    }
                    TagEnd::CodeBlock => {
                        in_code_block = false;
                        let mut marker_line = RenderedLine::new();
                        marker_line.is_code_block = true;
                        marker_line.push_plain("```".to_string());
                        lines.push(marker_line);
                        lines.push(RenderedLine::new());
                    }
                    TagEnd::List(_) => {
                        list_depth = list_depth.saturating_sub(1);
                        ordered_list_index.pop();
                        if list_depth == 0 {
                            lines.push(RenderedLine::new());
                        }
                    }
                    TagEnd::Item => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        lines.push(std::mem::take(&mut current_line));
                        current_line = RenderedLine::new();
                    }
                    TagEnd::BlockQuote(_) => {
                        if !current_text.is_empty() {
                            current_line.push_plain(std::mem::take(&mut current_text));
                        }
                        in_blockquote = false;
                        lines.push(std::mem::take(&mut current_line));
                        current_line = RenderedLine::new();
                        lines.push(RenderedLine::new());
                    }
                    TagEnd::Emphasis => {
                        current_line.push_emphasis(std::mem::take(&mut emphasis_text));
                        in_emphasis = false;
                    }
                    TagEnd::Strong => {
                        current_line.push_strong(std::mem::take(&mut strong_text));
                        in_strong = false;
                    }
                    TagEnd::Strikethrough => {}
                    TagEnd::Link => {
                        current_line.push_link(
                            std::mem::take(&mut link_text),
                            std::mem::take(&mut link_url),
                        );
                        in_link = false;
                    }
                    TagEnd::Image => {}
                    TagEnd::Table => {
                        // Now render the table with proper formatting
                        // Calculate column widths
                        let num_cols = table_column_widths.len();

                        // Add top border
                        let mut top_str = String::from("┌");
                        for (i, &width) in table_column_widths.iter().enumerate() {
                            top_str.push_str(&"─".repeat(width + 2));
                            if i < num_cols - 1 {
                                top_str.push('┬');
                            }
                        }
                        top_str.push('┐');
                        let mut top_line = RenderedLine::new();
                        top_line.is_table_separator = true;
                        top_line.is_table_row = true;
                        top_line.push_plain(top_str);
                        lines.push(top_line);

                        for row in &table_rows {
                            // Build the row string with proper spacing
                            let mut row_str = String::from("│");
                            for (i, cell) in row.iter().enumerate() {
                                let width = if i < num_cols { table_column_widths[i] } else { cell.len() };
                                row_str.push_str(&format!(" {:<width$} │", cell, width = width));
                            }

                            let mut table_line = RenderedLine::new();
                            table_line.is_table_row = true;
                            table_line.push_plain(row_str);
                            lines.push(table_line);

                            // Add separator after header row
                            if is_first_row && !table_rows.is_empty() && row == &table_rows[0] {
                                let mut sep_str = String::from("├");
                                for (i, &width) in table_column_widths.iter().enumerate() {
                                    sep_str.push_str(&"─".repeat(width + 2));
                                    if i < num_cols - 1 {
                                        sep_str.push('┼');
                                    }
                                }
                                sep_str.push('┤');

                                let mut sep_line = RenderedLine::new();
                                sep_line.is_table_separator = true;
                                sep_line.is_table_row = true;
                                sep_line.push_plain(sep_str);
                                lines.push(sep_line);
                            }
                        }

                        // Add bottom border
                        let mut bottom_str = String::from("└");
                        for (i, &width) in table_column_widths.iter().enumerate() {
                            bottom_str.push_str(&"─".repeat(width + 2));
                            if i < num_cols - 1 {
                                bottom_str.push('┴');
                            }
                        }
                        bottom_str.push('┘');
                        let mut bottom_line = RenderedLine::new();
                        bottom_line.is_table_separator = true;
                        bottom_line.is_table_row = true;
                        bottom_line.push_plain(bottom_str);
                        lines.push(bottom_line);

                        lines.push(RenderedLine::new());
                        in_table = false;
                        is_first_row = false;
                    }
                    TagEnd::TableHead => {
                    }
                    TagEnd::TableRow => {
                        // Store the row
                        let row = std::mem::take(&mut table_row_cells);

                        // Update column widths
                        for (i, cell) in row.iter().enumerate() {
                            if i >= table_column_widths.len() {
                                table_column_widths.push(cell.len());
                            } else if cell.len() > table_column_widths[i] {
                                table_column_widths[i] = cell.len();
                            }
                        }

                        table_rows.push(row);
                    }
                    TagEnd::TableCell => {
                        table_row_cells.push(std::mem::take(&mut table_cell_text));
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                if in_table {
                    table_cell_text.push_str(&text);
                } else if in_code_block {
                    for line in text.lines() {
                        let mut code_line = RenderedLine::new();
                        code_line.is_code_block = true;
                        code_line.push_plain(format!("  {}", line));
                        lines.push(code_line);
                    }
                } else if in_blockquote {
                    for line in text.lines() {
                        let mut quote_line = RenderedLine::new();
                        quote_line.is_blockquote = true;
                        quote_line.push_plain(format!("│ {}", line));
                        lines.push(quote_line);
                    }
                } else if in_link {
                    link_text.push_str(&text);
                } else if in_emphasis {
                    emphasis_text.push_str(&text);
                } else if in_strong {
                    strong_text.push_str(&text);
                } else {
                    current_text.push_str(&text);
                }
            }
            Event::Code(code) => {
                if in_table {
                    // In tables, just add code as text with backticks
                    table_cell_text.push_str(&format!("`{}`", code));
                } else {
                    if !current_text.is_empty() {
                        current_line.push_plain(std::mem::take(&mut current_text));
                    }
                    current_line.push_code(code.to_string());
                }
            }
            Event::SoftBreak => {
                if in_link {
                    link_text.push(' ');
                } else if in_emphasis {
                    emphasis_text.push(' ');
                } else if in_strong {
                    strong_text.push(' ');
                } else {
                    current_text.push(' ');
                }
            }
            Event::HardBreak => {
                if !current_text.is_empty() {
                    current_line.push_plain(std::mem::take(&mut current_text));
                }
                lines.push(std::mem::take(&mut current_line));
                current_line = RenderedLine::new();
            }
            Event::Rule => {
                if !current_text.is_empty() {
                    current_line.push_plain(std::mem::take(&mut current_text));
                }
                if !current_line.segments.is_empty() {
                    lines.push(std::mem::take(&mut current_line));
                    current_line = RenderedLine::new();
                }
                let mut rule_line = RenderedLine::new();
                rule_line.is_horizontal_rule = true;
                rule_line.push_plain("────────────────────────────────────────".to_string());
                lines.push(rule_line);
                lines.push(RenderedLine::new());
            }
            _ => {}
        }
    }

    if !current_text.is_empty() {
        current_line.push_plain(current_text);
    }
    if !current_line.segments.is_empty() {
        lines.push(current_line);
    }

    // Remove trailing empty lines
    while lines.last().map(|l| l.segments.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    if lines.is_empty() {
        let mut empty_line = RenderedLine::new();
        empty_line.push_plain("(Empty file)".to_string());
        lines.push(empty_line);
    }

    lines
}
