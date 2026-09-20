# Native compile and test-type check

Source: `12fc032945b7f77bb1219108439acf1deffc7820`. Exit code: 101. This is a compiler check, not a desktop or runtime test pass.

```text
   Compiling knovum v1.0.0-beta4 (/home/runner/work/knovum/knovum/src-tauri)
src/spaces/canvas_state/mod.rs:85:27: error[E0364]: `valid_id` is private, and cannot be re-exported
src/spaces/editing/receipts.rs:6:38: error[E0603]: function import `valid_structure_id` is private: private function import
src/spaces/organization/draft.rs:10:42: error[E0603]: function import `valid_structure_id` is private: private function import
src/file_inspection/mod.rs:8:41: warning: unused import: `is_sensitive_file`
src/spaces/organization/mod.rs:20:59: warning: unused import: `FrameUpdate`
src/spaces/previews/thumbnail.rs:5:5: warning: unused import: `crate::spaces::assets::enrichment::ensure_active`
src/spaces/previews/thumbnail.rs:7:5: warning: unused import: `super::paths`
src/system_preview/thumbnail/mod.rs:1:5: warning: unused import: `std::path::Path`
src/ai/providers/agents/discovery.rs:82:13: error[E0282]: type annotations needed for `HashSet<_>`
src/desktop/mod.rs:120:16: error[E0599]: no method named `app_handle` found for reference `&tauri::Window<R>` in the current scope: private field, not a method
src/spaces/canvas_state/tests/mod.rs:72:9: error[E0560]: struct `CommitCanvasGestureRequest` has no field named `changes`: `CommitCanvasGestureRequest` does not have this field
warning: `knovum` (lib) generated 5 warnings
error: could not compile `knovum` (lib) due to 5 previous errors; 5 warnings emitted
warning: build failed, waiting for other jobs to finish...
warning: `knovum` (lib test) generated 4 warnings (4 duplicates)
error: could not compile `knovum` (lib test) due to 6 previous errors; 4 warnings emitted

```
