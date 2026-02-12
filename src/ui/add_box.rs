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
    let modal_height = std::cmp::min(20, area.height.saturating_sub(2));
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
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Platform
            Constraint::Length(3),  // IP
            Constraint::Length(3),  // Tags
            Constraint::Length(2),  // Help text
        ])
        .split(inner);
    
    // Render each field with a single-line display
    render_field(f, chunks[0], "Title", &form.title, form.current_field == 0);
    render_field(f, chunks[1], "Platform", &form.platform, form.current_field == 1);
    render_field(f, chunks[2], "IP Address", &form.ip, form.current_field == 2);
    render_field(f, chunks[3], "Tags", &form.tags, form.current_field == 3);
    
    // Help text
    let help = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("/"),
            Span::styled("Shift+Tab", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(": Navigate | "),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(": Submit | "),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(": Cancel"),
        ]),
        Line::from(vec![
            Span::styled("Platform: ", Style::default().fg(Color::DarkGray)),
            Span::raw("HTB, picoCTF, TryHackMe, etc."),
        ]),
    ])
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
    
    // Position cursor at end of text in the input area
    // The text starts after "Label: " which is calculated dynamically
    let label = match form.current_field {
        0 => "Title: ",
        1 => "Platform: ",
        2 => "IP Address: ",
        3 => "Tags: ",
        _ => "",
    };
    
    f.set_cursor_position((
        active_chunk.x + 1 + label.len() as u16 + text.chars().count() as u16,
        active_chunk.y + 1
    ));
}

/// Helper function to render a single field
fn render_field(f: &mut Frame, area: Rect, label: &str, value: &str, is_active: bool) {
    let style = if is_active {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    
    // Display label and value on the same line
    let content = Line::from(vec![
        Span::styled(format!("{}: ", label), style),
        Span::styled(value, Style::default().fg(Color::Cyan)),
    ]);
    
    let field = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style));
    
    f.render_widget(field, area);
}