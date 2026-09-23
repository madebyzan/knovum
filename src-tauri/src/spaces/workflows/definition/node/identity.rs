use super::WorkflowNode;

impl WorkflowNode {
    pub fn id(&self) -> &str {
        match self {
            Self::Input { id, .. }
            | Self::Output { id, .. }
            | Self::Action { id, .. }
            | Self::Condition { id, .. }
            | Self::Parallel { id }
            | Self::Join { id, .. }
            | Self::ForEach { id, .. }
            | Self::Iterate { id, .. }
            | Self::Subworkflow { id, .. }
            | Self::HumanInput { id, .. }
            | Self::Review { id, .. }
            | Self::Delay { id, .. }
            | Self::DataTransform { id, .. }
            | Self::ExternalTool { id, .. }
            | Self::DelegateTask { id, .. }
            | Self::Publish { id, .. } => id,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Input { .. } => "input",
            Self::Output { .. } => "output",
            Self::Action { .. } => "action",
            Self::Condition { .. } => "condition",
            Self::Parallel { .. } => "parallel",
            Self::Join { .. } => "join",
            Self::ForEach { .. } => "for_each",
            Self::Iterate { .. } => "iterate",
            Self::Subworkflow { .. } => "subworkflow",
            Self::HumanInput { .. } => "human_input",
            Self::Review { .. } => "review",
            Self::Delay { .. } => "delay",
            Self::DataTransform { .. } => "data_transform",
            Self::ExternalTool { .. } => "external_tool",
            Self::DelegateTask { .. } => "delegate_task",
            Self::Publish { .. } => "publish",
        }
    }
}
