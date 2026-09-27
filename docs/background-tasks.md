# Background tasks

Task Center is a user-visible projection of long-running work and drives OS taskbar/Dock progress. It does not execute jobs, persist durable Run state or provide a universal runner.

## Feature-owned progress and cancellation

Rust operations stream ordered progress through Tauri Channels. [media/transcription.ts](../src/features/spaces/media/transcription.ts) creates a `Channel<SpaceMediaTranscriptionProgress>` and passes it as `onProgress` to `plugin:spaces|transcribe_media`. The caller owns the job ID and cancellation request. [media/actions.ts](../src/features/spaces/media/actions.ts) manages Task Center progress, source-revision validation and the cancellation code.

The following uses current APIs and illustrates the progress bridge; the full feature implementation also guards queued cancellation and validates the media source before publishing results.

```ts
import { useTaskStore } from "../src/foundation/tasks/task-store";
import {
  cancelSpaceMediaTranscription,
  isMediaTranscriptionCancellation,
  transcribeSpaceMedia,
} from "../src/features/spaces/media/transcription";

const tasks = useTaskStore();
const jobId = crypto.randomUUID();
const taskId = tasks.start("Transcribe media", undefined, () => cancelSpaceMediaTranscription(jobId));
try {
  await transcribeSpaceMedia(jobId, sourcePath, ({ progress, stage }) => {
    tasks.progress(taskId, progress, stage);
  });
  tasks.succeed(taskId);
} catch (error) {
  if (isMediaTranscriptionCancellation(error)) tasks.dismiss(taskId);
  else tasks.fail(taskId, String(error));
}
```

`sourcePath` is the feature's validated input; product titles and stages are localized by the real caller. Foundation's [task-store.ts](../src/foundation/tasks/task-store.ts) owns presentation state, cancellation/open handlers, retry presentation and history trimming. The feature owns execution, terminal outcomes and cleanup. Required failures remain visible.

Assets own process-local transfer/processing concurrency budgets; media owns inference concurrency. These Renderer budgets limit submission and are not Rust durable schedulers. Low-volume broadcasts may use events; ordered task progress uses Channels.

## Durable Run projection

Spaces persists Assistant and Workflow execution as Runs. [runs/run-observation.ts](../src/features/spaces/runs/run-observation.ts) observes `Channel<RunSummary>` via `plugin:spaces|observe_run`. Conversation and Workflow owners project the observed states and cancellation handlers into Task Center and restore active Runs after reload. Closing the conversation panel does not end the Rust Run.

Task Center titles open the owning Space/conversation. Approval and external-Agent permission decisions stay at that owner; Task Center shows their global summary. Run IDs, revision guards and recovery remain Rust-owned. See [system-architecture.md](system-architecture.md) for runtime flow and [architecture.md](architecture.md) for dependency rules.
