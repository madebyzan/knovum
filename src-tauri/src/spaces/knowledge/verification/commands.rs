//! IPC commands of answer verification.

use tauri::AppHandle;

use super::{
    capture::{self, VerifiedAnswer},
    contracts::{AnswerVerification, StatementSupport, SupportCorrection, VerificationState},
    repository, start, Restart,
};
use crate::{
    foundation::app_error::AppError,
    spaces::{
        db,
        knowledge::proposals::{self, ProposalReceipt, Proposer},
    },
};

/// The verification of an answer, or nothing when the answer was never verified.
#[tauri::command]
pub(crate) async fn load_answer_verification(
    app: AppHandle,
    message_id: String,
) -> Result<Option<AnswerVerification>, AppError> {
    let pool = db::pool(&app).await?;
    Ok(repository::load(
        &mut *pool.acquire().await.map_err(db::sql_error)?,
        &message_id,
    )
    .await?)
}

/// Verifies the answer now: for the first time, or again after a failure, a skip or a change of
/// Settings. A verification in progress is left to finish.
#[tauri::command]
pub(crate) async fn verify_answer(app: AppHandle, message_id: String) -> Result<(), AppError> {
    let pool = db::pool(&app).await?;
    let (answer, existing) = {
        let mut connection = pool.acquire().await.map_err(db::sql_error)?;
        let answer = repository::load_answer(&mut connection, &message_id).await?;
        (
            answer,
            repository::load(&mut connection, &message_id).await?,
        )
    };
    let restart = match existing.map(|verification| verification.state) {
        None => Restart::No,
        Some(VerificationState::Pending | VerificationState::Running) => return Ok(()),
        Some(_) => Restart::Yes,
    };
    start(&app, pool, &answer, restart).await
}

/// Records the person's verdict on one statement. It wins over the model's wherever the
/// verification is shown or used.
#[tauri::command]
pub(crate) async fn correct_statement_support(
    app: AppHandle,
    message_id: String,
    statement: u32,
    support: StatementSupport,
) -> Result<(), AppError> {
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    repository::correct(
        &mut tx,
        &message_id,
        &SupportCorrection { statement, support },
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(())
}

/// Proposes the chosen statements of a verified answer as Claims with their evidence.
#[tauri::command]
pub(crate) async fn record_answer_claims(
    app: AppHandle,
    message_id: String,
    statements: Vec<u32>,
) -> Result<ProposalReceipt, AppError> {
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let answer = repository::load_answer(&mut tx, &message_id).await?;
    let verification = repository::load(&mut tx, &message_id)
        .await?
        .filter(|verification| verification.state == VerificationState::Completed)
        .ok_or("answer-capture-unverified: the answer has not been verified")?;
    let draft = capture::claims_proposal(
        &VerifiedAnswer {
            conversation_id: &answer.conversation_id,
            message_id: &message_id,
            statements: &verification.statements,
            verdicts: &verification.verdicts,
            corrections: &verification.corrections,
        },
        &statements,
    )?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let receipt = proposals::propose(
        &mut tx,
        &answer.space_id,
        &Proposer {
            kind: "system",
            id: None,
        },
        draft,
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    proposals::announce(&app, &answer.space_id);
    Ok(receipt)
}
