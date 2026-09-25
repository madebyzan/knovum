use super::WorkflowNode;

impl WorkflowNode {
    pub fn id(&self) -> &str {
        match self {
            Self::Input { id, .. }
            | Self::Output { id, .. }
            | Self::Action { id, .. }
            | Self::Join { id, .. }
            | Self::Review { id, .. } => id,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Input { .. } => "input",
            Self::Output { .. } => "output",
            Self::Action { .. } => "action",
            Self::Join { .. } => "join",
            Self::Review { .. } => "review",
        }
    }
}
