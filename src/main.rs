mod app;
mod models;
mod storage;
mod ui;

use anyhow::Result;
use app::{AddBoxForm, App, AppView, EnvVarForm};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal, 
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};
use std::collections::HashMap;
use std::io;

fn main() -> Result<()> {
    // Install shell hook if not present
    if let Err(e) = storage::ensure_shell_hook_installed() {
        eprintln!("Warning: Failed to install shell hook: {}", e);
    }

    // Load boxes from storage, or use sample data if empty
    let mut boxes = storage::load_boxes().unwrap_or_else(|e| {
        eprintln!(
            "Warning: Failed to load boxes ({}). Starting with sample data.",
            e
        );
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
                env_vars: HashMap::new(),
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
                notes: vec![models::Note {
                    category: models::NoteCategory::Web,
                    content: "SQL injection vulnerability in login form".to_string(),
                    created_date: chrono::Utc::now(),
                }],
                env_vars: HashMap::new(),
            },
            models::CtfBox {
                id: 3,
                title: "Blue".to_string(),
                platform: "TryHackMe".to_string(),
                ip_address: "10.10.88.45".parse().unwrap(),
                tags: vec!["windows".to_string(), "medium".to_string()],
                created_date: chrono::Utc::now(),
                updated_date: chrono::Utc::now(),
                actions: vec![models::Action {
                    timestamp: chrono::Utc::now(),
                    command: "msfconsole".to_string(),
                    result: models::ActionResult::Unknown,
                    note: Some("Testing EternalBlue exploit".to_string()),
                }],
                notes: vec![],
                env_vars: HashMap::new(),
            },
        ];
    }

    let mut app = App::new(boxes);
    let mut add_box_form: Option<AddBoxForm> = None;
    let mut env_var_form: Option<EnvVarForm> = None;

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
            
            // Create layout with footer
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),      // Main content
                    Constraint::Length(2),   // Footer
                ])
                .split(area);
            
            // Render main view
            match &app.view {
                AppView::List => ui::list::render(f, &app, main_chunks[0]),
                AppView::Details(id) => ui::detail::render(f, &app, main_chunks[0], *id),
                AppView::DeleteBox(id) => {
                    // Render list in background
                    ui::list::render(f, &app, main_chunks[0]);
                    // Render delete modal on top
                    ui::delete_box::render(f, &app, main_chunks[0], *id);
                }
                AppView::AddBox => {
                    // Render list in background
                    ui::list::render(f, &app, main_chunks[0]);
                    // Render form modal on top
                    if let Some(form) = &add_box_form {
                        ui::add_box::render(f, &app, form, main_chunks[0]);
                    }
                }
                AppView::EditEnvVars(id) => {
                    ui::edit_env_vars::render(f, &app, env_var_form.as_ref(), main_chunks[0], *id);
                }
            }
            
            // Render footer with shortcuts
            ui::footer::render_footer(f, &app.view, main_chunks[1]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events, not release
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                // Handle env vars form
                if let AppView::EditEnvVars(box_id) = app.view {
                    if let Some(form) = &mut env_var_form {
                        match key.code {
                            KeyCode::Char(c) => {
                                if form.current_field == 0 {
                                    form.key.push(c);
                                } else {
                                    form.value.push(c);
                                }
                            }
                            KeyCode::Backspace => {
                                if form.current_field == 0 {
                                    form.key.pop();
                                } else {
                                    form.value.pop();
                                }
                            }
                            KeyCode::Tab => {
                                app.next_env_field(form);
                            }
                            KeyCode::BackTab => {
                                app.previous_env_field(form);
                            }
                            KeyCode::Enter => {
                                match app.add_env_var(box_id, form.key.clone(), form.value.clone()) {
                                    Ok(_) => {
                                        if let Err(e) = storage::save_boxes(&app.boxes) {
                                            eprintln!("Failed to save: {}", e);
                                        }
                                        // Reset form
                                        form.key.clear();
                                        form.value.clear();
                                        form.current_field = 0;
                                    }
                                    Err(e) => {
                                        eprintln!("Error: {}", e);
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                app.view = AppView::Details(box_id);
                                env_var_form = None;
                            }
                            _ => {}
                        }
                    } else {
                        // No form active, just viewing
                        match key.code {
                            KeyCode::Char('a') => {
                                env_var_form = app.start_edit_env_vars(box_id);
                            }
                            KeyCode::Esc => {
                                app.view = AppView::Details(box_id);
                            }
                            _ => {}
                        }
                    }
                }
                // If in AddBox form, handle text input first
                else if app.view == AppView::AddBox {
                    if let Some(form) = &mut add_box_form {
                        match key.code {
                            KeyCode::Char(c) => match form.current_field {
                                0 => form.title.push(c),
                                1 => form.platform.push(c),
                                2 => form.ip.push(c),
                                3 => form.tags.push(c),
                                _ => {}
                            },
                            KeyCode::Tab => app.next_field(form),
                            KeyCode::BackTab => app.previous_field(form),
                            KeyCode::Backspace => match form.current_field {
                                0 => {
                                    form.title.pop();
                                }
                                1 => {
                                    form.platform.pop();
                                }
                                2 => {
                                    form.ip.pop();
                                }
                                3 => {
                                    form.tags.pop();
                                }
                                _ => {}
                            },
                            KeyCode::Enter => match app.submit_add_box(form) {
                                Ok(_) => {
                                    if let Err(e) = storage::save_boxes(&app.boxes) {
                                        eprintln!("Failed to save: {}", e);
                                    }
                                    add_box_form = None;
                                }
                                Err(e) => {
                                    eprintln!("Validation error: {}", e);
                                }
                            },
                            KeyCode::Esc => {
                                app.cancel_form();
                                add_box_form = None;
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Handle other views
                    match key.code {
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Char('a') if app.view == AppView::List => {
                            add_box_form = Some(app.start_add_box());
                        }
                        KeyCode::Char('d') if app.view == AppView::List => app.start_delete_box(),
                        KeyCode::Char('j') | KeyCode::Down if app.view == AppView::List => {
                            app.next()
                        }
                        KeyCode::Char('k') | KeyCode::Up if app.view == AppView::List => {
                            app.previous()
                        }
                        KeyCode::Enter if app.view == AppView::List => app.select_current(),
                        // Touche 'e' dans Details pour éditer env vars
                        KeyCode::Char('e') if matches!(app.view, AppView::Details(_)) => {
                            if let AppView::Details(id) = app.view {
                                env_var_form = app.start_edit_env_vars(id);
                            }
                        }
                        // Touche 'l' pour lancer le shell
                        KeyCode::Char('l') => {
                            match &app.view {
                                AppView::List => {
                                    if let Some(idx) = app.selected_box_id {
                                        if let Some(ctf_box) = app.boxes.get(idx as usize) {
                                            // Disable raw mode before exec
                                            disable_raw_mode()?;
                                            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                                            if let Err(e) = app.launch_box_shell(ctf_box.id) {
                                                eprintln!("Failed to launch shell: {}", e);
                                                // Re-enable raw mode if exec failed
                                                enable_raw_mode()?;
                                                execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                                            }
                                        }
                                    }
                                }
                                AppView::Details(id) => {
                                    // Disable raw mode before exec
                                    disable_raw_mode()?;
                                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                                    if let Err(e) = app.launch_box_shell(*id) {
                                        eprintln!("Failed to launch shell: {}", e);
                                        // Re-enable raw mode if exec failed
                                        enable_raw_mode()?;
                                        execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                                    }
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Esc => {
                            if let AppView::DeleteBox(_) = app.view {
                                app.cancel_delete();
                            } else {
                                app.go_back();
                            }
                        }
                        KeyCode::Char(c) => {
                            if let AppView::DeleteBox(id) = app.view {
                                if c == 'y' || c == 'Y' {
                                    app.confirm_delete_box(id);
                                    if let Err(e) = storage::save_boxes(&app.boxes) {
                                        eprintln!("Failed to save: {}", e);
                                    }
                                } else if c == 'n' || c == 'N' {
                                    app.cancel_delete();
                                }
                            }
                        }
                        _ => {}
                    }
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