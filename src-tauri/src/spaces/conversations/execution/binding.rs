use crate::{
    agent_runtime::protocol::models::AgentModelBinding,
    ai::{contracts::ChatModelProtocol, models::ModelExecutionTarget},
};

pub(in crate::spaces::conversations) async fn resolve(
    target: &ModelExecutionTarget,
    profile: &crate::ai::settings::contracts::BuiltinAgentModelSettings,
) -> Result<AgentModelBinding, String> {
    let protocol = profile.protocol;
    let security = crate::ai::transport::SecurityContext::from_target(target)
        .await
        .map_err(|error| error.to_string())?;
    let mut base_url = security.endpoint;
    let mut api_version = None;
    if matches!(protocol, ChatModelProtocol::Gemini) {
        let mut url =
            reqwest::Url::parse(&base_url).map_err(|_| "builtin-agent-endpoint-invalid")?;
        let version = url.path().trim_matches('/');
        if !matches!(version, "v1" | "v1beta") {
            return Err(
                "builtin-agent-endpoint-invalid: Gemini endpoint must declare v1 or v1beta".into(),
            );
        }
        api_version = Some(version.into());
        url.set_path("");
        base_url = url.to_string();
    }
    Ok(AgentModelBinding {
        protocol,
        model: target.model.clone(),
        base_url,
        api_version,
        api_key: security.api_key,
        temperature: Some(target.temperature),
        max_output_tokens: profile.max_output_tokens,
        max_input_tokens: profile.context_window,
    })
}
