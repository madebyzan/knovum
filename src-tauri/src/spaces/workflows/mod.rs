mod compiler;
pub(crate) mod definition;
mod execution;
pub(crate) mod expressions;
pub(crate) mod external_tools;
mod history_extraction;
mod personal_templates;
mod readiness;
pub(crate) mod repository;
mod runtime;
mod template_dependencies;
mod templates;
mod workflow_dependencies;
mod workflow_rows;

pub(crate) use compiler::{compile, WorkflowDiagnostic, WorkflowExecutionPlan};
pub(crate) use definition::{WorkflowDefinition, WorkflowDraft, WorkflowVersion};
pub(crate) use execution::execute_step;
pub(crate) use personal_templates::{SaveWorkflowTemplateRequest, WorkflowTemplate};
pub(crate) use repository::load_version as load_workflow_version;
pub(crate) use runtime::start_with_run_id;
pub(crate) use runtime::StartWorkflowStepTestRequest;
pub(crate) use runtime::{start as start_workflow_internal, StartWorkflowRequest};
pub(crate) use templates::{BuiltInWorkflowTemplateId, InstantiateWorkflowTemplateRequest};

#[tauri::command]
pub async fn save_workflow_draft(
    app: tauri::AppHandle,
    draft: WorkflowDraft,
) -> Result<WorkflowDraft, String> {
    repository::save_draft(crate::spaces::db::pool(&app).await?, draft).await
}

#[tauri::command]
pub async fn validate_workflow(
    definition: WorkflowDefinition,
) -> Result<WorkflowExecutionPlan, Vec<WorkflowDiagnostic>> {
    compile(&definition)
}

#[tauri::command]
pub async fn preflight_workflow(
    app: tauri::AppHandle,
    definition: WorkflowDefinition,
) -> Result<Vec<readiness::WorkflowReadinessIssue>, String> {
    compile(&definition).map_err(|diagnostics| {
        serde_json::to_string(&diagnostics)
            .unwrap_or_else(|_| "Workflow definition is invalid".into())
    })?;
    readiness::inspect(&app, &definition).await
}

#[tauri::command]
pub async fn publish_workflow(
    app: tauri::AppHandle,
    workflow_id: String,
    expected_revision: i64,
) -> Result<WorkflowVersion, String> {
    repository::publish(
        crate::spaces::db::pool(&app).await?,
        &workflow_id,
        expected_revision,
    )
    .await
}

#[tauri::command]
pub async fn list_workflow_drafts(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Vec<WorkflowDraft>, String> {
    repository::list_drafts(crate::spaces::db::pool(&app).await?, &space_id).await
}

#[tauri::command]
pub async fn list_workflow_versions(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Vec<WorkflowVersion>, String> {
    repository::list_versions(crate::spaces::db::pool(&app).await?, &space_id).await
}

#[tauri::command]
pub async fn delete_workflow_draft(
    app: tauri::AppHandle,
    workflow_id: String,
) -> Result<(), String> {
    repository::delete_draft(crate::spaces::db::pool(&app).await?, &workflow_id).await
}

#[tauri::command]
pub async fn start_workflow(
    app: tauri::AppHandle,
    request: StartWorkflowRequest,
) -> Result<crate::spaces::runs::RunSummary, String> {
    start_workflow_internal(crate::spaces::db::pool(&app).await?, request).await
}

#[tauri::command]
pub async fn start_workflow_step_test(
    app: tauri::AppHandle,
    request: StartWorkflowStepTestRequest,
) -> Result<crate::spaces::runs::RunSummary, String> {
    runtime::start_step_test(crate::spaces::db::pool(&app).await?, request).await
}

#[tauri::command]
pub async fn instantiate_workflow_template(
    app: tauri::AppHandle,
    request: InstantiateWorkflowTemplateRequest,
) -> Result<WorkflowVersion, String> {
    templates::instantiate(crate::spaces::db::pool(&app).await?, request).await
}

#[tauri::command]
pub async fn extract_workflow_draft_from_history(
    app: tauri::AppHandle,
    space_id: String,
    title: String,
) -> Result<WorkflowDraft, String> {
    history_extraction::extract(crate::spaces::db::pool(&app).await?, &space_id, &title).await
}

#[tauri::command]
pub async fn save_workflow_template(
    app: tauri::AppHandle,
    request: personal_templates::SaveWorkflowTemplateRequest,
) -> Result<personal_templates::WorkflowTemplate, String> {
    personal_templates::save(crate::spaces::db::pool(&app).await?, request).await
}

#[tauri::command]
pub async fn list_workflow_templates(
    app: tauri::AppHandle,
) -> Result<Vec<personal_templates::WorkflowTemplate>, String> {
    personal_templates::list(crate::spaces::db::pool(&app).await?).await
}

#[tauri::command]
pub async fn delete_workflow_template(
    app: tauri::AppHandle,
    template_id: String,
) -> Result<(), String> {
    personal_templates::delete(crate::spaces::db::pool(&app).await?, &template_id).await
}

#[tauri::command]
pub async fn copy_workflow_template(
    app: tauri::AppHandle,
    template_id: String,
    space_id: String,
) -> Result<WorkflowDraft, String> {
    personal_templates::copy(
        crate::spaces::db::pool(&app).await?,
        &template_id,
        &space_id,
    )
    .await
}
