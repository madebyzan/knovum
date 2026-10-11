mod request;
pub(super) mod response;

use futures::future::BoxFuture;
use genai::{
    adapter::AdapterKind,
    resolver::{AuthData, Endpoint},
    Client, ModelIden, ServiceTarget,
};

use crate::ai::{
    contracts::{ModelTurnCapability, ModelTurnRequest, ModelTurnResponse},
    transport::SecurityContext,
};
use crate::foundation::app_error::{AppError, AppResult};

pub(crate) use request::{to_genai_options, to_genai_request};

pub struct GenAiModelTurnAdapter {
    adapter_kind: AdapterKind,
}

impl GenAiModelTurnAdapter {
    pub const fn new(adapter_kind: AdapterKind) -> Self {
        Self { adapter_kind }
    }
}

pub(super) fn service_target(adapter: AdapterKind, context: &SecurityContext) -> ServiceTarget {
    let auth = context
        .api_key
        .as_ref()
        .map_or(AuthData::None, |key| AuthData::from_single(key.clone()));
    ServiceTarget {
        endpoint: Endpoint::from_owned(format!("{}/", context.endpoint.trim_end_matches('/'))),
        auth,
        model: ModelIden::new(adapter, context.model.clone()),
    }
}

impl ModelTurnCapability for GenAiModelTurnAdapter {
    fn execute_turn<'a>(
        &'a self,
        context: &'a SecurityContext,
        request: &'a ModelTurnRequest,
    ) -> BoxFuture<'a, AppResult<ModelTurnResponse>> {
        Box::pin(async move {
            let response = Client::default()
                .exec_chat(
                    service_target(self.adapter_kind, context),
                    to_genai_request(request)?,
                    Some(&to_genai_options(request)),
                )
                .await
                .map_err(|error| AppError::new("provider_request_failed", error.to_string()))?;
            response::from_genai_response(context, response)
        })
    }
}
