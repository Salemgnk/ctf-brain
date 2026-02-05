use crate::models::CtfBox;

#[derive(Debug, Clone, PartialEq)]
pub enum AppView {
    List,
    Details(i32),
    AddBox,
}

#[derive(Debug, Clone)]
pub struct AddBoxForm {
    pub title: String,
    pub platform: String,
    pub ip: String,
    pub tags: String,
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
        
        let ip_addr = form.ip.parse()
            .map_err(|_| "Invalid IP address".to_string())?;
        
        // Generate new ID
        let new_id = self.boxes.iter().map(|b| b.id).max().unwrap_or(0) + 1;
        
        // Parse tags
        let tags: Vec<String> = form.tags
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
        };
        
        self.boxes.push(new_box);
        self.view = AppView::List;
        
        Ok(())
    }
}