use agent_client_protocol::schema::v1::{
    PermissionOptionKind, RequestPermissionOutcome, RequestPermissionRequest,
    RequestPermissionResponse, SelectedPermissionOutcome,
};

pub(super) fn allow_always(request: &RequestPermissionRequest) -> RequestPermissionResponse {
    let selected = request
        .options
        .iter()
        .find(|option| option.kind == PermissionOptionKind::AllowAlways);
    let outcome = selected.map_or(RequestPermissionOutcome::Cancelled, |option| {
        RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(option.option_id.clone()))
    });
    RequestPermissionResponse::new(outcome)
}
