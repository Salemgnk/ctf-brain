use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(f: &mut Frame, app: &App, area: Rect, box_id: i32) {
    let ctf_box = match app.boxes.iter().find(|b| b.id == box_id) {
        Some(b) => b,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(3),   // Info
            Constraint::Length(5), // Path input
        ])
        .split(area);

    // Header
    let header = Paragraph::new(format!(
        "📄 Export Write-up — {}",
        ctf_box.title
    ))
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Summary of what will be exported
    let action_count = ctf_box.actions.len();
    let note_count = ctf_box.notes.len();
    let actions_with_output = ctf_box.actions.iter().filter(|a| a.output.is_some()).count();

    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Actions: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} ({} with output)", action_count, actions_with_output)),
        ]),
        Line::from(vec![
            Span::styled("Notes: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", note_count)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Only sections with content will be included.",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("📊 Summary"),
    );
    f.render_widget(info, chunks[1]);

    // Path input
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("📁 Output Path")
        .border_style(Style::default().fg(Color::Green));

    let input_inner = input_block.inner(chunks[2]);
    f.render_widget(input_block, chunks[2]);

    let input_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Path field
            Constraint::Length(1), // Help
        ])
        .split(input_inner);

    let path_line = Line::from(vec![
        Span::styled(
            "File: ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(&app.writeup_path, Style::default().fg(Color::White)),
    ]);
    f.render_widget(Paragraph::new(path_line), input_chunks[0]);

    let help = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(": Export | "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(": Cancel"),
    ]);
    f.render_widget(
        Paragraph::new(help).alignment(Alignment::Center),
        input_chunks[1],
    );

    // Cursor at end of path
    f.set_cursor_position((
        input_chunks[0].x + 6 + app.writeup_path.len() as u16, // "File: " = 6
        input_chunks[0].y,
    ));
}
