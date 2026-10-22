mod build_support;

fn main() {
    for source in [
        "src/spaces/db/migrations",
        "src/extensions/migrations",
        "src/ai/usage/migrations",
        "src/secrets/migrations",
        "src/spaces/command_registry/declarations.rs",
    ] {
        println!("cargo:rerun-if-changed={source}");
    }
    println!("cargo:rerun-if-changed=../package.json");
    println!("cargo:rustc-check-cfg=cfg(knovum_supported_desktop)");
    let os = std::env::var("CARGO_CFG_TARGET_OS").expect("Cargo target OS");
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo target architecture");
    if matches!(
        (os.as_str(), arch.as_str()),
        ("macos", "aarch64" | "x86_64") | ("windows", "x86_64")
    ) {
        println!("cargo:rustc-cfg=knovum_supported_desktop");
    }
    build_support::media::configure();
    build_support::python_runtime::embed_manifest();
    build_support::inference_runtime::embed_manifest();
    let spaces_commands = build_support::spaces::commands();
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&[
                "get_update_state",
                "check_for_update",
                "download_update",
                "cancel_update_download",
                "install_update",
            ]))
            .plugin(
                "desktop",
                tauri_build::InlinedPlugin::new().commands(build_support::desktop::COMMANDS),
            )
            .plugin(
                "backup",
                tauri_build::InlinedPlugin::new().commands(&[
                    "await_maintenance",
                    "schedule_backup",
                    "schedule_restore",
                    "get_maintenance_receipt",
                    "acknowledge_maintenance_receipt",
                ]),
            )
            .plugin(
                "dictation",
                tauri_build::InlinedPlugin::new().commands(&[
                    "get_state",
                    "cancel",
                    "dismiss_transcription",
                    "copy_transcription",
                ]),
            )
            .plugin(
                "local-resources",
                tauri_build::InlinedPlugin::new().commands(&[
                    "get_status",
                    "get_model_catalog",
                    "remove_model",
                    "select_local_model",
                    "prepare_resources",
                    "cancel_resource_preparation",
                    "list_resource_preparations",
                ]),
            )
            .plugin(
                "secrets",
                tauri_build::InlinedPlugin::new().commands(&["get", "inspect", "set", "delete"]),
            )
            .plugin(
                "ai",
                tauri_build::InlinedPlugin::new().commands(&[
                    "await_ai_runtime",
                    "cancel_job",
                    "detect_external_agent_installations",
                    "list_external_agent_descriptors",
                    "list_provider_descriptors",
                    "transform_editor_text_stream",
                    "translate_text_stream",
                    "validate_model_settings",
                    "get_token_usage_report",
                ]),
            )
            .plugin(
                "extensions",
                tauri_build::InlinedPlugin::new()
                    .commands(&[
                        "inspect_plugin_source",
                        "cancel_plugin_operation",
                        "inspect_plugin_update",
                        "install_plugin",
                        "update_plugin",
                        "list_plugin_installations",
                        "enable_plugin",
                        "disable_plugin",
                        "uninstall_plugin",
                        "create_plugin_marketplace",
                        "refresh_plugin_marketplace",
                        "delete_plugin_marketplace",
                        "search_plugins",
                        "list_skills",
                        "read_skill",
                        "read_skill_file",
                        "create_skill",
                        "save_skill_revision",
                        "import_skill",
                        "export_skill",
                        "duplicate_skill",
                        "enable_skill",
                        "disable_skill",
                        "delete_skill",
                        "read_mcp_resource",
                        "get_mcp_prompt",
                        "create_mcp_server",
                        "create_mcp_connection",
                        "update_mcp_connection",
                        "list_mcp_connections",
                        "check_mcp_connection",
                        "delete_mcp_connection",
                        "list_connector_accounts",
                        "list_connector_definitions",
                        "update_connector_definition",
                        "delete_connector_definition",
                        "configure_plugin_connector",
                        "connect_connector",
                        "cancel_connector_authorization",
                        "disconnect_connector",
                        "get_agent_extension_availability",
                    ])
                    .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
            )
            .plugin(
                "spaces",
                tauri_build::InlinedPlugin::new()
                    .commands(spaces_commands)
                    .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
            ),
    )
    .expect("failed to build Tauri application manifest");

    // tauri-build gives only binaries the Common-Controls v6 manifest (rustc-link-arg-bins), so the
    // knovum_lib unit-test binary binds comctl32 v5.82 and dies at startup with 0xc0000139 on rfd's
    // TaskDialogIndirect import. No cargo link-arg scope reaches that lib test binary without also
    // reaching the binaries, where a second copy of tauri's resource.lib fails to link (CVT1100
    // duplicate VERSION resource). Delay-loading comctl32 is a duplicate-safe flag that defers the
    // v6-only import until a dialog is shown, which unit tests never do; binaries keep their
    // manifest, so the shipped app still resolves comctl32 v6 on first use. /DELAYLOAD resolves
    // imports through __delayLoadHelper2, which only delayimp.lib provides.
    if os == "windows" {
        println!("cargo:rustc-link-arg=/DELAYLOAD:comctl32.dll");
        println!("cargo:rustc-link-arg=/DEFAULTLIB:delayimp.lib");
    }
}
