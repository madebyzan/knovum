use super::super::contracts::{ConnectorAuthorization, McpEndpoint};
use std::collections::BTreeSet;

pub(crate) fn validate(authorization: &ConnectorAuthorization) -> Result<(), String> {
    match authorization {
        ConnectorAuthorization::SecretFields { names } => {
            let mut unique = BTreeSet::new();
            if names.is_empty()
                || names.iter().any(|name| {
                    name.is_empty()
                        || name.contains(['=', '\0'])
                        || !unique.insert(name.to_ascii_lowercase())
                })
            {
                return Err("connector-secret-field-names-invalid".into());
            }
        }
        ConnectorAuthorization::OAuth { client_id, scopes }
            if client_id.as_ref().is_some_and(|id| id.trim().is_empty())
                || scopes
                    .iter()
                    .any(|scope| scope.is_empty() || scope.chars().any(char::is_whitespace)) =>
        {
            return Err("connector-public-oauth-requirements-invalid".into());
        }
        _ => {}
    }
    Ok(())
}

pub(in crate::extensions) fn validate_access(
    endpoint: &McpEndpoint,
    authorization: &ConnectorAuthorization,
) -> Result<(), String> {
    super::super::mcp::endpoint::validate(endpoint)?;
    validate(authorization)?;
    match (endpoint, authorization) {
        (
            McpEndpoint::Stdio { .. },
            ConnectorAuthorization::OAuth { .. } | ConnectorAuthorization::Bearer,
        ) => Err("connector-token-authorization-requires-http-endpoint".into()),
        (McpEndpoint::StreamableHttp { .. }, ConnectorAuthorization::SecretFields { names }) => {
            for name in names {
                reqwest::header::HeaderName::from_bytes(name.as_bytes())
                    .map_err(|_| "connector-secret-header-name-invalid")?;
                if super::super::mcp::endpoint::reserved_header(name) {
                    return Err("connector-cannot-override-mcp-transport-header".into());
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
