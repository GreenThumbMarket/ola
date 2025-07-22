use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectFile {
    pub id: String,
    pub filename: String,
    pub size: u64,
    pub mime_type: Option<String>,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Goal {
    pub id: String,
    pub text: String,
    pub order: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
    pub id: String,
    pub text: String,
    pub order: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub files: Vec<ProjectFile>,
    pub goals: Vec<Goal>,
    pub contexts: Vec<Context>,
}

impl Project {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            created_at: now,
            updated_at: now,
            files: Vec::new(),
            goals: Vec::new(),
            contexts: Vec::new(),
        }
    }

    pub fn add_file(&mut self, file: ProjectFile) {
        self.files.push(file);
        self.updated_at = Utc::now();
    }

    pub fn remove_file(&mut self, file_id: &str) -> bool {
        let initial_len = self.files.len();
        self.files.retain(|f| f.id != file_id);
        if self.files.len() != initial_len {
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn add_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
        self.updated_at = Utc::now();
    }

    pub fn remove_goal(&mut self, goal_id: &str) -> bool {
        let initial_len = self.goals.len();
        self.goals.retain(|g| g.id != goal_id);
        if self.goals.len() != initial_len {
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn add_context(&mut self, context: Context) {
        self.contexts.push(context);
        self.updated_at = Utc::now();
    }

    pub fn remove_context(&mut self, context_id: &str) -> bool {
        let initial_len = self.contexts.len();
        self.contexts.retain(|c| c.id != context_id);
        if self.contexts.len() != initial_len {
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn reorder_goals(&mut self, goal_orders: HashMap<String, u32>) {
        for goal in &mut self.goals {
            if let Some(&new_order) = goal_orders.get(&goal.id) {
                goal.order = new_order;
            }
        }
        self.goals.sort_by_key(|g| g.order);
        self.updated_at = Utc::now();
    }

    pub fn reorder_contexts(&mut self, context_orders: HashMap<String, u32>) {
        for context in &mut self.contexts {
            if let Some(&new_order) = context_orders.get(&context.id) {
                context.order = new_order;
            }
        }
        self.contexts.sort_by_key(|c| c.order);
        self.updated_at = Utc::now();
    }
}

impl Goal {
    pub fn new(text: String, order: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            order,
        }
    }
}

impl Context {
    pub fn new(text: String, order: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            order,
        }
    }
}

impl ProjectFile {
    pub fn new(filename: String, size: u64, mime_type: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            filename,
            size,
            mime_type,
            uploaded_at: Utc::now(),
        }
    }
}