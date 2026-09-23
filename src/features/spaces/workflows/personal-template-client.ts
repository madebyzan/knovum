import { invoke } from "@tauri-apps/api/core";

import type { WorkflowDefinition, WorkflowDraft } from "./workflow-client";

export interface WorkflowTemplate {
  id: string;
  name: string;
  definition: WorkflowDefinition;
  dependencies: Array<{
    workflowId: string;
    version: number;
    definition: WorkflowDefinition;
  }>;
  sourceWorkflowId: string | null;
  sourceWorkflowVersion: number | null;
  createdAt: number;
  updatedAt: number;
}

export function listWorkflowTemplates(): Promise<WorkflowTemplate[]> {
  return invoke("plugin:spaces|list_workflow_templates");
}

export function saveWorkflowTemplate(draft: WorkflowDraft): Promise<WorkflowTemplate> {
  return invoke("plugin:spaces|save_workflow_template", {
    request: {
      id: crypto.randomUUID(),
      name: draft.definition.title,
      definition: draft.definition,
      sourceWorkflowId: draft.id,
      sourceWorkflowVersion: null,
    },
  });
}

export function copyWorkflowTemplate(templateId: string, spaceId: string): Promise<WorkflowDraft> {
  return invoke("plugin:spaces|copy_workflow_template", { templateId, spaceId });
}

export function deleteWorkflowTemplate(templateId: string): Promise<void> {
  return invoke("plugin:spaces|delete_workflow_template", { templateId });
}
