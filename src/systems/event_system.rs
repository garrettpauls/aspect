use super::events::AppEvent;

use std::slice::Iter;
use std::sync::Mutex;

pub struct EventSystem {
    current: Vec<AppEvent>,
    pending: Mutex<Vec<AppEvent>>,
}

impl EventSystem {
    pub fn new() -> Self {
        EventSystem {
            current: Vec::new(),
            pending: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&mut self, event: AppEvent) {
        log::trace!("Queuing event: {:?}", event);
        self.pending.lock().unwrap().push(event);
    }

    pub fn events(&self) -> Iter<AppEvent> { self.current.iter() }

    pub fn update(&mut self) {
        use std::mem::replace;
        let mut pending = self.pending.lock().unwrap();
        self.current = replace(&mut *pending, Vec::new());
    }
}
