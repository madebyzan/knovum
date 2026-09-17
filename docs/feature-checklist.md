# Feature start checklist

Use this before writing a real feature.

## 1. Define ownership

- user outcome and feature boundary;
- local vs shared state;
- whether it must run while the WebView is hidden;
- whether it needs its own sidebar/inspector/multi-window behavior.

## 2. OSS benchmark

Search Tauri/Vue first-party options, GitHub, npm and/or crates.io. Record maintained candidates and exact selection/rejection reasons. Use stable maintained OSS by default.

## 3. Data/security

- validate external data with Zod or Rust types at the boundary;
- Store for preferences, SQLite for structured local data, and the foundation keyring commands for secrets;
- add only the exact Tauri capability scopes the feature uses;
- never grant arbitrary filesystem, Shell or network access for convenience.

## 4. UI

- reuse `shared/ui` and Reka behavior;
- use semantic Tailwind v4 desktop tokens (`text-label`, `bg-canvas`, `h-(--knovum-control-height-md)`, etc.); do not reintroduce raw token variables or arbitrary pixel typography;
- provide keyboard/focus behavior and OS-correct shortcuts;
- use TanStack Virtual/Table for high-volume lists/grids;
- no fake controls or webpage/dashboard visual patterns.

## 5. Long work

Use the shared Task Center for presentation. Keep durable background work in Rust and use Tauri Channels for high-frequency streaming.

## 6. Quality

Add focused tests, then run:

```bash
pnpm check:web
pnpm check:rust
```

Do not solve a gate by disabling it globally or by introducing a compatibility fallback.
