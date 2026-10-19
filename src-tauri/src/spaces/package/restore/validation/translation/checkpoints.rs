use super::{field, integer};
use crate::spaces::package::restore::contracts::{PackageRun, PackageRunStep};
use serde_json::Value;

pub(super) fn validate_steps(run: &PackageRun, steps: &[&PackageRunStep]) -> Result<(), String> {
    let segments = run
        .input_snapshot
        .get("segments")
        .and_then(Value::as_array)
        .ok_or("Package video translation Run segments are unavailable")?;
    if steps.len() != segments.len() {
        return Err("Package video translation checkpoints are incomplete".into());
    }
    let mut ordinals = std::collections::HashSet::with_capacity(steps.len());
    for step in steps {
        let output = step
            .output
            .as_ref()
            .ok_or("Package video translation checkpoint output is unavailable")?;
        let ordinal = step
            .configuration
            .get("ordinal")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .filter(|ordinal| *ordinal < segments.len())
            .ok_or("Package video translation checkpoint ordinal is invalid")?;
        let segment = step
            .configuration
            .get("segment")
            .ok_or("Package video translation checkpoint segment is unavailable")?;
        if !ordinals.insert(ordinal)
            || step.state != "succeeded"
            || field(output, "stepId")? != step.id
            || field(&step.configuration, "projectId")? != field(&run.input_snapshot, "projectId")?
            || field(&step.configuration, "revisionId")?
                != field(&run.input_snapshot, "revisionId")?
            || integer(&step.configuration, "expectedRevision")?
                != integer(&run.input_snapshot, "expectedRevision")?
            || field(&step.configuration, "inputRevision")?
                != field(&run.input_snapshot, "inputRevision")?
            || field(&step.configuration, "targetLanguage")?
                != field(&run.input_snapshot, "targetLanguage")?
            || field(&step.configuration, "provider")? != field(&run.input_snapshot, "provider")?
            || field(&step.configuration, "model")? != field(&run.input_snapshot, "model")?
            || field(&step.configuration, "baseUrl")? != field(&run.input_snapshot, "baseUrl")?
            || step.configuration.get("executionTarget")
                != run.input_snapshot.get("executionTarget")
            || segment != &segments[ordinal]
            || output.get("sourceSegmentId") != segment.get("sourceSegmentId")
            || output.get("sourceSegmentRevision") != segment.get("sourceSegmentRevision")
            || output.get("sourceText") != segment.get("text")
        {
            return Err("Package video translation checkpoint identity is invalid".into());
        }
    }
    Ok(())
}
