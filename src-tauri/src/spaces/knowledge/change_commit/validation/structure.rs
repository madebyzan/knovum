use std::collections::HashSet;

use super::super::contracts::KnowledgeChange;
use crate::spaces::knowledge::{locator, provenance, relations, validation};

const MAX_CHANGE_SET_OPERATIONS: usize = 512;

pub(crate) fn validate_structure(operations: &[KnowledgeChange]) -> Result<(), String> {
    if operations.is_empty() {
        return Err("Knowledge Change Set must contain at least one operation".into());
    }
    if operations.len() > MAX_CHANGE_SET_OPERATIONS {
        return Err(format!(
            "Knowledge Change Set exceeds the maximum of {} operations",
            MAX_CHANGE_SET_OPERATIONS
        ));
    }
    let mut temp_ids = HashSet::new();
    for operation in operations {
        validate_operation(operation, &mut temp_ids)?;
    }
    Ok(())
}

fn validate_operation(
    operation: &KnowledgeChange,
    temp_ids: &mut HashSet<String>,
) -> Result<(), String> {
    match operation {
        KnowledgeChange::CreateObject {
            temp_id,
            title,
            status,
            position,
            ..
        } => {
            validate_temp_id(temp_ids, temp_id)?;
            if title.trim().is_empty() {
                return Err("Knowledge Change Set object title is required".into());
            }
            validation::validate_object_status(status)?;
            if position
                .as_ref()
                .is_some_and(|value| !value.x.is_finite() || !value.y.is_finite())
            {
                return Err("Knowledge Change Set object position must be finite".into());
            }
        }
        KnowledgeChange::UpdateObject { status, .. } => {
            if let Some(status) = status {
                validation::validate_object_status(status)?;
            }
        }
        KnowledgeChange::CreateRelation {
            temp_id,
            kind,
            rationale,
            confidence,
            ..
        } => {
            validate_temp_id(temp_ids, temp_id)?;
            relations::validate_kind(kind)?;
            relations::validate_rationale(rationale)?;
            relations::validate_confidence(*confidence)?;
        }
        KnowledgeChange::AddProvenance {
            target_kind,
            source_revision,
            locator: value,
            quote,
            ..
        } => {
            provenance::validation::validate_target_kind(target_kind)?;
            if source_revision.trim().is_empty() {
                return Err("Knowledge Change Set provenance source revision is required".into());
            }
            locator::validate(value, Some(source_revision))?;
            provenance::validation::validate_quote(quote)?;
        }
    }
    Ok(())
}

fn validate_temp_id(ids: &mut HashSet<String>, id: &str) -> Result<(), String> {
    if id.trim().is_empty() {
        return Err("Knowledge Change Set operation temp ID is required".into());
    }
    if !ids.insert(id.to_owned()) {
        return Err("Knowledge Change Set operation temp ID must be unique".into());
    }
    Ok(())
}
