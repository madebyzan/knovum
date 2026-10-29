//! Knowledge Proposals: Change Sets waiting for a person to accept them.
//!
//! Analysis never writes the Knowledge graph. It proposes a Change Set with the evidence it rests
//! on; accepting it runs the same atomic commit every Knowledge change uses.

use serde::{Deserialize, Serialize};

use crate::spaces::knowledge::{
    change_commit::KnowledgeChange, grounding::evidence_span::EvidenceSpan,
};

/// Where a proposal came from, so review can show it and navigate back.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum ProposalOrigin {
    /// Claims recorded from a verified conversation answer.
    Answer {
        conversation_id: String,
        message_id: String,
    },
}

impl ProposalOrigin {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::Answer { .. } => "answer",
        }
    }
}

/// A proposal before it is stored.
pub(crate) struct ProposalDraft {
    pub origin: ProposalOrigin,
    /// Why the change is proposed, when the origin alone does not say it. May be empty.
    pub rationale: String,
    pub changes: Vec<KnowledgeChange>,
    /// Every span a change rests on; the proposal is current while they all are.
    pub basis: Vec<EvidenceSpan>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum ProposalState {
    Pending,
    Accepted,
    Dismissed,
}

impl ProposalState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Dismissed => "dismissed",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "dismissed" => Ok(Self::Dismissed),
            _ => Err(format!("Unknown Knowledge Proposal state: {value}")),
        }
    }
}

/// Whether a pending proposal still means what it meant. Computed on read, never stored.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum ProposalFreshness {
    Current,
    /// A source it quotes has changed since; accepting would record outdated evidence.
    BasisChanged,
    /// An existing node it changes or relates to is gone.
    TargetGone,
}

/// One operation as review shows it: what it needs, so only closed subsets can be chosen.
#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct ProposedChange {
    pub change: KnowledgeChange,
    /// Indexes of the operations this one cannot be accepted without.
    pub requires: Vec<usize>,
}

/// An existing node a proposal names, with the title review shows for it.
#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct ProposalNode {
    pub id: String,
    pub title: String,
}

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct KnowledgeProposal {
    pub id: String,
    pub space_id: String,
    pub origin: ProposalOrigin,
    pub rationale: String,
    pub changes: Vec<ProposedChange>,
    pub basis: Vec<EvidenceSpan>,
    pub nodes: Vec<ProposalNode>,
    pub state: ProposalState,
    pub freshness: ProposalFreshness,
    pub created_at: i64,
}

/// What proposing returned: the stored proposal, which is the earlier one when the Space already
/// holds a proposal with the same meaning.
#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct ProposalReceipt {
    pub proposal_id: String,
    pub state: ProposalState,
}

/// What accepting committed: the node each created temp id became.
#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct ProposalAcceptance {
    pub proposal_id: String,
    #[ts(type = "Record<string, string>")]
    pub resolved_ids: serde_json::Value,
}
