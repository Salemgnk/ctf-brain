mod app;
mod models;
mod storage;
mod ui;

use anyhow::Result;
use app::{AddBoxForm, App, AppView, EnvVarForm, NoteForm, StatusKind};
use crossterm::{
    cursor::Show,
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
                        output: Some("PORT    STATE SERVICE     VERSION\n21/tcp  open  ftp         vsftpd 2.3.4\n22/tcp  open  ssh         OpenSSH 4.7p1\n445/tcp open  netbios-ssn Samba smbd 3.X".to_string()),
                    },
                    models::Action {
                        timestamp: chrono::Utc::now(),
                        command: "gobuster dir -u http://10.10.10.3".to_string(),
                        result: models::ActionResult::Fail,
                        note: None,
                        output: None,
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
                    output: None,
                }],
                notes: vec![],
                env_vars: HashMap::new(),
            },
        ];
    }

    let mut app = App::new(boxes);
    let mut add_box_form: Option<AddBoxForm> = None;
    let mut env_var_form: Option<EnvVarForm> = None;
    let mut note_form: Option<NoteForm> = None;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    loop {
        // Expire old status messages
        app.tick_status();

        // Render
        terminal.draw(|f| {
            let area = f.area();

            // Footer height: 3 if status message, 2 otherwise
            let footer_height = if app.status_message.is_some() { 3 } else { 2 };

            // Create layout with footer
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),                      // Main content
                    Constraint::Length(footer_height),        // Footer
                ])
                .split(area);

            // Render main view
            match &app.view {
                AppView::List => ui::list::render(f, &app, main_chunks[0]),
                AppView::Details(id) => ui::detail::render(f, &app, main_chunks[0], *id),
                AppView::DeleteBox(id) => {
                    ui::list::render(f, &app, main_chunks[0]);
                    ui::delete_box::render(f, &app, main_chunks[0], *id);
                }
                AppView::AddBox => {
                    ui::list::render(f, &app, main_chunks[0]);
                    if let Some(form) = &add_box_form {
                        ui::add_box::render(f, &app, form, main_chunks[0]);
                    }
                }
                AppView::EditEnvVars(id) => {
                    ui::edit_env_vars::render(f, &app, env_var_form.as_ref(), main_chunks[0], *id);
                }
                AppView::EditNotes(id) => {
                    ui::edit_notes::render(f, &app, note_form.as_ref(), main_chunks[0], *id);
                }
                AppView::WriteupExport(id) => {
                    ui::writeup_export::render(f, &app, main_chunks[0], *id);
                }
            }

            // Render footer with shortcuts + optional status
            ui::footer::render_footer(f, &app.view, app.status_message.as_ref(), main_chunks[1]);
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
                                match app.add_env_var(box_id, form.key.clone(), form.value.clone())
                                {
                                    Ok(_) => {
                                        if let Err(e) = storage::save_boxes(&app.boxes) {
                                            app.set_status(format!("Save failed: {}", e), StatusKind::Error);
                                        } else {
                                            app.set_status("Variable added", StatusKind::Success);
                                        }
                                        // Reset form
                                        form.key.clear();
                                        form.value.clear();
                                        form.current_field = 0;
                                    }
                                    Err(e) => {
                                        app.set_status(e, StatusKind::Error);
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
                            KeyCode::Char('j') | KeyCode::Down => {
                                app.next_env_var(box_id);
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                app.previous_env_var(box_id);
                            }
                            KeyCode::Char('d') => {
                                match app.delete_selected_env_var(box_id) {
                                    Ok(_) => {
                                        if let Err(e) = storage::save_boxes(&app.boxes) {
                                            app.set_status(format!("Save failed: {}", e), StatusKind::Error);
                                        } else {
                                            app.set_status("Variable deleted", StatusKind::Success);
                                        }
                                    }
                                    Err(e) => {
                                        app.set_status(e, StatusKind::Error);
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                app.view = AppView::Details(box_id);
                            }
                            _ => {}
                        }
                    }
                }
                // Handle EditNotes view
                else if let AppView::EditNotes(box_id) = app.view {
                    if let Some(form) = &mut note_form {
                        match key.code {
                            KeyCode::Char(c) => {
                                form.content.push(c);
                            }
                            KeyCode::Backspace => {
                                form.content.pop();
                            }
                            KeyCode::Left => {
                                let categories_len = App::note_categories().len();
                                if form.category_index == 0 {
                                    form.category_index = categories_len - 1;
                                } else {
                                    form.category_index -= 1;
                                }
                            }
                            KeyCode::Right => {
                                let categories_len = App::note_categories().len();
                                form.category_index = (form.category_index + 1) % categories_len;
                            }
                            KeyCode::Enter => {
                                match app.add_note(box_id, form.category_index, form.content.clone()) {
                                    Ok(_) => {
                                        if let Err(e) = storage::save_boxes(&app.boxes) {
                                            app.set_status(format!("Save failed: {}", e), StatusKind::Error);
                                        } else {
                                            app.set_status("Note added", StatusKind::Success);
                                        }
                                        // Reset form
                                        form.content.clear();
                                        form.category_index = 0;
                                    }
                                    Err(e) => {
                                        app.set_status(e, StatusKind::Error);
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                app.view = AppView::Details(box_id);
                                note_form = None;
                            }
                            _ => {}
                        }
                    } else {
                        // No form active, just viewing
                        match key.code {
                            KeyCode::Char('a') => {
                                note_form = Some(NoteForm {
                                    content: String::new(),
                                    category_index: 0,
                                });
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                app.next_note(box_id);
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                app.previous_note(box_id);
                            }
                            KeyCode::Char('d') => {
                                match app.delete_selected_note(box_id) {
                                    Ok(_) => {
                                        if let Err(e) = storage::save_boxes(&app.boxes) {
                                            app.set_status(format!("Save failed: {}", e), StatusKind::Error);
                                        } else {
                                            app.set_status("Note deleted", StatusKind::Success);
                                        }
                                    }
                                    Err(e) => {
                                        app.set_status(e, StatusKind::Error);
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                app.view = AppView::Details(box_id);
                            }
                            _ => {}
                        }
                    }
                }
                // Handle WriteupExport view
                else if let AppView::WriteupExport(box_id) = app.view {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.writeup_path.push(c);
                        }
                        KeyCode::Backspace => {
                            app.writeup_path.pop();
                        }
                        KeyCode::Enter => {
                            match app.generate_writeup(box_id) {
                                Ok(path) => {
                                    app.set_status(
                                        format!("Write-up exported → {}", path.display()),
                                        StatusKind::Success,
                                    );
                                    app.view = AppView::Details(box_id);
                                }
                                Err(e) => {
                                    app.set_status(e, StatusKind::Error);
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.view = AppView::Details(box_id);
                        }
                        _ => {}
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
                                        app.set_status(format!("Save failed: {}", e), StatusKind::Error);
                                    } else {
                                        app.set_status("Box added", StatusKind::Success);
                                    }
                                    add_box_form = None;
                                }
                                Err(e) => {
                                    app.set_status(e, StatusKind::Error);
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
                        // Touche 'n' dans Details pour éditer notes
                        KeyCode::Char('n') if matches!(app.view, AppView::Details(_)) => {
                            if let AppView::Details(id) = app.view {
                                app.start_edit_notes(id);
                            }
                        }
                        // Touche 'w' dans Details pour ouvrir l'export write-up
                        KeyCode::Char('w') if matches!(app.view, AppView::Details(_)) => {
                            if let AppView::Details(id) = app.view {
                                app.start_writeup_export(id);
                            }
                        }
                        // Touche 'l' pour lancer le shell
                        KeyCode::Char('l') => match &app.view {
                            AppView::List => {
                                if let Some(idx) = app.selected_box_id {
                                    if let Some(ctf_box) = app.boxes.get(idx as usize) {
                                        let box_id = ctf_box.id;
                                        disable_raw_mode()?;
                                        execute!(
                                            terminal.backend_mut(),
                                            LeaveAlternateScreen,
                                            Show
                                        )?;

                                        if let Err(e) = app.launch_box_shell(box_id) {
                                            eprintln!("Failed to launch shell: {}", e);
                                        }

                                        // Import shell logs after returning
                                        match app.import_shell_logs(box_id) {
                                            Ok(count) if count > 0 => {
                                                if let Err(e) = storage::save_boxes(&app.boxes) {
                                                    eprintln!("Save failed: {}", e);
                                                }
                                            }
                                            _ => {}
                                        }

                                        enable_raw_mode()?;
                                        execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                                        terminal.clear()?;

                                        // Show status in TUI after returning
                                        app.set_status("Shell session ended — commands imported", StatusKind::Info);
                                    }
                                }
                            }
                            AppView::Details(id) => {
                                let box_id = *id;
                                disable_raw_mode()?;
                                execute!(terminal.backend_mut(), LeaveAlternateScreen, Show)?;

                                if let Err(e) = app.launch_box_shell(box_id) {
                                    eprintln!("Failed to launch shell: {}", e);
                                }

                                // Import shell logs after returning
                                match app.import_shell_logs(box_id) {
                                    Ok(count) if count > 0 => {
                                        if let Err(e) = storage::save_boxes(&app.boxes) {
                                            eprintln!("Save failed: {}", e);
                                        }
                                    }
                                    _ => {}
                                }

                                enable_raw_mode()?;
                                execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                                terminal.clear()?;

                                app.set_status("Shell session ended — commands imported", StatusKind::Info);
                            }
                            _ => {}
                        },
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
                                        app.set_status(format!("Save failed: {}", e), StatusKind::Error);
                                    } else {
                                        app.set_status("Box deleted", StatusKind::Success);
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
