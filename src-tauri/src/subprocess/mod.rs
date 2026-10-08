use std::process::ExitStatus;

pub(crate) struct SubprocessOutput {
    pub status: ExitStatus,
    pub diagnostics: Vec<String>,
}

pub fn diagnostic_error(lines: &[String], default_message: &str) -> String {
    let message = lines.join("\n").trim().to_owned();
    if message.is_empty() {
        default_message.to_owned()
    } else {
        message
    }
}

mod runner;
pub(crate) use runner::{wait_with_output, wait_with_stderr};

#[cfg(test)]
mod tests;

pub(crate) mod process_tree;
pub(crate) mod python;
#[cfg(windows)]
pub(crate) mod windows_job;
