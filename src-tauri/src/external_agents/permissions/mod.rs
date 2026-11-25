use std::sync::Arc;

use agent_client_protocol::{
    schema::v1::{
        NewSessionResponse, PermissionOptionKind, RequestPermissionOutcome,
        RequestPermissionRequest, RequestPermissionResponse, SelectedPermissionOutcome,
        SetSessionModeRequest,
    },
    Error,
};
use futures::future::BoxFuture;
use tokio_util::sync::CancellationToken;

/// Asynchronous user-approval callback used when a Desktop Agent is not in Trusted mode.
pub type PermissionApprover = Arc<
    dyn Fn(
            RequestPermissionRequest,
            CancellationToken,
        ) -> BoxFuture<'static, Result<RequestPermissionResponse, Error>>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub enum PermissionPolicy {
    /// Explicit user opt-in: switch adapter to its full-access mode and auto-approve requests.
    Trusted,
    /// Default for interactive Space conversations: keep safe mode and prompt the user.
    Interactive(PermissionApprover),
    /// Default for non-interactive background tasks: keep safe mode and reject elevated requests.
    DenyElevated,
}

pub(super) async fn authorize(
    request: &RequestPermissionRequest,
    policy: &PermissionPolicy,
    cancellation: &CancellationToken,
) -> Result<RequestPermissionResponse, Error> {
    if cancellation.is_cancelled() {
        return Ok(RequestPermissionResponse::new(
            RequestPermissionOutcome::Cancelled,
        ));
    }
    match policy {
        PermissionPolicy::Trusted => select_allow(request),
        PermissionPolicy::DenyElevated => Ok(select_reject_or_cancel(request)),
        PermissionPolicy::Interactive(approver) => {
            approver(request.clone(), cancellation.clone()).await
        }
    }
}

fn select_allow(request: &RequestPermissionRequest) -> Result<RequestPermissionResponse, Error> {
    let option = request
        .options
        .iter()
        .find(|option| option.kind == PermissionOptionKind::AllowAlways)
        .or_else(|| {
            request
                .options
                .iter()
                .find(|option| option.kind == PermissionOptionKind::AllowOnce)
        })
        .ok_or_else(|| {
            Error::invalid_request()
                .data("Desktop Agent permission request provides no allow option")
        })?;
    Ok(RequestPermissionResponse::new(
        RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
            option.option_id.clone(),
        )),
    ))
}

pub fn select_reject_or_cancel(request: &RequestPermissionRequest) -> RequestPermissionResponse {
    if let Some(option) = request
        .options
        .iter()
        .find(|option| option.kind == PermissionOptionKind::RejectOnce)
        .or_else(|| {
            request
                .options
                .iter()
                .find(|option| option.kind == PermissionOptionKind::RejectAlways)
        })
    {
        return RequestPermissionResponse::new(RequestPermissionOutcome::Selected(
            SelectedPermissionOutcome::new(option.option_id.clone()),
        ));
    }
    RequestPermissionResponse::new(RequestPermissionOutcome::Cancelled)
}

pub(super) fn trusted_mode_request(
    mode_id: Option<&str>,
    response: &NewSessionResponse,
    policy: &PermissionPolicy,
) -> Result<Option<SetSessionModeRequest>, Error> {
    if !matches!(policy, PermissionPolicy::Trusted) {
        return Ok(None);
    }
    let Some(mode_id) = mode_id else {
        return Ok(None);
    };
    let modes = response.modes.as_ref().ok_or_else(|| {
        Error::invalid_request()
            .data("Desktop Agent does not advertise its required full-access session mode")
    })?;
    let mode = modes
        .available_modes
        .iter()
        .find(|mode| mode.id.0.as_ref() == mode_id)
        .ok_or_else(|| {
            Error::invalid_request().data("Desktop Agent full-access session mode is unavailable")
        })?;
    if modes.current_mode_id == mode.id {
        return Ok(None);
    }
    Ok(Some(SetSessionModeRequest::new(
        response.session_id.clone(),
        mode.id.clone(),
    )))
}

#[cfg(test)]
mod tests;
