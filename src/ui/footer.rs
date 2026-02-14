use crate::app::{AppView, StatusKind};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Render a footer with keyboard shortcuts and optional status message
pub fn render_footer(
    f: &mut Frame,
    view: &AppView,
    status: Option<&(String, StatusKind, std::time::Instant)>,
    area: Rect,
) {
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
            ("n", "Edit Notes", Color::Yellow),
            ("w", "Write-up", Color::Green),
            ("l", "Shell", Color::Magenta),
            ("Esc", "Back", Color::Cyan),
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
        AppView::WriteupExport(_) => vec![
            ("Enter", "Export", Color::Green),
            ("Esc", "Cancel", Color::Red),
        ],
    };

    // If there's a status message, split footer into 2 lines
    if let Some((msg, kind, _)) = status {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Status message
                Constraint::Length(1), // Shortcuts
            ])
            .split(area);

        let (icon, color) = match kind {
            StatusKind::Success => ("✔ ", Color::Green),
            StatusKind::Error => ("✗ ", Color::Red),
            StatusKind::Info => ("ℹ ", Color::Cyan),
        };

        let status_line = Paragraph::new(Line::from(vec![
            Span::styled(icon, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(msg.as_str(), Style::default().fg(color)),
        ]))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(color)),
        );
        f.render_widget(status_line, chunks[0]);

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
        let shortcut_line = Paragraph::new(Line::from(spans))
            .style(Style::default().fg(Color::White));
        f.render_widget(shortcut_line, chunks[1]);
    } else {
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
}
