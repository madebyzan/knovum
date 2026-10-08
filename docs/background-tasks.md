# Background tasks

Task Center is a user-visible projection of long-running work and drives OS taskbar/Dock progress. It does not execute jobs, persist durable Run state or provide a universal runner.

## Feature-owned progress and cancellation

Spaces enrichment runs in Rust independently of an open Space view. Manual transcription sends a
node identity through [media/transcription.ts](../src/features/spaces/media/transcription.ts) to
`plugin:spaces|request_media_transcription`; automatic intent enters the same durable job queue.
Rust owns the source revision, resource preparation, concurrency lanes, cancellation and publication.
Before local model execution, the owning enrichment job or Run obtains a shared Rust compute permit.
The permit covers model loading and the active calculation, but it does not create another durable
job state. Cancelling first revokes the owner Attempt's right to publish, then notifies the private
Python worker; the permit is released only after that calculation ends or its process tree exits.
Interactive requests take priority over queued background indexing and synthesis, while aging
prevents background work from waiting indefinitely.

Images and video covers use one durable `analyze_visual` job and one Vision execution pipeline.
Node kind determines input preparation: images use the edited rendering and asset authorization;
video covers use the saved preview and its origin-specific authorization. Their visual input revision
guards scheduling, retry and publication. Replacing or removing a cover invalidates existing visual
work without invalidating media analysis or transcription. Cover analysis describes a still image;
it does not claim to understand the video timeline.

[enrichment/use-enrichment-job-observation.ts](../src/features/spaces/enrichment/use-enrichment-job-observation.ts) observes
`space-enrichment-update` and restores persisted actionable jobs. It projects progress, partial
transcripts and terminal outcomes into Task Center without dispatching or sequencing work. Retry
submits `retry_enrichment_job`; acceptance of that control intent does not mark the observed task as
succeeded. The execution owner publishes its terminal outcome. Cancel submits `cancel_enrichment_job`.

Explicit media insights use a Tauri Channel through
[media/insights.ts](../src/features/spaces/media/insights.ts). Rust owns prompts, chunking, AI routing,
chapter validation and guarded publication; [media/actions.ts](../src/features/spaces/media/actions.ts)
owns only the interaction and Task Center projection. A concurrent transcript or chapter edit rejects
publication rather than replacing the user's changes.

Foundation's [task-store.ts](../src/foundation/tasks/task-store.ts) owns presentation state,
cancellation/open handlers, retry presentation and history trimming. Feature executions own terminal
outcomes and cleanup. Required failures remain visible. Asset transfer UI may bound process-local
submission; durable enrichment concurrency and media inference limits belong to Rust. Channels carry
ordered progress within an execution; events publish enrichment observations to the WebView.

## Durable Run projection

Spaces persists Builtin Agent and Workflow execution as Runs. [runs/run-observation.ts](../src/features/spaces/runs/run-observation.ts) observes `Channel<RunObservation>` via `plugin:spaces|observe_run`. Conversation and Workflow owners project the observed states and cancellation handlers into Task Center and restore active Runs after reload. Closing the conversation panel does not end the Rust Run.

Speech projects use the same durable Run authority. One accepted start owns a stable request ID;
segment synthesis and final assembly receive separate attempt identities; audiobooks additionally
checkpoint chapter PCM and loudness measurement in independent steps. Every claim carries
the current execution epoch. Segment completion, output reservation and publication use epoch-aware
compare-and-swap writes, so a recovered or cancelled worker cannot publish late output. Startup
reconciles interrupted attempts and resumes only durable work. Audio normalization, video stretching
and assembly artifacts remain attempt-owned until final publication; Task Center observes these
phases but does not decide retry or success.

Task Center titles open the owning Space/conversation. Approval and external-Agent permission decisions stay at that owner; Task Center shows their global summary. Run IDs, revision guards and recovery remain Rust-owned. See [system-architecture.md](system-architecture.md) for runtime flow and [architecture.md](architecture.md) for dependency rules.
