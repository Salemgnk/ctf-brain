use crate::app::{App, NoteForm};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

fn category_icon(cat: &crate::models::NoteCategory) -> &'static str {
    match cat {
        crate::models::NoteCategory::Web => "🌐",
        crate::models::NoteCategory::Pwn => "💥",
        crate::models::NoteCategory::Crypto => "🔐",
        crate::models::NoteCategory::Reversing => "🔄",
        crate::models::NoteCategory::Stego => "🖼️",
        crate::models::NoteCategory::Recon => "🔍",
        crate::models::NoteCategory::Foothold => "🚪",
        crate::models::NoteCategory::Privesc => "⬆️",
        crate::models::NoteCategory::Misc => "📝",
    }
}

fn category_name(cat: &crate::models::NoteCategory) -> &'static str {
    match cat {
        crate::models::NoteCategory::Recon => "Recon",
        crate::models::NoteCategory::Foothold => "Foothold",
        crate::models::NoteCategory::Privesc => "Privesc",
        crate::models::NoteCategory::Web => "Web",
        crate::models::NoteCategory::Pwn => "Pwn",
        crate::models::NoteCategory::Crypto => "Crypto",
        crate::models::NoteCategory::Reversing => "Reversing",
        crate::models::NoteCategory::Stego => "Stego",
        crate::models::NoteCategory::Misc => "Misc",
    }
}

pub fn render(f: &mut Frame, app: &App, form: Option<&NoteForm>, area: Rect, box_id: i32) {
    let ctf_box = match app.boxes.iter().find(|b| b.id == box_id) {
        Some(b) => b,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Notes list
            Constraint::Length(7), // Add form or instructions
        ])
        .split(area);

    // Header
    let header = Paragraph::new(format!(
        "📝 Notes - {} ({} notes)",
        ctf_box.title,
        ctf_box.notes.len()
    ))
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Notes list with selection
    let note_items: Vec<ListItem> = ctf_box
        .notes
        .iter()
        .enumerate()
        .map(|(i, note)| {
            let is_selected = app.selected_note == Some(i) && form.is_none();
            let icon = category_icon(&note.category);
            let cat_name = category_name(&note.category);

            let style = if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            let time = note.created_date.format("%m/%d %H:%M");

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(
                    format!("{} {}", icon, cat_name),
                    style.fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" [{}] ", time), style.fg(Color::DarkGray)),
                Span::styled(&note.content, style.fg(Color::White)),
            ]))
        })
        .collect();

    let notes_list =
        List::new(note_items).block(Block::default().borders(Borders::ALL).title("📋 Notes"));
    f.render_widget(notes_list, chunks[1]);

    // Form or instructions
    if let Some(form) = form {
        let categories = App::note_categories();
        let current_cat = &categories[form.category_index];
        let cat_display = format!(
            "{} {}",
            category_icon(current_cat),
            category_name(current_cat)
        );

        let form_block = Block::default()
            .borders(Borders::ALL)
            .title("➕ Add Note")
            .border_style(Style::default().fg(Color::Green));

        let form_inner = form_block.inner(chunks[2]);
        f.render_widget(form_block, chunks[2]);

        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Category
                Constraint::Length(1), // Content
                Constraint::Length(1), // Help
            ])
            .split(form_inner);

        // Category selector
        let cat_line = Line::from(vec![
            Span::styled("CATEGORY: ", Style::default().fg(Color::Cyan)),
            Span::styled("◀ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &cat_display,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ▶", Style::default().fg(Color::DarkGray)),
        ]);
        f.render_widget(Paragraph::new(cat_line), form_chunks[0]);

        // Content field
        let content_line = Line::from(vec![
            Span::styled("CONTENT: ", Style::default().fg(Color::Cyan)),
            Span::raw(&form.content),
        ]);
        f.render_widget(Paragraph::new(content_line), form_chunks[1]);

        // Help
        let help = Line::from(vec![
            Span::styled("←/→", Style::default().fg(Color::Green)),
            Span::raw(": Category | "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": Add | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": Cancel"),
        ]);
        f.render_widget(
            Paragraph::new(help).alignment(Alignment::Center),
            form_chunks[2],
        );

        // Cursor position on content field
        f.set_cursor_position((
            form_chunks[1].x + 9 + form.content.len() as u16, // "CONTENT: " = 9 chars
            form_chunks[1].y,
        ));
    } else {
        let help = Paragraph::new(Line::from(vec![
            Span::styled(
                "a",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Add | "),
            Span::styled(
                "d",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Delete | "),
            Span::styled(
                "j/k",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Navigate | "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Back"),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Commands"));

        f.render_widget(help, chunks[2]);
    }
}
