use crate::app::{App, EnvVarForm};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render(f: &mut Frame, app: &App, form: Option<&EnvVarForm>, area: Rect, box_id: i32) {
    // Find the box
    let ctf_box = match app.boxes.iter().find(|b| b.id == box_id) {
        Some(b) => b,
        None => return,
    };

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Variables list
            Constraint::Length(7), // Add form or instructions
        ])
        .split(area);

    // Header
    let var_count = ctf_box.env_vars.len();
    let header = Paragraph::new(format!(
        "🔧 Environment Variables - {} ({} vars)",
        ctf_box.title, var_count
    ))
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Variables list with selection
    let _keys: Vec<&String> = ctf_box.env_vars.keys().collect();
    let var_items: Vec<ListItem> = ctf_box
        .env_vars
        .iter()
        .enumerate()
        .map(|(i, (key, value))| {
            let display_value = if value.len() > 50 {
                format!("{}...", &value[..50])
            } else {
                value.clone()
            };

            let is_selected = app.selected_env_var == Some(i) && form.is_none();

            let style = if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            let prefix = if is_selected { "▶ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(key, style.fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" = ", style),
                Span::styled(display_value, style.fg(Color::White)),
            ]))
        })
        .collect();

    let var_list = List::new(var_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("📋 Current Variables"),
    );

    f.render_widget(var_list, chunks[1]);

    // Form or instructions
    if let Some(form) = form {
        // Add form active
        let form_block = Block::default()
            .borders(Borders::ALL)
            .title("➕ Add Variable")
            .border_style(Style::default().fg(Color::Green));

        let form_inner = form_block.inner(chunks[2]);
        f.render_widget(form_block, chunks[2]);

        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Key
                Constraint::Length(1), // Value
                Constraint::Length(1), // Help
            ])
            .split(form_inner);

        // Key field
        let key_style = if form.current_field == 0 {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let key_line = Line::from(vec![Span::styled("KEY: ", key_style), Span::raw(&form.key)]);
        f.render_widget(Paragraph::new(key_line), form_chunks[0]);

        // Value field
        let value_style = if form.current_field == 1 {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let value_line = Line::from(vec![
            Span::styled("VALUE: ", value_style),
            Span::raw(&form.value),
        ]);
        f.render_widget(Paragraph::new(value_line), form_chunks[1]);

        // Help
        let help = Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Green)),
            Span::raw(": Switch | "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(": Add | "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(": Cancel"),
        ]);
        f.render_widget(
            Paragraph::new(help).alignment(Alignment::Center),
            form_chunks[2],
        );

        // Cursor
        let (x_offset, text) = if form.current_field == 0 {
            (5, &form.key) // "KEY: " = 5 chars
        } else {
            (7, &form.value) // "VALUE: " = 7 chars
        };

        f.set_cursor_position((
            form_chunks[form.current_field].x + x_offset + text.len() as u16,
            form_chunks[form.current_field].y,
        ));
    } else {
        // No form, show instructions
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
