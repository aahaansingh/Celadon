use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    MarkRead(i32),
    MarkUnread(i32),
    DeleteArticle(i32),
    DeleteFeed(i32),
    DeleteSuperfeed(i32),
    DeleteTag(i32),
}

pub struct UndoStack {
    stack: Mutex<Vec<Action>>,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            stack: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&self, action: Action) -> Option<Action> {
        let mut stack = self.stack.lock().unwrap();
        stack.push(action);
        // Limit stack size to 50 for memory safety
        if stack.len() > 50 {
            Some(stack.remove(0))
        } else {
            None
        }
    }

    pub fn pop(&self) -> Option<Action> {
        let mut stack = self.stack.lock().unwrap();
        stack.pop()
    }

    pub fn clear(&self) {
        let mut stack = self.stack.lock().unwrap();
        stack.clear();
    }
}
