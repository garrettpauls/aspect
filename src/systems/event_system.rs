use super::events::AppEvent;

use std::slice::Iter;

pub struct EventSystem {
    current: Vec<AppEvent>,
    pending: Vec<AppEvent>,
}

impl EventSystem {
    pub fn new() -> Self {
        EventSystem {
            current: Vec::new(),
            pending: Vec::new(),
        }
    }

    pub fn push(&mut self, event: AppEvent) { self.pending.push(event); }

    pub fn events(&self) -> Iter<AppEvent> { self.current.iter() }

    pub fn update(&mut self) {
        use std::mem::replace;
        self.current = replace(&mut self.pending, Vec::new());
    }
}
