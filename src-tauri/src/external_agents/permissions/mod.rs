use agent_client_protocol::{
    schema::v1::{
        NewSessionResponse, PermissionOptionKind, RequestPermissionOutcome,
        RequestPermissionRequest, RequestPermissionResponse, SelectedPermissionOutcome,
        SetSessionModeRequest,
    },
    Error,
};
use tokio_util::sync::CancellationToken;

pub(super) fn authorize(
    request: &RequestPermissionRequest,
    cancellation: &CancellationToken,
) -> Result<RequestPermissionResponse, Error> {
    if cancellation.is_cancelled() {
        return Ok(RequestPermissionResponse::new(
            RequestPermissionOutcome::Cancelled,
        ));
    }
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

pub(super) fn trusted_mode_request(
    mode_id: Option<&str>,
    response: &NewSessionResponse,
) -> Result<Option<SetSessionModeRequest>, Error> {
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
