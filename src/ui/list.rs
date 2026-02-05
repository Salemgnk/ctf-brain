use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Convert boxes to list items with icons and formatting
    let items: Vec<ListItem> = app
        .boxes
        .iter()
        .enumerate()
        .map(|(idx, ctf_box)| {
            let platform_icon = match ctf_box.platform.as_str() {
                "HTB" => "🔴",
                "picoCTF" => "🎯",
                "TryHackMe" => "🟢",
                _ => "📦",
            };
            
            let content = format!(
                "{} [{}] {} - {}",
                platform_icon,
                ctf_box.platform,
                ctf_box.title,
                ctf_box.ip_address
            );
            
            let style = if Some(idx) == app.selected_box_id.map(|id| id as usize) {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            ListItem::new(Line::from(content)).style(style)
        })
        .collect();

    // Create the list widget
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🧩 CTF Boxes")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    // Create state for highlighting
    let mut list_state = ListState::default();
    if let Some(selected) = app.selected_box_id {
        list_state.select(Some(selected as usize));
    }

    // Render the widget
    f.render_stateful_widget(list, area, &mut list_state);
}
