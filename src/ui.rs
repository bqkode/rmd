use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::app::{App, AppMode, Focus, Theme};
use crate::markdown::{RenderedLine, TextSegment};

// Theme color definitions
struct ThemeColors {
    foreground: Color,
    background: Color,
    comment: Color,
    heading1: Color,
    heading2: Color,
    heading3: Color,
    heading4: Color,
    heading5: Color,
    heading6: Color,
    code: Color,
    code_bg: Color,
    link: Color,
    table: Color,
    highlight_fg: Color,
    highlight_bg: Color,
}

fn get_theme_colors(theme: Theme) -> ThemeColors {
    match theme {
        Theme::Dark => ThemeColors {
            foreground: Color::Rgb(248, 248, 242),    // Monokai foreground
            background: Color::Rgb(39, 40, 34),       // Monokai background
            comment: Color::Rgb(117, 113, 94),        // Monokai comment
            heading1: Color::Rgb(253, 151, 31),       // Monokai orange
            heading2: Color::Rgb(166, 226, 46),       // Monokai green
            heading3: Color::Rgb(230, 219, 116),      // Monokai yellow
            heading4: Color::Rgb(174, 129, 255),      // Monokai purple
            heading5: Color::Rgb(102, 217, 239),      // Monokai cyan
            heading6: Color::Rgb(117, 113, 94),       // Monokai comment
            code: Color::Rgb(166, 226, 46),           // Monokai green
            code_bg: Color::Rgb(39, 40, 34),          // Monokai background
            link: Color::Rgb(102, 217, 239),          // Monokai cyan
            table: Color::Rgb(102, 217, 239),         // Monokai cyan
            highlight_fg: Color::Rgb(102, 217, 239),  // Cyan
            highlight_bg: Color::Rgb(39, 40, 34),     // Dark background
        },
        Theme::Light => ThemeColors {
            // GitHub Light theme inspired colors
            foreground: Color::Rgb(36, 41, 46),       // GitHub dark text
            background: Color::Rgb(255, 255, 255),    // White
            comment: Color::Rgb(106, 115, 125),       // GitHub comment gray
            heading1: Color::Rgb(215, 58, 73),        // GitHub red/pink
            heading2: Color::Rgb(34, 134, 58),        // GitHub green
            heading3: Color::Rgb(111, 66, 193),       // GitHub purple
            heading4: Color::Rgb(0, 92, 197),         // GitHub blue
            heading5: Color::Rgb(227, 98, 9),         // GitHub orange
            heading6: Color::Rgb(106, 115, 125),      // GitHub comment gray
            code: Color::Rgb(0, 92, 197),             // GitHub blue
            code_bg: Color::Rgb(246, 248, 250),       // GitHub light gray bg
            link: Color::Rgb(3, 102, 214),            // GitHub link blue
            table: Color::Rgb(0, 92, 197),            // GitHub blue
            highlight_fg: Color::Rgb(36, 41, 46),     // Dark text
            highlight_bg: Color::Rgb(255, 251, 221),  // GitHub yellow highlight
        },
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    // Split into main area and bottom bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(f.area());

    // In select mode, show only content (full width for clean text selection)
    if app.mode == AppMode::Select {
        app.set_content_height(main_chunks[0].height);
        draw_content(f, app, main_chunks[0]);
        draw_status_bar(f, app, main_chunks[1]);
        return;
    }

    // Split main area into sidebar and content
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(main_chunks[0]);

    app.set_content_height(chunks[1].height);

    draw_sidebar(f, app, chunks[0]);
    draw_content(f, app, chunks[1]);
    draw_status_bar(f, app, main_chunks[1]);

    // Draw search overlay if in search mode
    if app.mode == AppMode::Search {
        draw_search_overlay(f, app, f.area());
    }

    // Draw settings overlay if in settings mode
    if app.mode == AppMode::Settings {
        draw_settings_overlay(f, app, f.area());
    }

    // Draw about overlay if in about mode
    if app.mode == AppMode::About {
        draw_about_overlay(f, f.area());
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.mode == AppMode::Select {
        // Show select mode help
        Line::from(vec![
            Span::styled(" SELECT MODE ", Style::default().fg(Color::Black).bg(Color::Rgb(253, 151, 31))),
            Span::raw(" Use mouse to select text, then copy with terminal shortcut (Cmd+C / Ctrl+Shift+C)  "),
            Span::styled(" v/Esc ", Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(" Exit "),
        ])
    } else {
        Line::from(vec![
            Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(" Quit  "),
            Span::styled(" hjkl ", Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(" Nav  "),
            Span::styled(" gg/G ", Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(" Top/Bot  "),
            Span::styled(" ^u/^d ", Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(" Half  "),
            Span::styled(" ^b/^f ", Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(" Full  "),
            Span::styled(" / ", Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(" Find  "),
            Span::styled(" ^s ", Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(" Search  "),
            Span::styled(" v ", Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(" Select  "),
            Span::styled(" ^p ", Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(" Settings  "),
            Span::styled(" ? ", Style::default().fg(Color::Black).bg(Color::White)),
            Span::raw(" About "),
        ])
    };

    let paragraph = Paragraph::new(help_text)
        .style(Style::default().bg(Color::DarkGray));

    f.render_widget(paragraph, area);
}

fn draw_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let items = app.visible_items();

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(idx, node)| {
            let indent = "  ".repeat(node.depth);
            let icon = if node.is_dir {
                if node.expanded {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };

            let style = if idx == app.selected_index {
                Style::default()
                    .bg(if app.focus == Focus::Sidebar {
                        Color::Blue
                    } else {
                        Color::DarkGray
                    })
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if node.is_dir {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            let content = format!("{}{}{}", indent, icon, node.name);
            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();

    let border_style = if app.focus == Focus::Sidebar {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Files "),
        );

    f.render_widget(list, area);
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    // If in document search mode, split area for search bar
    let (search_area, content_area) = if app.mode == AppMode::DocumentSearch {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(area);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, area)
    };

    // Draw search bar if in document search mode
    if let Some(search_rect) = search_area {
        let match_info = if app.doc_search_matches.is_empty() {
            if app.doc_search_query.is_empty() {
                String::new()
            } else {
                " (0 matches)".to_string()
            }
        } else {
            format!(" ({}/{})", app.doc_search_current + 1, app.doc_search_matches.len())
        };

        let search_input = Paragraph::new(Line::from(vec![
            Span::raw(&app.doc_search_query),
            Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            Span::styled(&match_info, Style::default().fg(Color::Rgb(117, 113, 94))),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(253, 151, 31)))
                .title(" Find (Enter/^n: next, ^p: prev, Esc: close) "),
        )
        .style(Style::default().fg(Color::White));

        f.render_widget(search_input, search_rect);
    }

    let border_style = if app.focus == Focus::Content {
        Style::default().fg(Color::Blue)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if let Some(ref path) = app.current_file {
        format!(" {} ", path.file_name().unwrap_or_default().to_string_lossy())
    } else {
        " Content ".to_string()
    };

    let area = content_area;

    // Get theme colors
    let colors = get_theme_colors(app.settings.theme);

    // Wrap lines at max width and track source line indices
    let max_width = app.settings.wrap_width.to_usize();
    let mut wrapped_lines: Vec<(Line, usize, bool)> = Vec::new(); // (line, source_idx, is_first)

    for (source_idx, line) in app.rendered_content.iter().enumerate() {
        let wrapped = wrap_line(line, max_width, &colors);
        for (i, wrapped_line) in wrapped.into_iter().enumerate() {
            wrapped_lines.push((wrapped_line, source_idx, i == 0));
        }
    }

    // Check if we need to highlight search matches
    let search_query = if app.mode == AppMode::DocumentSearch && !app.doc_search_query.is_empty() {
        Some(app.doc_search_query.to_lowercase())
    } else {
        None
    };

    // Disable line numbers in select mode for clean text selection
    let show_line_numbers = app.settings.show_line_numbers && app.mode != AppMode::Select;

    let lines: Vec<Line> = if show_line_numbers {
        let total_source_lines = app.rendered_content.len();
        let width = total_source_lines.to_string().len();

        wrapped_lines
            .into_iter()
            .enumerate()
            .skip(app.content_scroll)
            .take(area.height.saturating_sub(2) as usize)
            .map(|(_wrapped_idx, (line, source_idx, is_first))| {
                let is_match = app.doc_search_matches.contains(&source_idx);

                let num_style = if is_match {
                    Style::default().fg(colors.highlight_fg)
                } else {
                    Style::default().fg(colors.comment)
                };

                // Only show line number for first line of wrapped sequence
                let num_span = if is_first {
                    Span::styled(
                        format!("{:>width$} │ ", source_idx + 1, width = width),
                        num_style,
                    )
                } else {
                    Span::styled(
                        format!("{:>width$} │ ", "", width = width),
                        num_style,
                    )
                };

                let mut spans = vec![num_span];
                if let Some(ref query) = search_query {
                    spans.extend(highlight_matches(line.spans, query, &colors));
                } else {
                    spans.extend(line.spans);
                }
                Line::from(spans)
            })
            .collect()
    } else {
        wrapped_lines
            .into_iter()
            .skip(app.content_scroll)
            .take(area.height.saturating_sub(2) as usize)
            .map(|(line, _source_idx, _is_first)| {
                if let Some(ref query) = search_query {
                    Line::from(highlight_matches(line.spans, query, &colors))
                } else {
                    line
                }
            })
            .collect()
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(title),
        )
        .style(Style::default().bg(colors.background));

    f.render_widget(paragraph, area);

    // Calculate total wrapped lines for scrollbar
    let total_wrapped_lines = app.rendered_content.iter()
        .map(|line| wrap_line(line, app.settings.wrap_width.to_usize(), &colors).len())
        .sum::<usize>();

    // Draw scrollbar if content is scrollable
    if total_wrapped_lines > area.height.saturating_sub(2) as usize {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = ScrollbarState::new(
            total_wrapped_lines.saturating_sub(area.height.saturating_sub(2) as usize)
        )
        .position(app.content_scroll);

        let scrollbar_area = Rect {
            x: area.x + area.width - 1,
            y: area.y + 1,
            width: 1,
            height: area.height - 2,
        };

        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

fn wrap_line(line: &RenderedLine, max_width: Option<usize>, colors: &ThemeColors) -> Vec<Line<'static>> {
    if line.segments.is_empty() {
        return vec![Line::from("")];
    }

    // Don't wrap table rows or separators
    if line.is_table_row || line.is_table_separator {
        return vec![render_line_with_theme(line, colors)];
    }

    // If no wrapping, just render
    let max_width = match max_width {
        Some(w) => w,
        None => return vec![render_line_with_theme(line, colors)],
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

    // If line fits, just render normally
    if full_text.len() <= max_width {
        return vec![render_line_with_theme(line, colors)];
    }

    // For long lines, we need to wrap
    // Simple word-wrap implementation
    let mut result = Vec::new();
    let mut current_line = String::new();

    for word in full_text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            // Create a simple line with the base style
            let style = get_line_style(line, colors);
            result.push(Line::from(Span::styled(current_line.clone(), style)));
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        let style = get_line_style(line, colors);
        result.push(Line::from(Span::styled(current_line, style)));
    }

    if result.is_empty() {
        result.push(Line::from(""));
    }

    result
}

fn get_line_style(line: &RenderedLine, colors: &ThemeColors) -> Style {
    if line.heading_level > 0 {
        get_heading_style(line.heading_level, colors)
    } else if line.is_code_block {
        Style::default().fg(colors.code)
    } else if line.is_blockquote {
        Style::default().fg(colors.comment)
    } else if line.is_horizontal_rule {
        Style::default().fg(colors.comment)
    } else if line.is_table_row || line.is_table_separator {
        Style::default().fg(colors.table)
    } else {
        Style::default().fg(colors.foreground)
    }
}

fn render_line_with_theme(line: &RenderedLine, colors: &ThemeColors) -> Line<'static> {
    if line.segments.is_empty() {
        return Line::from("");
    }

    let mut spans = Vec::new();

    // Determine base style based on line type
    let base_style = get_line_style(line, colors);

    for segment in &line.segments {
        match segment {
            TextSegment::Plain(text) => {
                spans.push(Span::styled(text.clone(), base_style));
            }
            TextSegment::Code(text) => {
                spans.push(Span::styled(
                    format!("`{}`", text),
                    Style::default()
                        .fg(colors.heading1) // Use heading1 color for inline code
                        .bg(colors.code_bg),
                ));
            }
            TextSegment::Link { text, .. } => {
                spans.push(Span::styled(
                    text.clone(),
                    Style::default()
                        .fg(colors.link),
                ));
            }
            TextSegment::Emphasis(text) => {
                spans.push(Span::styled(
                    text.clone(),
                    base_style.add_modifier(Modifier::ITALIC),
                ));
            }
            TextSegment::Strong(text) => {
                spans.push(Span::styled(
                    text.clone(),
                    base_style.add_modifier(Modifier::BOLD),
                ));
            }
        }
    }

    Line::from(spans)
}


fn draw_settings_overlay(f: &mut Frame, app: &App, area: Rect) {
    // Calculate overlay size (centered, 50% width, 40% height)
    let overlay_width = (area.width as f32 * 0.5) as u16;
    let overlay_height = (area.height as f32 * 0.4) as u16;
    let overlay_x = (area.width - overlay_width) / 2;
    let overlay_y = (area.height - overlay_height) / 2;

    let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

    // Clear the area
    f.render_widget(Clear, overlay_area);

    // Build settings items
    let mut items: Vec<ListItem> = Vec::new();

    // Setting 0: Show line numbers (toggle)
    let checkbox = if app.settings.show_line_numbers { "[x]" } else { "[ ]" };
    let style = if app.settings_selected == 0 {
        Style::default().bg(Color::Rgb(102, 217, 239)).fg(Color::Black)
    } else {
        Style::default().fg(Color::White)
    };
    items.push(ListItem::new(Line::from(Span::styled(
        format!("{} Show line numbers", checkbox),
        style,
    ))));

    // Setting 1: Theme (cycle)
    let theme_name = match app.settings.theme {
        Theme::Dark => "Dark",
        Theme::Light => "Light",
    };
    let style = if app.settings_selected == 1 {
        Style::default().bg(Color::Rgb(102, 217, 239)).fg(Color::Black)
    } else {
        Style::default().fg(Color::White)
    };
    items.push(ListItem::new(Line::from(Span::styled(
        format!("    Theme: {}", theme_name),
        style,
    ))));

    // Setting 2: Wrap width (cycle)
    let style = if app.settings_selected == 2 {
        Style::default().bg(Color::Rgb(102, 217, 239)).fg(Color::Black)
    } else {
        Style::default().fg(Color::White)
    };
    items.push(ListItem::new(Line::from(Span::styled(
        format!("    Wrap width: {}", app.settings.wrap_width.display_name()),
        style,
    ))));

    let settings_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(102, 217, 239)))
            .title(" Settings (Enter to toggle, Esc to close) "),
    );

    f.render_widget(settings_list, overlay_area);
}

fn draw_search_overlay(f: &mut Frame, app: &App, area: Rect) {
    // Calculate overlay size (centered, 60% width, 50% height)
    let overlay_width = (area.width as f32 * 0.6) as u16;
    let overlay_height = (area.height as f32 * 0.5) as u16;
    let overlay_x = (area.width - overlay_width) / 2;
    let overlay_y = (area.height - overlay_height) / 2;

    let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

    // Clear the area
    f.render_widget(Clear, overlay_area);

    // Split into search input and results
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(overlay_area);

    // Search input
    let search_input = Paragraph::new(Line::from(vec![
        Span::raw(&app.search_query),
        Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(102, 217, 239)))
            .title(" Search "),
    )
    .style(Style::default().fg(Color::White));

    f.render_widget(search_input, chunks[0]);

    // Search results
    let results: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(idx, result)| {
            let is_selected = idx == app.search_selected;

            let name_style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(102, 217, 239))
                    .fg(Color::Rgb(180, 100, 0)) // Darker orange for contrast on cyan
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Rgb(253, 151, 31)) // Monokai orange
                    .add_modifier(Modifier::BOLD)
            };

            let preview_style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(102, 217, 239))
                    .fg(Color::Black)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::styled(&result.name, name_style),
                Span::styled(": ", preview_style),
                Span::styled(&result.match_preview, preview_style),
            ]))
        })
        .collect();

    let results_list = List::new(results).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(format!(" Results ({}) ", app.search_results.len())),
    );

    f.render_widget(results_list, chunks[1]);
}

fn highlight_matches(spans: Vec<Span<'static>>, query: &str, colors: &ThemeColors) -> Vec<Span<'static>> {
    let mut result = Vec::new();
    let highlight_style = Style::default()
        .fg(colors.highlight_fg)
        .bg(colors.highlight_bg);

    for span in spans {
        let text = span.content.to_string();
        let text_lower = text.to_lowercase();
        let style = span.style;

        if text_lower.contains(query) {
            let mut current_text = text.as_str();
            let mut current_lower = text_lower.as_str();

            while let Some(match_start) = current_lower.find(query) {
                // Add text before match
                if match_start > 0 {
                    result.push(Span::styled(
                        current_text[..match_start].to_string(),
                        style,
                    ));
                }

                // Add highlighted match
                let match_end = match_start + query.len();
                result.push(Span::styled(
                    current_text[match_start..match_end].to_string(),
                    highlight_style,
                ));

                // Move past this match
                current_text = &current_text[match_end..];
                current_lower = &current_lower[match_end..];
            }

            // Add remaining text
            if !current_text.is_empty() {
                result.push(Span::styled(current_text.to_string(), style));
            }
        } else {
            result.push(span);
        }
    }

    result
}

fn draw_about_overlay(f: &mut Frame, area: Rect) {
    // Calculate overlay size (centered, 50% width, 40% height)
    let overlay_width = (area.width as f32 * 0.5) as u16;
    let overlay_height = (area.height as f32 * 0.4) as u16;
    let overlay_x = (area.width - overlay_width) / 2;
    let overlay_y = (area.height - overlay_height) / 2;

    let overlay_area = Rect::new(overlay_x, overlay_y, overlay_width, overlay_height);

    // Clear the area
    f.render_widget(Clear, overlay_area);

    // Get version from build-time environment variable
    let version = option_env!("RMD_VERSION").unwrap_or("v0.0.1");

    let about_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "rmd",
            Style::default()
                .fg(Color::Rgb(253, 151, 31))
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Version: {}", version),
            Style::default().fg(Color::Rgb(166, 226, 46)),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("Author: "),
            Span::styled(
                "Bjørn Quentin Kvamme",
                Style::default().fg(Color::Rgb(102, 217, 239)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("GitHub: "),
            Span::styled(
                "https://github.com/bqkode/rmd",
                Style::default().fg(Color::Rgb(102, 217, 239)),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "A terminal-based Markdown viewer",
            Style::default().fg(Color::Rgb(117, 113, 94)),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc or Enter to close",
            Style::default().fg(Color::Rgb(117, 113, 94)),
        )),
    ];

    let about_paragraph = Paragraph::new(about_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(253, 151, 31)))
                .title(" About "),
        )
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(about_paragraph, overlay_area);
}

fn get_heading_style(level: u8, colors: &ThemeColors) -> Style {
    match level {
        1 => Style::default()
            .fg(colors.heading1)
            .add_modifier(Modifier::BOLD),
        2 => Style::default()
            .fg(colors.heading2)
            .add_modifier(Modifier::BOLD),
        3 => Style::default()
            .fg(colors.heading3)
            .add_modifier(Modifier::BOLD),
        4 => Style::default()
            .fg(colors.heading4),
        5 => Style::default()
            .fg(colors.heading5),
        6 => Style::default()
            .fg(colors.heading6),
        _ => Style::default(),
    }
}
