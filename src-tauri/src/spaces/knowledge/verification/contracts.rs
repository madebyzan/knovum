//! The verification of one stored answer: which of its sentences the sources actually say.

use serde::{Deserialize, Serialize};

use crate::spaces::knowledge::grounding::evidence_span::EvidenceSpan;

/// One checkable sentence of an answer, located in the answer's Markdown source.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct AnswerStatement {
    pub id: u32,
    /// The sentence as plain text, citation markers removed.
    pub text: String,
    /// UTF-8 byte range in the answer Markdown.
    pub start: usize,
    pub end: usize,
    /// Ordinals of the `[Cn]` markers that cite this sentence.
    pub citations: Vec<u32>,
}

/// What kind of sentence it is. Only factual sentences can be supported or not.
#[derive(
    Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, schemars::JsonSchema, ts_rs::TS,
)]
#[serde(rename_all = "kebab-case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum StatementRole {
    /// Asserts something about the world the sources could confirm.
    Factual,
    /// The answer's own reasoning or conclusion drawn from what it said before.
    Inference,
    /// Transitions, restatements of the question, offers of further help.
    Framing,
}

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, schemars::JsonSchema, ts_rs::TS,
)]
#[serde(rename_all = "kebab-case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum StatementSupport {
    /// A passage states it.
    Supported,
    /// A passage states part of it; the rest goes beyond the sources.
    Partial,
    /// It cites sources, and none of them states it.
    Unsupported,
    /// A factual sentence without a citation.
    Uncited,
    /// Inference or framing: not a claim about the sources.
    NotCheckable,
}

/// A passage that bears on a statement: the citation it was read as, and the exact quote.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct StatementEvidence {
    pub citation: u32,
    pub span: EvidenceSpan,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct StatementVerdict {
    pub statement: u32,
    pub role: StatementRole,
    pub support: StatementSupport,
    /// The quotes that support the statement; empty unless it is supported or partial.
    pub evidence: Vec<StatementEvidence>,
    /// For an uncited statement: passages the Run read that state it.
    pub possible_sources: Vec<StatementEvidence>,
    /// Why the verdict is what it is, in the answer's language. Bounded, never empty for a gap.
    pub reason: String,
}

/// A person's verdict on a statement. It wins over the model's on display.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SupportCorrection {
    pub statement: u32,
    pub support: StatementSupport,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum VerificationState {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl VerificationState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

/// Why an answer was not verified. Each is shown as such; none is an error.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum VerificationSkip {
    /// The answer read no indexed passage, so there is nothing to check it against.
    NoSources,
    /// The answer has no factual sentence.
    NoStatements,
    /// No AI execution method is configured for knowledge work.
    NoTarget,
}

impl VerificationSkip {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::NoSources => "no-sources",
            Self::NoStatements => "no-statements",
            Self::NoTarget => "no-target",
        }
    }
}

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct AnswerVerification {
    pub message_id: String,
    pub state: VerificationState,
    pub skip: Option<VerificationSkip>,
    pub statements: Vec<AnswerStatement>,
    pub verdicts: Vec<StatementVerdict>,
    pub corrections: Vec<SupportCorrection>,
    /// The answer was longer than verification reads; only its beginning was checked.
    pub truncated: bool,
    pub error: Option<String>,
    pub completed_at: Option<i64>,
}
