//! Verified answers: which sentences of an answer its sources actually state.
//!
//! When a conversation answer that cites passages is final, Knovum checks it in a Run of the
//! Application Action `verify-answer`: the answer is split into statements, a model judges them
//! against the passages the answer's Run read, and only quotes found in those passages count as
//! support. The person sees the result at the message and can record supported statements as
//! Claims through a Knowledge Proposal.

pub(crate) mod capture;
pub(crate) mod commands;
pub(crate) mod contracts;
mod judgment;
mod repository;
mod segmentation;

use serde_json::{json, Value};
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter};

use contracts::{VerificationSkip, VerificationState};
use repository::{Outcome, StoredAnswer};

use crate::{
    ai::{
        self,
        tasks::{
            completion::{ProductExecutor, TaskInput},
            structured,
        },
    },
    foundation::app_error::AppError,
    spaces::{
        actions::{self, ActionInvocationOutcome, ActionInvocationRequest},
        authorization::{create_grant, local_user_grant, RunScope},
        db,
        runs::AttemptClaim,
    },
};

const ACTION_ID: &str = "verify-answer";

/// Verifies every final answer of `run_id` that cites passages and is not verified yet. Called when
/// a conversation Run settles; a failure is logged, never surfaced into the conversation.
pub(crate) async fn request_for_run(app: &AppHandle, run_id: &str) {
    if let Err(error) = request_for_run_inner(app, run_id).await {
        log::warn!("Answer verification could not be requested for Run {run_id}: {error}");
    }
}

async fn request_for_run_inner(app: &AppHandle, run_id: &str) -> Result<(), AppError> {
    let pool = db::pool(app).await?;
    let answers =
        repository::cited_answers(&mut *pool.acquire().await.map_err(db::sql_error)?, run_id)
            .await?;
    for answer in answers {
        start(app, pool, &answer, Restart::No).await?;
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum Restart {
    No,
    Yes,
}

/// Starts the verification Run of one answer, or records why it cannot run.
async fn start(
    app: &AppHandle,
    pool: &SqlitePool,
    answer: &StoredAnswer,
    restart: Restart,
) -> Result<(), AppError> {
    let definition = actions::require(ACTION_ID)?;
    let invocation_id = uuid::Uuid::new_v4().to_string();
    let mut invocation = ActionInvocationRequest {
        invocation_id: invocation_id.clone(),
        idempotency_key: invocation_id.clone(),
        grant_id: invocation_id.clone(),
        space_id: answer.space_id.clone(),
        action_id: ACTION_ID.into(),
        action_version: definition.version,
        parameters: json!({ "messageId": answer.message_id }),
        input_snapshot: json!({ "targets": {} }),
        expected_revisions: Value::Null,
        parent_run_id: None,
    };
    let target = actions::resolve_target(app, &mut invocation);
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    match restart {
        Restart::Yes => repository::reset(&mut tx, &answer.message_id).await?,
        Restart::No if !repository::insert_pending(&mut tx, answer, now).await? => return Ok(()),
        Restart::No => {}
    }
    if target.is_err() {
        // Nothing can judge the answer until Settings assigns knowledge work to a model or Agent.
        repository::settle(
            &mut tx,
            &answer.message_id,
            None,
            skipped(VerificationSkip::NoTarget),
            now,
        )
        .await?;
    } else {
        let grant = local_user_grant(
            invocation_id,
            RunScope::Space {
                space_id: answer.space_id.clone(),
            },
            vec![ACTION_ID.into()],
            now,
        )?;
        create_grant(&mut tx, &grant, now).await?;
        let ActionInvocationOutcome::AcceptedRun { run_id } =
            actions::invoke_in_transaction(&mut tx, &invocation).await?
        else {
            return Err("action-executor-invalid: answer verification did not create a Run".into());
        };
        repository::assign_run(&mut tx, &answer.message_id, &run_id).await?;
    }
    tx.commit().await.map_err(db::sql_error)?;
    announce(app, answer);
    Ok(())
}

fn skipped(skip: VerificationSkip) -> Outcome<'static> {
    Outcome {
        state: VerificationState::Skipped,
        skip: Some(skip),
        statements: &[],
        verdicts: &[],
        truncated: false,
        error: None,
    }
}

/// Runs the `verify-answer` Action: judges the answer and records the verdicts. A failure is
/// recorded on the verification and fails the Run, so both the message and Task Center show it.
pub(crate) async fn execute(
    app: &AppHandle,
    claim: &AttemptClaim,
    parameters: &Value,
    snapshot: &Value,
) -> Result<Value, AppError> {
    let message_id = parameters
        .get("messageId")
        .and_then(Value::as_str)
        .ok_or("action-input-invalid: messageId is required")?;
    let pool = db::pool(app).await?;
    let answer = {
        let mut tx = db::begin_write(pool).await?;
        repository::start(&mut tx, message_id, &claim.run_id).await?;
        let answer = repository::load_answer(&mut tx, message_id).await?;
        tx.commit().await.map_err(db::sql_error)?;
        answer
    };
    announce(app, &answer);
    let judged = judge(app, pool, claim, &answer, snapshot).await;
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let result = match &judged {
        Ok(Judged::Skipped(skip)) => {
            repository::settle(
                &mut tx,
                message_id,
                Some(&claim.run_id),
                skipped(*skip),
                now,
            )
            .await
        }
        Ok(Judged::Completed {
            statements,
            verdicts,
            truncated,
        }) => {
            let outcome = Outcome {
                state: VerificationState::Completed,
                skip: None,
                statements,
                verdicts,
                truncated: *truncated,
                error: None,
            };
            repository::settle(&mut tx, message_id, Some(&claim.run_id), outcome, now).await
        }
        Err(error) => {
            let message = error.to_string();
            let outcome = Outcome {
                state: VerificationState::Failed,
                skip: None,
                statements: &[],
                verdicts: &[],
                truncated: false,
                error: Some(&message),
            };
            repository::settle(&mut tx, message_id, Some(&claim.run_id), outcome, now).await
        }
    };
    result?;
    tx.commit().await.map_err(db::sql_error)?;
    announce(app, &answer);
    let summary = judged?.summary();
    Ok(summary)
}

enum Judged {
    Skipped(VerificationSkip),
    Completed {
        statements: Vec<contracts::AnswerStatement>,
        verdicts: Vec<contracts::StatementVerdict>,
        truncated: bool,
    },
}

impl Judged {
    fn summary(self) -> Value {
        match self {
            Self::Skipped(skip) => json!({ "skip": skip }),
            Self::Completed {
                verdicts,
                truncated,
                ..
            } => json!({
                "statements": verdicts.len(),
                "truncated": truncated,
            }),
        }
    }
}

async fn judge(
    app: &AppHandle,
    pool: &SqlitePool,
    claim: &AttemptClaim,
    answer: &StoredAnswer,
    snapshot: &Value,
) -> Result<Judged, AppError> {
    let passages = repository::read_passages(
        &mut *pool.acquire().await.map_err(db::sql_error)?,
        &answer.run_id,
    )
    .await?;
    if passages.is_empty() {
        return Ok(Judged::Skipped(VerificationSkip::NoSources));
    }
    let segmentation = segmentation::segment(&answer.text);
    if segmentation.statements.is_empty() {
        return Ok(Judged::Skipped(VerificationSkip::NoStatements));
    }
    let target = ai::settings::resolve_task(
        app,
        actions::frozen_target(ACTION_ID, snapshot)?,
        ai::contracts::ExecutionCapability::TextGeneration,
    )?;
    let statements = segmentation.statements;
    let output = structured::execute_structured(
        &ProductExecutor(app),
        target,
        TaskInput {
            purpose: ai::usage::UsagePurpose::Knowledge,
            system_prompt: judgment::SYSTEM.into(),
            instruction: structured::schema_instruction::<judgment::Judgment>(),
            text: judgment::prompt(&statements, &passages),
            max_tokens: None,
            image: None,
        },
        |reply| judgment::parse(reply, &statements, &passages),
        claim.cancellation.clone(),
    )
    .await?;
    for usage in &output.usage {
        crate::spaces::runs::record_usage(
            pool,
            &claim.run_id,
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens,
        )
        .await?;
    }
    Ok(Judged::Completed {
        statements,
        verdicts: output.value,
        truncated: segmentation.truncated,
    })
}

/// Tells the conversation that a verification changed; it reads the verification itself.
fn announce(app: &AppHandle, answer: &StoredAnswer) {
    let update = json!({ "spaceId": answer.space_id, "messageId": answer.message_id });
    if let Err(error) = app.emit("space-answer-verification-update", update) {
        log::warn!("Could not publish answer verification update: {error}");
    }
}
