mod app;
mod models;
mod storage;
mod ui;

use anyhow::Result;
use app::{AddBoxForm, App, AppView};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> Result<()> {
    // Load boxes from storage, or use sample data if empty
    let mut boxes = storage::load_boxes().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load boxes ({}). Starting with sample data.", e);
        Vec::new()
    });
    
    // If no boxes exist, create sample data for testing
    if boxes.is_empty() {
        boxes = vec![
        models::CtfBox {
            id: 1,
            title: "Lame".to_string(),
            platform: "HTB".to_string(),
            ip_address: "10.10.10.3".parse().unwrap(),
            tags: vec!["easy".to_string(), "linux".to_string()],
            created_date: chrono::Utc::now(),
            updated_date: chrono::Utc::now(),
            actions: vec![
                models::Action {
                    timestamp: chrono::Utc::now(),
                    command: "nmap -sV 10.10.10.3".to_string(),
                    result: models::ActionResult::Success,
                    note: Some("Found open ports 21, 22, 445".to_string()),
                },
                models::Action {
                    timestamp: chrono::Utc::now(),
                    command: "gobuster dir -u http://10.10.10.3".to_string(),
                    result: models::ActionResult::Fail,
                    note: None,
                },
            ],
            notes: vec![
                models::Note {
                    category: models::NoteCategory::Recon,
                    content: "SMB version is outdated - potential exploit".to_string(),
                    created_date: chrono::Utc::now(),
                },
                models::Note {
                    category: models::NoteCategory::Foothold,
                    content: "Try CVE-2007-2447 for vsftpd".to_string(),
                    created_date: chrono::Utc::now(),
                },
            ],
        },
        models::CtfBox {
            id: 2,
            title: "Web Gauntlet".to_string(),
            platform: "picoCTF".to_string(),
            ip_address: "192.168.1.100".parse().unwrap(),
            tags: vec!["web".to_string(), "sql".to_string()],
            created_date: chrono::Utc::now(),
            updated_date: chrono::Utc::now(),
            actions: vec![],
            notes: vec![
                models::Note {
                    category: models::NoteCategory::Web,
                    content: "SQL injection vulnerability in login form".to_string(),
                    created_date: chrono::Utc::now(),
                },
            ],
        },
        models::CtfBox {
            id: 3,
            title: "Blue".to_string(),
            platform: "TryHackMe".to_string(),
            ip_address: "10.10.88.45".parse().unwrap(),
            tags: vec!["windows".to_string(), "medium".to_string()],
            created_date: chrono::Utc::now(),
            updated_date: chrono::Utc::now(),
            actions: vec![
                models::Action {
                    timestamp: chrono::Utc::now(),
                    command: "msfconsole".to_string(),
                    result: models::ActionResult::Unknown,
                    note: Some("Testing EternalBlue exploit".to_string()),
                },
            ],
            notes: vec![],
        },
    ];
    }

    let mut app = App::new(boxes);
    let mut add_box_form: Option<AddBoxForm> = None;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    loop {
        // Render
        terminal.draw(|f| {
            let area = f.area();
            match &app.view {
                AppView::List => ui::list::render(f, &app, area),
                AppView::Details(id) => ui::detail::render(f, &app, area, *id),
                AppView::AddBox => {
                    // Render list in background
                    ui::list::render(f, &app, area);
                    // Render form modal on top
                    if let Some(form) = &add_box_form {
                        ui::add_box::render(f, &app, form, area);
                    }
                }
            }
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        if app.view != AppView::AddBox {
                            app.quit()
                        }
                    }
                    KeyCode::Char('a') => {
                        if app.view == AppView::List {
                            add_box_form = Some(app.start_add_box());
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.view == AppView::List {
                            app.next();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.view == AppView::List {
                            app.previous();
                        }
                    }
                    KeyCode::Enter => {
                        if app.view == AppView::List {
                            app.select_current();
                        } else if app.view == AppView::AddBox {
                            if let Some(form) = &add_box_form {
                                match app.submit_add_box(form) {
                                    Ok(_) => {
                                        // Save immediately
                                        if let Err(e) = storage::save_boxes(&app.boxes) {
                                            eprintln!("Failed to save: {}", e);
                                        }
                                        add_box_form = None;
                                    }
                                    Err(e) => {
                                        // TODO: Show error message
                                        eprintln!("Validation error: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Tab => {
                        if let Some(form) = &mut add_box_form {
                            app.next_field(form);
                        }
                    }
                    KeyCode::BackTab => {
                        if let Some(form) = &mut add_box_form {
                            app.previous_field(form);
                        }
                    }
                    KeyCode::Char(c) => {
                        if let Some(form) = &mut add_box_form {
                            match form.current_field {
                                0 => form.title.push(c),
                                1 => form.platform.push(c),
                                2 => form.ip.push(c),
                                3 => form.tags.push(c),
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(form) = &mut add_box_form {
                            match form.current_field {
                                0 => { form.title.pop(); }
                                1 => { form.platform.pop(); }
                                2 => { form.ip.pop(); }
                                3 => { form.tags.pop(); }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Esc => {
                        if app.view == AppView::AddBox {
                            app.cancel_form();
                            add_box_form = None;
                        } else {
                            app.go_back();
                        }
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Save boxes before exit
    if let Err(e) = storage::save_boxes(&app.boxes) {
        eprintln!("Warning: Failed to save boxes: {}", e);
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
