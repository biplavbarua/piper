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
    } else {
         List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Compressible Artifacts "))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
            .highlight_symbol(">> ")
    };

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_footer(f: &mut Frame, _app: &App, area: Rect) {
    let instructions = Paragraph::new(" [Q] Quit | [S] Scan | [C] Compress | [D] Delete | [J/K] Navigate ")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(instructions, area);
}
