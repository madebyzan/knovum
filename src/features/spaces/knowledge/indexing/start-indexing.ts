import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { useTaskStore } from "../../../../foundation/tasks/task-store";
import { cancelKnowledgeIndexing, indexKnowledgeRetrieval } from "../persistence/retrieval-index";
import { isKnowledgeIndexingCancelled } from "./indexing-outcome";

export interface KnowledgeIndexingHandle {
  /** Resolves with the chunks this pass encoded, or rejects — cancellation included. */
  completion: Promise<{ indexed: number }>;
  cancel: () => Promise<void>;
}

/** One pass per Space, so everyone who asks for it in this window gets that one pass. */
const passBySpaceId = new Map<string, KnowledgeIndexingHandle>();

/**
 * Indexes a Space as a Task Center task: the Space's id names the pass, the task and the handle,
 * so a second surface asking for the same Space joins the pass that is running instead of opening
 * a second task that the Rust side would decline anyway.
 *
 * This is the only way the application asks for indexing on a person's behalf. Indexing picks the
 * smallest build that can make the Space searchable — the gaps of the active generation are filled,
 * and every chunk is re-encoded only when no generation can be reused — and the live index keeps
 * answering searches until the new generation is complete. Cancelling therefore costs the batch in
 * flight, never the Space's search, and the next pass resumes from the committed markers.
 */
export function startKnowledgeIndexing(spaceId: string, title: string): KnowledgeIndexingHandle {
  const running = passBySpaceId.get(spaceId);
  if (running) return running;
  const tasks = useTaskStore();
  const taskId = knowledgeIndexingTaskId(spaceId);
  const cancel = () => cancelKnowledgeIndexing(spaceId);
  tasks.start(title, undefined, cancel, taskId);
  const handle: KnowledgeIndexingHandle = {
    completion: run(tasks, taskId, spaceId).finally(() => passBySpaceId.delete(spaceId)),
    cancel,
  };
  passBySpaceId.set(spaceId, handle);
  return handle;
}

export function knowledgeIndexingTaskId(spaceId: string): string {
  return `knowledge-index:${spaceId}`;
}

async function run(
  tasks: ReturnType<typeof useTaskStore>,
  taskId: string,
  spaceId: string,
): Promise<{ indexed: number }> {
  try {
    const output = await indexKnowledgeRetrieval({
      spaceId,
      onProgress: ({ indexed, total }) => {
        // The pass reports what it measured. A Space written to while it indexes feeds the same
        // pass more chunks, so the ratio is bounded here rather than in the reported number.
        if (total > 0) tasks.progress(taskId, Math.min(100, Math.round((indexed / total) * 100)));
        else tasks.indeterminate(taskId);
      },
    });
    tasks.succeed(taskId);
    return output;
  } catch (error) {
    if (isKnowledgeIndexingCancelled(error)) tasks.cancelled(taskId);
    else tasks.fail(taskId, getErrorMessage(error));
    throw error;
  }
}
