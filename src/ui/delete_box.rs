use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect, box_id: i32) {
    // Determine box name
    let box_name = app.boxes.iter()
        .find(|b| b.id == box_id)
        .map(|b| b.title.clone())
        .unwrap_or_else(|| String::from("Unknown Box"));

    // Calculate center position for modal
    let modal_width = std::cmp::min(50, area.width.saturating_sub(4));
    let modal_height = 8;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    
    let modal_area = Rect {
        x,
        y,
        width: modal_width,
        height: modal_height,
    };
    
    // Clear background
    f.render_widget(Clear, modal_area);
    
    // Create warning block
    let block = Block::default()
        .title("⚠️  Delete Box")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    
    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);
    
    // Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),  // Message
            Constraint::Length(2),  // Confirmation
        ])
        .split(inner);
    
    // Warning message
    let message = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Are you sure you want to delete "),
            Span::styled(box_name, Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("?"),
        ]),
        Line::from("This action cannot be undone."),
    ])
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::White));
    
    f.render_widget(message, chunks[0]);
    
    // Controls
    let controls = Paragraph::new(Line::from(vec![
        Span::styled("y", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(": Confirm | "),
        Span::styled("n / Esc", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(": Cancel"),
    ]))
    .alignment(Alignment::Center);
    
    f.render_widget(controls, chunks[1]);
}
