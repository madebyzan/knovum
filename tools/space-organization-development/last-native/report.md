# Native compile and test-type check

Source: `338a721ed95500572ba8288605180a0e93e2d122`. Exit code: 0. This is a compiler check, not a desktop or runtime test pass.

```text
   Compiling knovum v1.0.0-beta4 (/home/runner/work/knovum/knovum/src-tauri)
src/file_inspection/mod.rs:8:41: warning: unused import: `is_sensitive_file`
src/spaces/organization/mod.rs:19:63: warning: unused import: `FragmentPlacement`
src/spaces/organization/mod.rs:24:59: warning: unused import: `FrameUpdate`
src/spaces/previews/thumbnail.rs:5:5: warning: unused import: `crate::spaces::assets::enrichment::ensure_active`
src/spaces/previews/thumbnail.rs:7:5: warning: unused import: `super::paths`
src/system_preview/thumbnail/mod.rs:1:5: warning: unused import: `std::path::Path`
src/ai/providers/agents/catalog.rs:24:9: warning: field `desktop_application` is never read
src/ai/providers/agents/discovery.rs:8:7: warning: constant `SHELL_PATH_TIMEOUT` is never used
src/ai/providers/agents/discovery.rs:41:4: warning: function `extract_delimited` is never used
src/documents/pdf_input.rs:54:19: warning: method `region` is never used
src/documents/rendering/mod.rs:36:4: warning: function `viewport` is never used
src/documents/rendering/mod.rs:62:4: warning: function `dimensions` is never used
src/spaces/canvas_state/navigation.rs:8:32: warning: function `ensure_existing_space` is never used
src/spaces/previews/paths.rs:19:8: warning: function `preview_path` is never used
src/system_preview/thumbnail/mod.rs:30:4: warning: function `ensure_not_cancelled` is never used
src/system_preview/thumbnail/policy.rs:6:15: warning: function `supports` is never used
warning: `knovum` (lib) generated 15 warnings (run `cargo fix --lib -p knovum` to apply 6 suggestions)
warning: `knovum` (lib test) generated 15 warnings (14 duplicates)
    Finished `dev` profile [unoptimized] target(s) in 30.45s

```
