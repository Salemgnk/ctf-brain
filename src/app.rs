use crate::models::CtfBox;

#[derive(Debug, Clone, PartialEq)]

pub enum AppView {
    List,
    Details(i32),
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
}