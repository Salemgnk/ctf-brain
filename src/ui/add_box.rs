use crate::app::{AddBoxForm, App};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, _app: &App, form: &AddBoxForm, area: Rect) {
    // Calculate center position for modal (make responsive to small terminals)
    let modal_width = std::cmp::min(60, area.width.saturating_sub(4));
    let modal_height = std::cmp::min(19, area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    
    let modal_area = Rect {
        x,
        y,
        width: modal_width,
        height: modal_height,
    };
    
    // Clear the background
    f.render_widget(Clear, modal_area);
    
    // Create modal block
    let block = Block::default()
        .title("➕ Add New CTF Box")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);
    
    // Split into fields
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4),  // Title
            Constraint::Length(4),  // Platform
            Constraint::Length(4),  // IP
            Constraint::Length(4),  // Tags
            Constraint::Length(1),  // Help text
        ])
        .split(inner);
    
    // Field 0: Title
    let title_style = if form.current_field == 0 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let title_field = Paragraph::new(vec![
        Line::from(Span::styled("Title:", title_style)),
        Line::from(Span::raw(&form.title)),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title_field, chunks[0]);
    
    // Field 1: Platform
    let platform_style = if form.current_field == 1 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let platform_field = Paragraph::new(vec![
        Line::from(Span::styled("Platform (HTB/picoCTF/TryHackMe):", platform_style)),
        Line::from(Span::raw(&form.platform)),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(platform_field, chunks[1]);
    
    // Field 2: IP
    let ip_style = if form.current_field == 2 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let ip_field = Paragraph::new(vec![
        Line::from(Span::styled("IP Address:", ip_style)),
        Line::from(Span::raw(&form.ip)),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(ip_field, chunks[2]);
    
    // Field 3: Tags
    let tags_style = if form.current_field == 3 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let tags_field = Paragraph::new(vec![
        Line::from(Span::styled("Tags (comma-separated):", tags_style)),
        Line::from(Span::raw(&form.tags)),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(tags_field, chunks[3]);
    
    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(": Next | "),
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(": Submit | "),
        Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(": Cancel"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(help, chunks[4]);

    // Set cursor to active field
    let (active_chunk, text) = match form.current_field {
        0 => (chunks[0], &form.title),
        1 => (chunks[1], &form.platform),
        2 => (chunks[2], &form.ip),
        3 => (chunks[3], &form.tags),
        _ => return,
    };
    
    // Position cursor at end of text
    // +1 for border X
    // +1 because text is on second line of the block (first is label)
    // For Y: active_chunk.y gives the top of the current block
    f.set_cursor_position((
        active_chunk.x + 1 + text.chars().count() as u16,
        active_chunk.y + 2
    ));
}
