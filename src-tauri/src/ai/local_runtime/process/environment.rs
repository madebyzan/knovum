use std::process::Command;

const PRIVATE_ENVIRONMENT: &[&str] = &[
    "PYTHONPATH",
    "PYTHONHOME",
    "PYTHONSTARTUP",
    "VOXCPM_MPS_DTYPE",
    "HF_TOKEN",
    "HUGGING_FACE_HUB_TOKEN",
    "ORT_DYLIB_PATH",
];

pub(crate) fn isolate_environment(command: &mut Command) {
    let overrides: Vec<_> = std::env::vars_os()
        .map(|(key, _)| key)
        .chain(command.get_envs().map(|(key, _)| key.to_owned()))
        .filter(|key| {
            let key = key.to_string_lossy().to_ascii_uppercase();
            ["DYLD_", "LD_", "GGML_", "VK_"]
                .iter()
                .any(|prefix| key.starts_with(prefix))
        })
        .collect();
    for name in overrides {
        command.env_remove(name);
    }
    for name in PRIVATE_ENVIRONMENT {
        command.env_remove(name);
    }
}

#[cfg(test)]
#[path = "tests/environment.rs"]
mod tests;
