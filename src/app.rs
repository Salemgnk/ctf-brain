use crate::models::CtfBox;
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum AppView {
    List,
    Details(i32),
    AddBox,
    DeleteBox(i32),
    EditEnvVars(i32),
    EditNotes(i32),
    WriteupExport(i32),
}

#[derive(Debug, Clone)]
pub struct AddBoxForm {
    pub title: String,
    pub platform: String,
    pub ip: String,
    pub tags: String,
    pub current_field: usize,
}

#[derive(Debug, Clone)]
pub struct EnvVarForm {
    pub key: String,
    pub value: String,
    pub current_field: usize,
}

#[derive(Debug, Clone)]
pub struct NoteForm {
    pub content: String,
    pub category_index: usize,
}

/// Message type for the status bar
#[derive(Debug, Clone, PartialEq)]
pub enum StatusKind {
    Info,
    Success,
    Error,
}

pub struct App {
    pub view: AppView,
    pub boxes: Vec<CtfBox>,
    pub selected_box_id: Option<i32>,
    pub should_quit: bool,
    pub selected_env_var: Option<usize>,
    pub selected_note: Option<usize>,
    pub status_message: Option<(String, StatusKind, Instant)>,
    pub writeup_path: String,
}

impl App {
    pub fn new(boxes: Vec<CtfBox>) -> Self {
        let selected_box_id = if !boxes.is_empty() { Some(0) } else { None };
        Self {
            view: AppView::List,
            boxes,
            selected_box_id,
            should_quit: false,
            selected_env_var: None,
            selected_note: None,
            status_message: None,
            writeup_path: String::new(),
        }
    }

    /// Set a status message that auto-expires after 4 seconds
    pub fn set_status(&mut self, msg: impl Into<String>, kind: StatusKind) {
        self.status_message = Some((msg.into(), kind, Instant::now()));
    }

    /// Clear the status message if it has expired
    pub fn tick_status(&mut self) {
        if let Some((_, _, when)) = &self.status_message {
            if when.elapsed().as_secs() >= 4 {
                self.status_message = None;
            }
        }
    }

    pub fn next(&mut self) {
        if self.boxes.is_empty() {
            return;
        }

        let current = self.selected_box_id.unwrap_or(0);
        let next_idx = (current + 1) % self.boxes.len() as i32;
        self.selected_box_id = Some(next_idx);
    }

    pub fn previous(&mut self) {
        if self.boxes.is_empty() {
            return;
        }

        let current = self.selected_box_id.unwrap_or(0);
        let prev_idx = if current == 0 {
            (self.boxes.len() - 1) as i32
        } else {
            current - 1
        };
        self.selected_box_id = Some(prev_idx);
    }

    pub fn select_current(&mut self) {
        if let Some(idx) = self.selected_box_id {
            if let Some(ctf_box) = self.boxes.get(idx as usize) {
                self.view = AppView::Details(ctf_box.id);
            }
        }
    }

    pub fn go_back(&mut self) {
        self.view = AppView::List;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn next_field(&mut self, form: &mut AddBoxForm) {
        form.current_field = (form.current_field + 1) % 4;
    }

    pub fn previous_field(&mut self, form: &mut AddBoxForm) {
        if form.current_field == 0 {
            form.current_field = 3;
        } else {
            form.current_field -= 1;
        }
    }

    pub fn start_add_box(&mut self) -> AddBoxForm {
        self.view = AppView::AddBox;
        AddBoxForm {
            title: String::new(),
            platform: String::from("HTB"),
            ip: String::new(),
            tags: String::new(),
            current_field: 0,
        }
    }

    pub fn cancel_form(&mut self) {
        self.view = AppView::List;
    }

    pub fn submit_add_box(&mut self, form: &AddBoxForm) -> Result<(), String> {
        // Validation
        if form.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        let ip_addr = form
            .ip
            .parse()
            .map_err(|_| "Invalid IP address".to_string())?;

        // Generate new ID
        let new_id = self.boxes.iter().map(|b| b.id).max().unwrap_or(0) + 1;

        // Parse tags
        let tags: Vec<String> = form
            .tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Create new box
        let new_box = CtfBox {
            id: new_id,
            title: form.title.trim().to_string(),
            platform: form.platform.clone(),
            ip_address: ip_addr,
            tags,
            created_date: chrono::Utc::now(),
            updated_date: chrono::Utc::now(),
            actions: Vec::new(),
            notes: Vec::new(),
            env_vars: HashMap::new(),
        };

        self.boxes.push(new_box);
        self.view = AppView::List;

        Ok(())
    }

    pub fn start_delete_box(&mut self) {
        if let Some(id) = self.selected_box_id {
            // Find actual box index to make sure it exists
            if self.boxes.iter().any(|b| b.id == id) {
                self.view = AppView::DeleteBox(id);
            }
        }
    }

    pub fn confirm_delete_box(&mut self, id: i32) {
        self.boxes.retain(|b| b.id != id);

        // Reset selection if necessary
        if self.boxes.is_empty() {
            self.selected_box_id = None;
        } else if let Some(current) = self.selected_box_id {
            if current == id {
                // If we deleted the selected box, select the first one
                self.selected_box_id = self.boxes.first().map(|b| b.id);
            }
        }

        self.view = AppView::List;
    }

    pub fn cancel_delete(&mut self) {
        self.view = AppView::List;
    }

    pub fn start_edit_env_vars(&mut self, box_id: i32) -> Option<EnvVarForm> {
        if self.boxes.iter().any(|b| b.id == box_id) {
            self.view = AppView::EditEnvVars(box_id);
            Some(EnvVarForm {
                key: String::new(),
                value: String::new(),
                current_field: 0,
            })
        } else {
            None
        }
    }

    pub fn add_env_var(&mut self, box_id: i32, key: String, value: String) -> Result<(), String> {
        // Validation
        if key.trim().is_empty() {
            return Err("Key cannot be empty".to_string());
        }

        // Only alphanumeric and underscore allowed
        if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err("Key must be alphanumeric with underscores only".to_string());
        }

        // Add to box
        if let Some(ctf_box) = self.boxes.iter_mut().find(|b| b.id == box_id) {
            ctf_box.env_vars.insert(key.trim().to_uppercase(), value);
            ctf_box.updated_date = chrono::Utc::now();
            Ok(())
        } else {
            Err("Box not found".to_string())
        }
    }

    #[allow(dead_code)]
    pub fn delete_env_var(&mut self, box_id: i32, key: &str) -> Result<(), String> {
        if let Some(ctf_box) = self.boxes.iter_mut().find(|b| b.id == box_id) {
            if ctf_box.env_vars.remove(key).is_some() {
                ctf_box.updated_date = chrono::Utc::now();
                Ok(())
            } else {
                Err("Variable not found".to_string())
            }
        } else {
            Err("Box not found".to_string())
        }
    }

    pub fn next_env_field(&mut self, form: &mut EnvVarForm) {
        form.current_field = (form.current_field + 1) % 2;
    }

    pub fn previous_env_field(&mut self, form: &mut EnvVarForm) {
        form.current_field = if form.current_field == 0 { 1 } else { 0 };
    }

    // ========== Env Var Navigation ==========

    pub fn next_env_var(&mut self, box_id: i32) {
        if let Some(ctf_box) = self.boxes.iter().find(|b| b.id == box_id) {
            let count = ctf_box.env_vars.len();
            if count == 0 {
                return;
            }
            self.selected_env_var = Some(match self.selected_env_var {
                Some(i) => (i + 1) % count,
                None => 0,
            });
        }
    }

    pub fn previous_env_var(&mut self, box_id: i32) {
        if let Some(ctf_box) = self.boxes.iter().find(|b| b.id == box_id) {
            let count = ctf_box.env_vars.len();
            if count == 0 {
                return;
            }
            self.selected_env_var = Some(match self.selected_env_var {
                Some(0) | None => count.saturating_sub(1),
                Some(i) => i - 1,
            });
        }
    }

    pub fn delete_selected_env_var(&mut self, box_id: i32) -> Result<(), String> {
        let selected = self.selected_env_var.ok_or("No variable selected")?;
        let ctf_box = self
            .boxes
            .iter_mut()
            .find(|b| b.id == box_id)
            .ok_or("Box not found")?;
        let keys: Vec<String> = ctf_box.env_vars.keys().cloned().collect();
        if selected >= keys.len() {
            return Err("Invalid selection".to_string());
        }
        ctf_box.env_vars.remove(&keys[selected]);
        ctf_box.updated_date = chrono::Utc::now();
        // Adjust selection
        let new_count = ctf_box.env_vars.len();
        if new_count == 0 {
            self.selected_env_var = None;
        } else if selected >= new_count {
            self.selected_env_var = Some(new_count - 1);
        }
        Ok(())
    }

    // ========== Notes Management ==========

    pub fn start_edit_notes(&mut self, box_id: i32) {
        if self.boxes.iter().any(|b| b.id == box_id) {
            self.view = AppView::EditNotes(box_id);
            self.selected_note = None;
        }
    }

    pub fn add_note(
        &mut self,
        box_id: i32,
        category_index: usize,
        content: String,
    ) -> Result<(), String> {
        if content.trim().is_empty() {
            return Err("Content cannot be empty".to_string());
        }

        let categories = Self::note_categories();
        if category_index >= categories.len() {
            return Err("Invalid category".to_string());
        }

        if let Some(ctf_box) = self.boxes.iter_mut().find(|b| b.id == box_id) {
            ctf_box.notes.push(crate::models::Note {
                category: categories[category_index].clone(),
                content: content.trim().to_string(),
                created_date: chrono::Utc::now(),
            });
            ctf_box.updated_date = chrono::Utc::now();
            Ok(())
        } else {
            Err("Box not found".to_string())
        }
    }

    pub fn delete_selected_note(&mut self, box_id: i32) -> Result<(), String> {
        let selected = self.selected_note.ok_or("No note selected")?;
        let ctf_box = self
            .boxes
            .iter_mut()
            .find(|b| b.id == box_id)
            .ok_or("Box not found")?;
        if selected >= ctf_box.notes.len() {
            return Err("Invalid selection".to_string());
        }
        ctf_box.notes.remove(selected);
        ctf_box.updated_date = chrono::Utc::now();
        let new_count = ctf_box.notes.len();
        if new_count == 0 {
            self.selected_note = None;
        } else if selected >= new_count {
            self.selected_note = Some(new_count - 1);
        }
        Ok(())
    }

    pub fn next_note(&mut self, box_id: i32) {
        if let Some(ctf_box) = self.boxes.iter().find(|b| b.id == box_id) {
            let count = ctf_box.notes.len();
            if count == 0 {
                return;
            }
            self.selected_note = Some(match self.selected_note {
                Some(i) => (i + 1) % count,
                None => 0,
            });
        }
    }

    pub fn previous_note(&mut self, box_id: i32) {
        if let Some(ctf_box) = self.boxes.iter().find(|b| b.id == box_id) {
            let count = ctf_box.notes.len();
            if count == 0 {
                return;
            }
            self.selected_note = Some(match self.selected_note {
                Some(0) | None => count.saturating_sub(1),
                Some(i) => i - 1,
            });
        }
    }

    pub fn note_categories() -> Vec<crate::models::NoteCategory> {
        vec![
            crate::models::NoteCategory::Recon,
            crate::models::NoteCategory::Foothold,
            crate::models::NoteCategory::Privesc,
            crate::models::NoteCategory::Web,
            crate::models::NoteCategory::Pwn,
            crate::models::NoteCategory::Crypto,
            crate::models::NoteCategory::Reversing,
            crate::models::NoteCategory::Stego,
            crate::models::NoteCategory::Misc,
        ]
    }

    // ========== Actions Import ==========

    /// Import actions from shell logs into the box
    pub fn import_shell_logs(&mut self, box_id: i32) -> Result<usize, String> {
        let actions = crate::storage::import_shell_logs(box_id)
            .map_err(|e| format!("Failed to import logs: {}", e))?;
        
        let count = actions.len();
        
        if let Some(ctf_box) = self.boxes.iter_mut().find(|b| b.id == box_id) {
            // Merge actions, avoiding duplicates based on timestamp and command
            for action in actions {
                let exists = ctf_box.actions.iter().any(|a| 
                    a.timestamp == action.timestamp && a.command == action.command
                );
                if !exists {
                    ctf_box.actions.push(action);
                }
            }
            ctf_box.updated_date = chrono::Utc::now();
            
            // Sort actions by timestamp
            ctf_box.actions.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        }
        
        // Clear the logs after import
        let _ = crate::storage::clear_shell_logs(box_id);
        
        Ok(count)
    }

    // ========== Write-up Generation ==========

    /// Start the write-up export flow with a default path
    pub fn start_writeup_export(&mut self, box_id: i32) {
        if let Some(ctf_box) = self.boxes.iter().find(|b| b.id == box_id) {
            let filename = format!("{}-writeup.md",
                ctf_box.title.to_lowercase().replace(' ', "-")
            );
            self.writeup_path = filename;
            self.view = AppView::WriteupExport(box_id);
        }
    }

    /// Generate and save a write-up for a box to the given path
    pub fn generate_writeup(&mut self, box_id: i32) -> Result<std::path::PathBuf, String> {
        let ctf_box = self
            .boxes
            .iter()
            .find(|b| b.id == box_id)
            .ok_or("Box not found")?;

        let path = std::path::PathBuf::from(&self.writeup_path);

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
        }

        let content = crate::storage::generate_writeup(ctf_box);
        std::fs::write(&path, &content)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(path)
    }

    /// Launch a shell with the box environment loaded
    /// This function replaces the current process with bash
    /// Launch a shell with the box environment loaded
    pub fn launch_box_shell(&self, box_id: i32) -> Result<(), String> {
        let ctf_box = self
            .boxes
            .iter()
            .find(|b| b.id == box_id)
            .ok_or("Box not found")?;

        crate::storage::create_box_environment(ctf_box)
            .map_err(|e| format!("Failed to create environment: {}", e))?;

        let env_file = dirs::home_dir()
            .ok_or("No home directory")?
            .join(format!(".ctf-brain/boxes/box-{}.env", box_id));

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

        println!("\n\x1b[32m╔══════════════════════════════════════╗");
        println!("║  🧠 CTF Brain Shell - {}", ctf_box.title);
        println!("║  📡 IP: {}", ctf_box.ip_address);
        println!("║  Tapez 'exit' pour revenir à CTF Brain");
        println!("╚══════════════════════════════════════╝\x1b[0m\n");

        let status = if shell.contains("zsh") {
            let boxes_dir = dirs::home_dir()
                .ok_or("No home directory")?
                .join(".ctf-brain/boxes");
            let zdotdir = boxes_dir.join(format!("zsh-{}", box_id));
            std::fs::create_dir_all(&zdotdir)
                .map_err(|e| format!("Failed to create zdotdir: {}", e))?;

            let custom_zshrc = format!(
                "# Source user's original .zshrc\n\
                 [ -f \"$HOME/.zshrc\" ] && source \"$HOME/.zshrc\"\n\
                 # Then apply CTF Brain environment (overrides prompt)\n\
                 source {}\n",
                env_file.display()
            );
            std::fs::write(zdotdir.join(".zshrc"), custom_zshrc)
                .map_err(|e| format!("Failed to write custom .zshrc: {}", e))?;

            Command::new(&shell)
                .env("ZDOTDIR", &zdotdir)
                .status()
                .map_err(|e| format!("Failed to spawn zsh: {}", e))?
        } else {
            Command::new(&shell)
                .arg("--rcfile")
                .arg(&env_file)
                .status()
                .map_err(|e| format!("Failed to spawn bash: {}", e))?
        };

        if !status.success() {
            return Err(format!("Shell exited with status: {}", status));
        }

        Ok(())
    }
}
