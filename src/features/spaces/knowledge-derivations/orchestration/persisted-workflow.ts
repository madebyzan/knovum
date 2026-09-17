import { useTaskStore } from "../../../../foundation/tasks/task-store";
import type { CreationWorkflowDefinition, CreationWorkflowRunSnapshot } from "../workflows/workflow";
import { executeCreationWorkflow, type CreationStepAdapters } from "./workflow-executor";
import { saveCreationWorkflowRun } from "./workflow-run-repository";

interface PersistedWorkflowOptions<Action extends string, Output> {
  spaceId: string;
  workflow: CreationWorkflowDefinition<Action>;
  adapters: CreationStepAdapters<Action, Output>;
  onChange?: (snapshot: CreationWorkflowRunSnapshot) => void;
  task: {
    title: string;
    detail?: string;
    id?: string;
  };
}

export async function executePersistedWorkflow<Action extends string, Output>(
  options: PersistedWorkflowOptions<Action, Output>,
) {
  const tasks = useTaskStore();
  const taskId = options.task.id ?? tasks.start(options.task.title, options.task.detail);
  try {
    const snapshot = await executeCreationWorkflow({
      runId: crypto.randomUUID(),
      spaceId: options.spaceId,
      workflow: options.workflow,
      adapters: options.adapters,
      onChange: async (current) => {
        await saveCreationWorkflowRun(options.spaceId, current);
        const completed = current.steps.filter(({ status }) => status === "succeeded").length;
        tasks.progress(taskId, (completed / current.steps.length) * 100);
        options.onChange?.(current);
      },
    });
    tasks.succeed(taskId);
    return snapshot;
  } catch (error) {
    tasks.fail(taskId, String(error));
    return undefined;
  }
}
