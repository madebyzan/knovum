use super::isolate_environment;
use std::{ffi::OsStr, process::Command};

#[test]
fn native_environment_removes_private_python_and_provider_credentials() {
    let mut command = Command::new("unused-test-command");
    for name in ["PYTHONPATH", "HF_TOKEN", "ORT_DYLIB_PATH"] {
        command.env(name, "untrusted");
    }
    command.env("KNOVUM_ENV_SENTINEL", "preserved");
    isolate_environment(&mut command);
    for name in ["PYTHONPATH", "HF_TOKEN", "ORT_DYLIB_PATH"] {
        assert!(command
            .get_envs()
            .find(|(key, _)| *key == OsStr::new(name))
            .unwrap()
            .1
            .is_none());
    }
    assert_eq!(
        command
            .get_envs()
            .find(|(key, _)| *key == OsStr::new("KNOVUM_ENV_SENTINEL"))
            .unwrap()
            .1,
        Some(OsStr::new("preserved"))
    );
}

#[test]
fn native_environment_rejects_loader_and_backend_overrides_beyond_known_names() {
    let names = [
        "DYLD_VERSIONED_LIBRARY_PATH",
        "DYLD_IMAGE_SUFFIX",
        "LD_AUDIT",
        "GGML_METAL_PATH_RESOURCES",
        "VK_DRIVER_FILES",
        "VK_INSTANCE_LAYERS",
    ];
    let mut command = Command::new("unused-test-command");
    for name in names {
        command.env(name, "untrusted");
    }
    isolate_environment(&mut command);
    for name in names {
        assert!(
            command
                .get_envs()
                .find(|(key, _)| *key == OsStr::new(name))
                .unwrap()
                .1
                .is_none(),
            "Native loader override survived: {name}"
        );
    }
}
