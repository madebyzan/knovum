# Background tasks

The task foundation represents user-visible long-running work and maps running progress to the OS taskbar/Dock indicator.

For Rust work that streams progress, use Tauri `Channel` rather than global events. The foundation exports `runTrackedTask` to connect a feature-owned `invoke` call to Task Center state:

```ts
await runTrackedTask(tasks, "Clone repository", (events) => invoke("git_clone", { repository, events }));
```

The Rust command owns the operation and sends `TaskProgressEvent` messages through the channel. The feature still owns cancellation semantics because cancellation differs by engine and operation; Foundation does not provide a universal job runner.

Use events for low-volume broadcasts. Use channels for ordered/high-throughput task streams.
