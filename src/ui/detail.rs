use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect, box_id: i32) {
    // Find the box by ID
    let ctf_box = match app.boxes.iter().find(|b| b.id == box_id) {
        Some(b) => b,
        None => {
            let error = Paragraph::new("Box not found!")
                .style(Style::default().fg(Color::Red));
            f.render_widget(error, area);
            return;
        }
    };

    // Create layout: header, info, notes, actions
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(6),  // Info block
            Constraint::Min(5),     // Notes
            Constraint::Min(5),     // Actions
        ])
        .split(area);

    // Header with title
    let platform_icon = match ctf_box.platform.as_str() {
        "HTB" => "🔴",
        "picoCTF" => "🎯",
        "TryHackMe" => "🟢",
        _ => "📦",
    };
    
    let header = Paragraph::new(format!(
        "{} {} - {}",
        platform_icon, ctf_box.title, ctf_box.platform
    ))
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(header, chunks[0]);

    // Info block
    let tags_str = ctf_box.tags.join(", ");
    let info_text = vec![
        Line::from(vec![
            Span::styled("IP: ", Style::default().fg(Color::Yellow)),
            Span::raw(ctf_box.ip_address.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Tags: ", Style::default().fg(Color::Yellow)),
            Span::raw(tags_str),
        ]),
        Line::from(vec![
            Span::styled("Created: ", Style::default().fg(Color::Yellow)),
            Span::raw(ctf_box.created_date.format("%Y-%m-%d %H:%M").to_string()),
        ]),
        Line::from(vec![
            Span::styled("Updated: ", Style::default().fg(Color::Yellow)),
            Span::raw(ctf_box.updated_date.format("%Y-%m-%d %H:%M").to_string()),
        ]),
    ];
    
    let info = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("ℹ️  Info"))
        .wrap(Wrap { trim: true });
    
    f.render_widget(info, chunks[1]);

    // Notes section
    let note_items: Vec<ListItem> = ctf_box
        .notes
        .iter()
        .map(|note| {
            let category_icon = match note.category {
                crate::models::NoteCategory::Web => "🌐",
                crate::models::NoteCategory::Pwn => "💥",
                crate::models::NoteCategory::Crypto => "🔐",
                crate::models::NoteCategory::Reversing => "🔄",
                crate::models::NoteCategory::Stego => "🖼️",
                crate::models::NoteCategory::Recon => "🔍",
                crate::models::NoteCategory::Foothold => "🚪",
                crate::models::NoteCategory::Privesc => "⬆️",
                crate::models::NoteCategory::Misc => "📝",
            };
            
            ListItem::new(format!("{} {:?}: {}", category_icon, note.category, note.content))
        })
        .collect();
    
    let notes = List::new(note_items)
        .block(Block::default().borders(Borders::ALL).title("📝 Notes"));
    
    f.render_widget(notes, chunks[2]);

    // Actions section
    let action_items: Vec<ListItem> = ctf_box
        .actions
        .iter()
        .map(|action| {
            let result_icon = match action.result {
                crate::models::ActionResult::Success => "✅",
                crate::models::ActionResult::Fail => "❌",
                crate::models::ActionResult::Unknown => "❓",
            };
            
            let time = action.timestamp.format("%H:%M:%S");
            let note_suffix = action.note.as_ref()
                .map(|n| format!(" - {}", n))
                .unwrap_or_default();
            
            ListItem::new(format!(
                "{} [{}] {}{}",
                result_icon, time, action.command, note_suffix
            ))
        })
        .collect();
    
    let actions = List::new(action_items)
        .block(Block::default().borders(Borders::ALL).title("🔧 Actions"));
    
    f.render_widget(actions, chunks[3]);
}