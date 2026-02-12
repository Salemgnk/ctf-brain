use crate::app::AppView;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Render a footer with keyboard shortcuts based on the current view
pub fn render_footer(f: &mut Frame, view: &AppView, area: Rect) {
    let shortcuts = match view {
        AppView::List => vec![
            ("j/k/↑/↓", "Navigate", Color::Green),
            ("Enter", "Details", Color::Cyan),
            ("a", "Add Box", Color::Yellow),
            ("d", "Delete", Color::Red),
            ("l", "Launch Shell", Color::Magenta),
            ("q", "Quit", Color::Red),
        ],
        AppView::Details(_) => vec![
            ("e", "Edit Vars", Color::Yellow),
            ("l", "Launch Shell", Color::Magenta),
            ("Esc", "Back", Color::Cyan),
            ("q", "Quit", Color::Red),
        ],
        AppView::AddBox => vec![
            ("Tab", "Next Field", Color::Green),
            ("Shift+Tab", "Prev Field", Color::Green),
            ("Enter", "Submit", Color::Cyan),
            ("Esc", "Cancel", Color::Red),
        ],
        AppView::DeleteBox(_) => vec![
            ("y", "Confirm", Color::Red),
            ("n/Esc", "Cancel", Color::Green),
        ],
        AppView::EditEnvVars(_) => vec![
            ("a", "Add", Color::Yellow),
            ("d", "Delete", Color::Red),
            ("j/k", "Navigate", Color::Green),
            ("Esc", "Back", Color::Cyan),
        ],
        AppView::EditNotes(_) => vec![
            ("a", "Add", Color::Yellow),
            ("d", "Delete", Color::Red),
            ("j/k", "Navigate", Color::Green),
            ("Esc", "Back", Color::Cyan),
        ],
    };

    let mut spans = Vec::new();
    for (i, (key, action, color)) in shortcuts.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" │ "));
        }
        spans.push(Span::styled(
            *key,
            Style::default().fg(*color).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(format!(": {}", action)));
    }

    let footer = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(footer, area);
}
