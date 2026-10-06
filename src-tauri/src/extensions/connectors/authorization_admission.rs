use super::super::contracts::{ConnectorAuthorization, McpEndpoint};
use std::collections::BTreeMap;

pub(super) async fn admit(
    pool: &sqlx::SqlitePool,
    account: &super::super::contracts::ConnectorAccount,
    connector: &super::super::contracts::ConnectorDefinition,
    fields: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, String> {
    super::accounts::validate_identity(connector, account)?;
    let fields = prepare(
        &account.authorization,
        connector.configured_endpoint()?,
        fields,
    )?;
    super::accounts::begin_authorization(pool, account).await?;
    Ok(fields)
}

fn prepare(
    authorization: &ConnectorAuthorization,
    endpoint: &McpEndpoint,
    mut fields: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, String> {
    match authorization {
        ConnectorAuthorization::OAuth { .. } if !fields.is_empty() => {
            Err("oauth-credentials-must-not-be-provided-manually".into())
        }
        ConnectorAuthorization::None if !fields.is_empty() => {
            Err("unauthenticated-connector-does-not-accept-secrets".into())
        }
        ConnectorAuthorization::Bearer => {
            if fields.len() != 1 || !fields.contains_key("token") {
                return Err("connector-secret-fields-invalid".into());
            }
            let token = fields.remove("token").expect("validated token field");
            if token.is_empty() {
                return Err("connector-bearer-token-required".into());
            }
            let header = format!("Bearer {token}");
            reqwest::header::HeaderValue::from_str(&header)
                .map_err(|_| "connector-secret-header-value-invalid")?;
            Ok([("Authorization".into(), header)].into_iter().collect())
        }
        ConnectorAuthorization::SecretFields { names } => {
            if names.len() != fields.len()
                || names.iter().any(|name| {
                    fields
                        .get(name)
                        .is_none_or(|value| value.is_empty() || value.contains('\0'))
                })
            {
                return Err("connector-required-secret-fields-mismatch".into());
            }
            if matches!(endpoint, McpEndpoint::StreamableHttp { .. }) {
                for value in fields.values() {
                    reqwest::header::HeaderValue::from_str(value)
                        .map_err(|_| "connector-secret-header-value-invalid")?;
                }
            }
            Ok(fields)
        }
        _ => Ok(fields),
    }
}

#[cfg(test)]
#[path = "tests/authorization_admission.rs"]
mod tests;
