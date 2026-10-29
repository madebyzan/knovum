//! What a proposal says, independent of how it says it.
//!
//! Two proposals that would record the same claims, relations and provenance have one
//! fingerprint, whatever temp ids, operation order, case or spacing they use. A Space keeps one
//! proposal per fingerprint, which is what stops the same suggestion from returning after it was
//! accepted or dismissed.

use std::collections::HashMap;

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::closure::temp_id;
use crate::spaces::knowledge::change_commit::KnowledgeChange;

pub(crate) fn fingerprint(changes: &[KnowledgeChange]) -> String {
    // Objects name nothing; relations name objects. Describing in that order reads every temp id
    // as what it creates.
    let mut created: HashMap<&str, String> = HashMap::new();
    for pass in [Created::Object, Created::Relation] {
        for change in changes
            .iter()
            .filter(|change| Created::of(change) == Some(pass))
        {
            let line = describe(change, &created);
            created.extend(temp_id(change).map(|id| (id, line)));
        }
    }
    let mut parts: Vec<String> = changes
        .iter()
        .map(|change| describe(change, &created))
        .collect();
    parts.sort_unstable();
    parts.dedup();
    format!(
        "sha256:{}",
        hex::encode(Sha256::digest(parts.join("\n").as_bytes()))
    )
}

#[derive(Clone, Copy, PartialEq)]
enum Created {
    Object,
    Relation,
}

impl Created {
    fn of(change: &KnowledgeChange) -> Option<Self> {
        match change {
            KnowledgeChange::CreateObject { .. } => Some(Self::Object),
            KnowledgeChange::CreateRelation { .. } => Some(Self::Relation),
            _ => None,
        }
    }
}

/// A canonical line for one operation; a reference to a created item reads as that item.
fn describe(change: &KnowledgeChange, created: &HashMap<&str, String>) -> String {
    let named = |id: &str| {
        created
            .get(id)
            .cloned()
            .unwrap_or_else(|| format!("node {id}"))
    };
    match change {
        KnowledgeChange::CreateObject {
            object_type_id,
            title,
            ..
        } => {
            format!("object {object_type_id} {}", normalize(title))
        }
        KnowledgeChange::UpdateObject {
            node_id,
            title,
            summary,
            status,
            properties,
        } => format!(
            "update {node_id} {} {} {} {}",
            title.as_deref().map(normalize).unwrap_or_default(),
            summary.as_deref().map(normalize).unwrap_or_default(),
            status.as_deref().unwrap_or_default(),
            properties.as_ref().map(canonical).unwrap_or_default(),
        ),
        KnowledgeChange::CreateRelation {
            source_id,
            target_id,
            kind,
            ..
        } => {
            format!(
                "relation {kind} ({}) ({})",
                named(source_id),
                named(target_id)
            )
        }
        KnowledgeChange::AddProvenance {
            target_id,
            source_node_id,
            source_revision,
            quote,
            ..
        } => format!(
            "provenance ({}) {source_node_id}@{source_revision} {}",
            named(target_id),
            normalize(quote)
        ),
    }
}

/// Case, whitespace and surrounding punctuation do not change what a statement says.
fn normalize(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

fn canonical(properties: &serde_json::Map<String, Value>) -> String {
    let mut keys: Vec<&String> = properties.keys().collect();
    keys.sort_unstable();
    keys.into_iter()
        .map(|key| format!("{key}={}", properties[key]))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
#[path = "tests/fingerprint.rs"]
mod tests;
