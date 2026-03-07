use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub active_workspace_id: String,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self {
            active_workspace_id: "primary".to_string(),
        }
    }
}
