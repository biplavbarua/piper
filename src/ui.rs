use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, FileStatus};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main List
                Constraint::Length(3), // Footer
            ]
            .as_ref(),
        )
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_file_list(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    if app.show_details {
        draw_details_popup(f, app);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(" PIPER | Weissman Score: {:.2} ", app.weissman_score);
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));
    let paragraph = Paragraph::new(title).block(block).style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(paragraph, area);
}

fn draw_file_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|i| {
            let status_icon = match i.status {
                FileStatus::Found => "📦",
                FileStatus::Compressing => "🔄",
                FileStatus::Done => "✅",
                FileStatus::Error => "❌",
                FileStatus::Deleted => "🗑️ ",
            };
            
            let size_info = if let Some(comp) = i.compressed_size {
                 format!("{} -> {}", i.original_size, comp)
            } else if i.status == FileStatus::Deleted {
                format!("{} -> 0 (Deleted)", i.original_size)
            } else {
                format!("{}", i.original_size)
            };

            let content = vec![Line::from(vec![
                Span::raw(format!("{}  ", status_icon)),
                Span::styled(format!("{:<50}", i.path), Style::default().fg(Color::Yellow)),
                Span::raw(format!("  [{}]", size_info)),
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = if app.is_scanning {
         let spinner = match app.spinner_state {
             0 => "⠋",
             1 => "⠙",
             2 => "⠹",
             3 => "⠸",
             _ => "⠼",
         };
         let items = vec![ListItem::new(Line::from(vec![
             Span::styled(format!(" {} Scanning for artifacts...", spinner), Style::default().fg(Color::Yellow))
         ]))];
         List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Status "))
    } else if app.is_compressing {
         let spinner = match app.spinner_state {
             0 => "⠋",
             1 => "⠙",
             2 => "⠹",
             3 => "⠸",
             _ => "⠼",
         };
         List::new(items)
            .block(Block::default().borders(Borders::ALL).title(format!(" {} Compressing Artifacts... ", spinner)))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
            .highlight_symbol(">> ")
    } else {
         List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Compressible Artifacts "))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
            .highlight_symbol(">> ")
    };

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_footer(f: &mut Frame, _app: &App, area: Rect) {
    let instructions = Paragraph::new(" [Q] Quit | [S] Scan | [C] Compress | [D] Delete | [J/K] Navigate | [Enter] Details ")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, area);
}

fn draw_details_popup(f: &mut Frame, app: &App) {
    if let Some(i) = app.list_state.selected() {
        if i >= app.items.len() { return; }
        
        let item = &app.items[i];
        
        let block = Block::default().title(" File Details ").borders(Borders::ALL);
        let area = centered_rect(60, 40, f.area());
        
        f.render_widget(ratatui::widgets::Clear, area); // Clear background
        f.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(1), // Path
                    Constraint::Length(1), // Spacer
                    Constraint::Length(1), // Original
                    Constraint::Length(1), // Compressed
                    Constraint::Length(1), // Spacer
                    Constraint::Length(1), // Savings
                ]
                .as_ref(),
            )
            .split(area);

        f.render_widget(Paragraph::new(format!("Path: {}", item.path)).style(Style::default().fg(Color::Yellow)), chunks[0]);
        
        f.render_widget(Paragraph::new(format!("Original Size:   {} bytes", item.original_size)), chunks[2]);
        
        let compressed_str = if let Some(s) = item.compressed_size {
            format!("{} bytes", s)
        } else {
            "N/A".to_string()
        };
        f.render_widget(Paragraph::new(format!("Compressed Size: {}", compressed_str)), chunks[3]);

        let savings = if let Some(s) = item.compressed_size {
            if item.original_size > s {
                let diff = item.original_size - s;
                let pct = (diff as f64 / item.original_size as f64) * 100.0;
                format!("Savings:         {} bytes ({:.2}%)", diff, pct)
            } else {
                "Savings:         0 bytes (0.00%)".to_string()
            }
        } else {
             "Savings:         Pending...".to_string()
        };
        f.render_widget(Paragraph::new(savings).style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)), chunks[5]);
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
