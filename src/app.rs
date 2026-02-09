use crate::models::CtfBox;
use std::collections::HashMap;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum AppView {
    List,
    Details(i32),
    AddBox,
    DeleteBox(i32),
    EditEnvVars(i32),
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

pub struct App {
    pub view: AppView,
    pub boxes: Vec<CtfBox>,
    pub selected_box_id: Option<i32>,
    pub should_quit: bool,
}

impl App {
    pub fn new(boxes: Vec<CtfBox>) -> Self {
        let selected_box_id = if !boxes.is_empty() { Some(0) } else { None };
        Self {
            view: AppView::List,
            boxes,
            selected_box_id,
            should_quit: false,
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

    /// Launch a shell with the box environment loaded
    /// This function replaces the current process with bash
    pub fn launch_box_shell(&self, box_id: i32) -> Result<(), String> {
        let ctf_box = self.boxes.iter()
            .find(|b| b.id == box_id)
            .ok_or("Box not found")?;
        
        // Create/update the environment file
        crate::storage::create_box_environment(ctf_box)
            .map_err(|e| format!("Failed to create environment: {}", e))?;
        
        // Get path to the environment file
        let env_file = dirs::home_dir()
            .ok_or("No home directory")?
            .join(format!(".ctf-brain/boxes/box-{}.env", box_id));
        
        // On Unix systems, we can replace the process with exec()
        #[cfg(unix)]
        {
            // This never returns on success
            let error = Command::new("bash")
                .arg("--rcfile")
                .arg(env_file)
                .exec();
            
            // Only reached if exec fails
            Err(format!("Failed to exec bash: {}", error))
        }
        
        // On non-Unix, just spawn (not ideal but works)
        #[cfg(not(unix))]
        {
            Command::new("bash")
                .arg("--rcfile")
                .arg(env_file)
                .spawn()
                .map_err(|e| format!("Failed to spawn bash: {}", e))?;
            
            Ok(())
        }
    }
}