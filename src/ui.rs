use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Row, Table, List, ListItem, Paragraph, Tabs, Gauge, Chart, Axis, Dataset, GraphType, BarChart
    },
    symbols,
    Frame,
};

use crate::app::{App, FileStatus, AppTab, AppView};

pub fn draw(f: &mut Frame, app: &mut App) {
    match app.view {
        AppView::Home => draw_home(f),
        AppView::Dashboard => draw_dashboard(f, app),
    }
}

fn draw_home(f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(35), // Logo (More space)
                Constraint::Percentage(35), // Menu
                Constraint::Percentage(30), // Footer
            ]
            .as_ref(),
        )
        .split(f.area());

    // 1. ASCII Art Logo (Professional Slant Style)
    // Font: ANSI Shadow / Slant
    let logo_text = vec![
        "██████╗ ██╗██████╗ ███████╗██████╗ ",
        "██╔══██╗██║██╔══██╗██╔════╝██╔══██╗",
        "██████╔╝██║██████╔╝█████╗  ██████╔╝",
        "██╔═══╝ ██║██╔═══╝ ██╔══╝  ██╔══██╗",
        "██║     ██║██║     ███████╗██║  ██║",
        "╚═╝     ╚═╝╚═╝     ╚══════╝╚═╝  ╚═╝",
        "",
        "The Middle-Out Data Optimizer",
    ];
    
    let logo_alignment = Paragraph::new(logo_text.join("\n"))
        .alignment(Alignment::Center) // Center logic
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
    
    // We render directly to the chunk, relying on Alignment::Center
    f.render_widget(logo_alignment, chunks[0]);

    // 2. Menu
    // Centered menu items
    let menu_text = vec![
        Line::from(vec![
            Span::styled("1. Scan      ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
             Span::raw(" Find compressible artifacts"),
        ]),
        Line::from(""), // Spacer
        Line::from(vec![
            Span::styled("2. Analytics ", Style::default().fg(Color::White)),
             Span::raw(" Visualize storage efficiency"),
        ]),
        Line::from(""),
        Line::from(vec![
             Span::styled("3. Status    ", Style::default().fg(Color::White)),
            Span::raw(" System health monitor"),
        ]),
        Line::from(vec![
             Span::styled("Q. Quit      ", Style::default().fg(Color::Red)),
            Span::raw(" Exit Application"),
        ]),
    ];
    
    let menu_p = Paragraph::new(menu_text)
        .alignment(Alignment::Center); // Center the text within the paragraph
    
    // To vertically center the menu, we can use a helper or just render to the middle chunk
    f.render_widget(menu_p, chunks[1]);

    // 3. Footer
    let footer_text = " [1-3] Select | [Q] Quit ";
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[2]);
}

fn draw_dashboard(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1), // Minimal Header
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Main Content
                Constraint::Length(1), // Footer/Status Bar
            ]
            .as_ref(),
        )
        .split(f.area());

    draw_minimal_header(f, app, chunks[0]);
    draw_tabs(f, app, chunks[1]);
    
    match app.current_tab {
        AppTab::Scanner => draw_file_list(f, app, chunks[2]),
        AppTab::Analytics => draw_analytics(f, app, chunks[2]),
        AppTab::Status => draw_status(f, app, chunks[2]),
    }
    
    draw_footer(f, app, chunks[3]);

    if app.show_details {
        draw_details_popup(f, app);
    }
}

fn draw_minimal_header(f: &mut Frame, app: &App, area: Rect) {
    let score = app.weissman_score;
    let label = format!(" PIPER v1.0 | Weissman Score: {:.2} ", score);
    let p = Paragraph::new(label)
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD).bg(Color::Black));
    f.render_widget(p, area);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec![" Scanner ", " Analytics ", " Status "];
    let tabs = Tabs::new(titles)
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        .divider(" | ")
        .select(match app.current_tab {
            AppTab::Scanner => 0,
            AppTab::Analytics => 1,
            AppTab::Status => 2,
        });
    f.render_widget(tabs, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
             Constraint::Length(3), // CPU
             Constraint::Length(3), // RAM
             Constraint::Min(0),    // Details/Other
        ].as_ref())
        .margin(1)
        .split(area);
        
    // CPU Gauge
    let cpu_gauge = Gauge::default()
        .block(Block::default().title(format!(" CPU Usage: {:.1}% ", app.cpu_usage)).borders(Borders::ALL))
        .gauge_style(Style::default().fg(if app.cpu_usage > 80.0 { Color::Red } else { Color::Green }))
        .percent(app.cpu_usage as u16);
    f.render_widget(cpu_gauge, chunks[0]);
    
    // RAM Gauge
    let mem_pct = (app.mem_usage as f64 / app.total_mem as f64) * 100.0;
    let mem_gauge = Gauge::default()
        .block(Block::default().title(format!(" Memory Usage: {:.1}% ({}/{}) ", mem_pct, format_size(app.mem_usage), format_size(app.total_mem))).borders(Borders::ALL))
        .gauge_style(Style::default().fg(if mem_pct > 80.0 { Color::Red } else { Color::Cyan }))
        .percent(mem_pct as u16);
    f.render_widget(mem_gauge, chunks[1]);
    
    let info_text = Paragraph::new("\n   System Monitor Active.\n   Real-time metrics provided by `sysinfo`.")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(info_text, chunks[2]);
}
fn draw_analytics(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Summary stats
            Constraint::Min(0),    // Chart
        ].as_ref())
        .margin(1)
        .split(area);
        
    // 1. Summary
    let total_sessions = app.history.entries.len();
    let all_time_savings: u64 = app.history.entries.iter().map(|e| e.savings).sum();
    
    let summary_text = format!(" All-Time Savings: {} | Total Sessions: {}", format_size(all_time_savings), total_sessions);
    let summary = Paragraph::new(summary_text)
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title(" Overview "));
    f.render_widget(summary, chunks[0]);
    
    // 2. Chart (Recent Activity)
    // We take the last 10 entries
    let take_last = 10;
    let start_idx = app.history.entries.len().saturating_sub(take_last);
    let recent_entries = &app.history.entries[start_idx..];
    
    if recent_entries.is_empty() {
        let no_data = Paragraph::new("\n   No compression history yet.\n   Start compressing files to see trends here.")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(no_data, chunks[1]);
        return;
    }
    
    // Prepare data for BarChart
    // BarChart expects u64, but our sizes can be huge (GBs). We should conceptually normalize to MB for the chart height?
    // Ratatui BarChart handles scaling automatically? No, it just draws bars.
    // If values are 100,000,000 bytes, bars might be huge or clipped?
    // We should scale to "MB".
    
    let data_points: Vec<(String, u64)> = recent_entries.iter().enumerate().map(|(i, e)| {
        let label = format!("#{}", start_idx + i + 1); // Simple label #1, #2...
        // Convert to MB for readability in values
        let mb = e.savings / (1024 * 1024); 
        (label, mb) 
    }).collect();
    
    let bar_data: Vec<(&str, u64)> = data_points.iter().map(|(l, v)| (l.as_str(), *v)).collect();
    
    let chart = ratatui::widgets::BarChart::default()
        .block(Block::default().title(" Savings Trend (MB) ").borders(Borders::ALL))
        .data(&bar_data)
        .bar_width(8)
        .bar_gap(2)
        .bar_style(Style::default().fg(Color::Cyan))
        .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
        
    f.render_widget(chart, chunks[1]);
}




fn draw_file_list(f: &mut Frame, app: &mut App, area: Rect) {
    if app.items.is_empty() && !app.is_scanning {
         let text = Paragraph::new("\n   No artifacts found. Press [S] to Scan.")
            .style(Style::default().fg(Color::DarkGray));
         f.render_widget(text, area);
         return;
    }

    if app.is_scanning {
        let spinner = match app.spinner_state {
             0 => "⠋", 1 => "⠙", 2 => "⠹", 3 => "⠸", _ => "⠼",
         };
        let text = format!("\n   {} Scanning directory...", spinner);
        let p = Paragraph::new(text).style(Style::default().fg(Color::Yellow));
        f.render_widget(p, area);
        return;
    } 
    
    if app.is_compressing {
         let spinner = match app.spinner_state {
             0 => "⠋", 1 => "⠙", 2 => "⠹", 3 => "⠸", _ => "⠼",
         };
         let text = format!("\n   {} Compressing artifacts... Please wait.", spinner);
         let p = Paragraph::new(text).style(Style::default().fg(Color::Cyan));
        f.render_widget(p, area);
        return;
    }

    let rows: Vec<Row> = app.items.iter().map(|i| {
        let status_icon = match i.status {
            FileStatus::Found => "📦",
            FileStatus::Compressing => "🔄",
            FileStatus::Done => "✅",
            FileStatus::Error => "❌",
            FileStatus::Deleted => "🗑️ ",
            FileStatus::Restored => "↩ ",
        };

        let style = if i.status == FileStatus::Deleted {
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT)
        } else if i.status == FileStatus::Done {
            Style::default().fg(Color::Green)
        } else if i.status == FileStatus::Error {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::White)
        };

        let reason_style = Style::default().fg(Color::DarkGray); // Muted reason

        let size_str = if let Some(comp) = i.compressed_size {
             format!("{} -> {}", format_size(i.original_size), format_size(comp))
        } else if i.status == FileStatus::Deleted {
            format!("{} -> 0", format_size(i.original_size))
        } else {
            format_size(i.original_size)
        };

        let check = if i.selected { " [x]" } else { " [ ]" };
        let path_str = format!("{}{}", check, i.path);

        Row::new(vec![
            Cell::from(status_icon),
            Cell::from(path_str).style(style),
            Cell::from(i.reason.clone()).style(reason_style),
            Cell::from(size_str).style(Style::default().fg(Color::Cyan)),
        ])
    }).collect();

    let table = Table::new(rows, [
            Constraint::Length(3),
            Constraint::Percentage(50), 
            Constraint::Percentage(25), 
            Constraint::Percentage(22)
        ])
        .header(
            Row::new(vec!["", " Artifact", " Type", " Size"])
                .style(Style::default().fg(Color::DarkGray))
                .bottom_margin(1)
        )
        // No borders for cleaner look
        .highlight_symbol(" > ");

    f.render_stateful_widget(table, area, &mut app.list_state);
}

fn draw_footer(f: &mut Frame, _app: &App, area: Rect) {
    // Minimal status line, vim-like
    let instructions = Paragraph::new(" NORMAL MODE | [S]can [C]ompress [D]elete [E]restore [Q]uit [Space]Select")
        .style(Style::default().fg(Color::Black).bg(Color::Cyan));
    f.render_widget(instructions, area);
}

fn draw_details_popup(f: &mut Frame, app: &App) {
    if let Some(i) = app.list_state.selected() {
        if i >= app.items.len() { return; }
        
        let item = &app.items[i];
        
        let block = Block::default().title(" Details ").borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let area = centered_rect(60, 40, f.area());
        
        f.render_widget(ratatui::widgets::Clear, area); // Clear background
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(1), // Path
                    Constraint::Length(1), // Sort Reason
                    Constraint::Length(1), // Original
                    Constraint::Length(1), // Compressed
                    Constraint::Length(1), // Spacer
                    Constraint::Length(1), // Savings
                ]
                .as_ref(),
            )
            .split(area);

        f.render_widget(Paragraph::new(format!("Path: {}", item.path)).style(Style::default().fg(Color::Yellow)), chunks[0]);
        f.render_widget(Paragraph::new(format!("Type: {}", item.reason)).style(Style::default().fg(Color::DarkGray)), chunks[1]);
        f.render_widget(Paragraph::new(format!("Original:   {}", format_size(item.original_size))), chunks[2]);
        
        let compressed_str = if let Some(s) = item.compressed_size {
            format!("{}", format_size(s))
        } else {
            "Pending".to_string()
        };
        f.render_widget(Paragraph::new(format!("Compressed: {}", compressed_str)), chunks[3]);

        let savings = if item.status == FileStatus::Error {
            "Savings:    Failed (Incompressible)".to_string()
        } else if let Some(s) = item.compressed_size {
            if item.original_size > s {
                let diff = item.original_size - s;
                let pct = (diff as f64 / item.original_size as f64) * 100.0;
                format!("Savings:    {} ({:.1}%)", format_size(diff), pct)
            } else {
                 "Savings:    None".to_string()
            }
        } else {
             "Savings:    Pending...".to_string()
        };
        f.render_widget(Paragraph::new(savings).style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)), chunks[5]);
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

/// Helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
