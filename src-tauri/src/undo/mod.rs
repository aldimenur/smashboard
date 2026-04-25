use crate::models::timeline::TimelineEvent;

#[derive(Clone, Debug)]
pub struct EventTimeChange {
    pub event_id: String,
    pub old_time_ms: f64,
    pub new_time_ms: f64,
}

#[derive(Clone, Debug)]
pub enum UndoAction {
    AddEvents(Vec<TimelineEvent>),
    DeleteEvents(Vec<TimelineEvent>),
    UpdateEventTimes(Vec<EventTimeChange>),
}

pub struct UndoManager {
    undo_stack: Vec<UndoAction>,
    redo_stack: Vec<UndoAction>,
    max_depth: usize,
}

impl UndoManager {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    pub fn push(&mut self, action: UndoAction) {
        self.undo_stack.push(action);
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }

        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<UndoAction> {
        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<UndoAction> {
        if let Some(action) = self.redo_stack.pop() {
            self.undo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
