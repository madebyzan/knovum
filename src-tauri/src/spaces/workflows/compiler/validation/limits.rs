use super::{global_diagnostic, WorkflowDefinition, WorkflowDiagnostic};

pub(super) fn validate(definition: &WorkflowDefinition, diagnostics: &mut Vec<WorkflowDiagnostic>) {
    let limits = &definition.limits;
    if !(1..=10_000).contains(&limits.max_expanded_steps) {
        diagnostics.push(global_diagnostic(
            "expanded_step_limit",
            "Expanded step limit must be 1 to 10000",
        ));
    }
    if definition.nodes.len() > limits.max_expanded_steps as usize {
        diagnostics.push(global_diagnostic(
            "expanded_step_limit",
            "Definition nodes exceed the expanded Step budget",
        ));
    }
    if !(1..=64).contains(&limits.max_parallelism) {
        diagnostics.push(global_diagnostic(
            "parallelism_limit",
            "Parallelism must be 1 to 64",
        ));
    }
    if !(1..=604_800).contains(&limits.max_duration_seconds) {
        diagnostics.push(global_diagnostic(
            "duration_limit",
            "Duration must be 1 second to 7 days",
        ));
    }
    if !(1..=10_000_000).contains(&limits.max_total_tokens) {
        diagnostics.push(global_diagnostic(
            "token_limit",
            "Token budget must be 1 to 10000000",
        ));
    }
    if !(1..=100_000).contains(&limits.max_step_attempts) {
        diagnostics.push(global_diagnostic(
            "attempt_limit",
            "Step attempt budget must be 1 to 100000",
        ));
    }
    if !(1..=1_073_741_824).contains(&limits.max_input_bytes)
        || !(1..=1_073_741_824).contains(&limits.max_output_bytes)
    {
        diagnostics.push(global_diagnostic(
            "content_budget",
            "Input and output budgets must be 1 byte to 1 GiB",
        ));
    }
}
