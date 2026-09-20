# Exact-source native check tail

# Native compile and test-type check

Source: `9b1a5aba096de51ed2bf377ef8bbea9c5b2e9ad1`. Exit code: 101. This is a compiler check, not a desktop or runtime test pass.

```text
    Checking lance-namespace-impls v10.0.0
   Compiling knovum v1.0.0-beta4 (/home/runner/work/knovum/knovum/src-tauri)
    Checking tract-onnx v0.23.5
    Checking tauri-plugin-dialog v2.7.2
    Checking genai v0.6.5
    Checking sqlx v0.8.6
    Checking tauri-plugin-clipboard v2.1.11
    Checking tauri-plugin-autostart v2.5.1
    Checking tauri-plugin-opener v2.5.4
    Checking typst-pdf v0.15.1
    Checking typst-as-lib v0.16.0
    Checking pdf-inspector v1.20.0
    Checking keyring v4.1.6
    Checking candle-nn v0.11.0
    Checking office_oxide v0.1.8
    Checking tauri-plugin-store v2.4.4
    Checking tauri-plugin-window-state v2.4.1
    Checking docx-rs v0.4.22
    Checking tauri-plugin-single-instance v2.4.3
    Checking zip v4.6.1
    Checking lancedb v0.37.1
src/spaces/canvas_state/mod.rs:83:27: error[E0364]: `valid_id` is private, and cannot be re-exported
src/spaces/organization/draft.rs:10:42: error[E0603]: function import `valid_structure_id` is private: private function import
src/file_inspection/mod.rs:8:41: warning: unused import: `is_sensitive_file`
src/spaces/organization/mod.rs:20:44: warning: unused imports: `EditorCommand`, `FrameUpdate`, and `TextMeasurement`
src/spaces/organization/mod.rs:21:27: warning: unused import: `editor_plan::plan_editor`
src/spaces/canvas_state/mod.rs:17:61: warning: unused import: `load_in_transaction`
src/spaces/previews/thumbnail.rs:5:5: warning: unused import: `crate::spaces::assets::enrichment::ensure_active`
src/spaces/previews/thumbnail.rs:7:5: warning: unused import: `super::paths`
src/system_preview/thumbnail/mod.rs:1:5: warning: unused import: `std::path::Path`
src/ai/providers/agents/discovery.rs:82:13: error[E0282]: type annotations needed for `HashSet<_>`
src/desktop/mod.rs:120:16: error[E0599]: no method named `app_handle` found for reference `&tauri::Window<R>` in the current scope: private field, not a method
src/spaces/canvas_state/tests/mod.rs:72:9: error[E0560]: struct `CommitCanvasGestureRequest` has no field named `changes`: `CommitCanvasGestureRequest` does not have this field
warning: `knovum` (lib) generated 7 warnings
error: could not compile `knovum` (lib) due to 4 previous errors; 7 warnings emitted
warning: build failed, waiting for other jobs to finish...
warning: `knovum` (lib test) generated 6 warnings (6 duplicates)
error: could not compile `knovum` (lib test) due to 5 previous errors; 6 warnings emitted

```
