# Background tasks

The task foundation represents user-visible long-running work and maps running progress to the OS taskbar/Dock indicator.

For Rust work that streams progress, use Tauri `Channel` rather than global events. The foundation exports `runTrackedTask` to connect a feature-owned `invoke` call to Task Center state:

```ts
await runTrackedTask(tasks, "Clone repository", (events) => invoke("git_clone", { repository, events }));
```

The Rust command owns the operation and sends `TaskProgressEvent` messages through the channel. The feature still owns cancellation semantics because cancellation differs by engine and operation; Foundation does not provide a universal job runner.

Use events for low-volume broadcasts. Use channels for ordered/high-throughput task streams.

Spaces persists Assistant and captured-workflow execution as Runs. On opening the Spaces
workspace, each feature lists its active Runs and observes their state through a Tauri Channel.
Task Center projects those Run IDs, states and cancellation handlers; it does not own execution.
Assistant task titles in Task Center open the owning Space and conversation, including after a
WebView reload. Closing the conversation panel leaves the Rust Run active. Approval and Agent
permission decisions remain in that conversation, while Task Center shows the global summary.
