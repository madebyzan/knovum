import { decodeAppError } from "../../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { useTaskStore } from "../../../../foundation/tasks/task-store";
import { cancelKnowledgeIndexing, indexKnowledgeRetrieval } from "../persistence/retrieval-index";

export interface KnowledgeIndexingHandle {
  /** Resolves with the chunks this pass encoded, or rejects — cancellation included. */
  completion: Promise<{ indexed: number }>;
  cancel: () => Promise<void>;
}

/**
 * Indexes a Space as a Task Center task: one entry with progress, one cancel button, outliving the
 * panel that started it.
 *
 * This is the only way the application asks for indexing on a person's behalf. Indexing picks the
 * smallest build that can make the Space searchable — the gaps of the active generation are filled,
 * and every chunk is re-encoded only when no generation can be reused — and the live index keeps
 * answering searches until the new generation is complete. Cancelling therefore costs the batch in
 * flight, never the Space's search, and the next pass resumes from the committed markers.
 */
export function startKnowledgeIndexing(
  spaceId: string,
  title: string,
  nodeIds?: readonly string[],
): KnowledgeIndexingHandle {
  const tasks = useTaskStore();
  const jobId = crypto.randomUUID();
  const cancel = () => cancelKnowledgeIndexing(jobId);
  tasks.start(title, undefined, cancel, jobId);
  return { completion: run(tasks, jobId, spaceId, nodeIds), cancel };
}

export function isKnowledgeIndexingCancelled(error: unknown): boolean {
  return decodeAppError(error)?.code === "cancelled";
}

async function run(
  tasks: ReturnType<typeof useTaskStore>,
  jobId: string,
  spaceId: string,
  nodeIds: readonly string[] | undefined,
): Promise<{ indexed: number }> {
  try {
    const output = await indexKnowledgeRetrieval({
      spaceId,
      jobId,
      nodeIds,
      onProgress: ({ indexed, total }) => {
        // The pass reports what it measured. A Space written to while it indexes feeds the same
        // pass more chunks, so the ratio is bounded here rather than in the reported number.
        if (total > 0) tasks.progress(jobId, Math.min(100, Math.round((indexed / total) * 100)));
        else tasks.indeterminate(jobId);
      },
    });
    tasks.succeed(jobId);
    return output;
  } catch (error) {
    if (isKnowledgeIndexingCancelled(error)) tasks.cancelled(jobId);
    else tasks.fail(jobId, getErrorMessage(error));
    throw error;
  }
}
