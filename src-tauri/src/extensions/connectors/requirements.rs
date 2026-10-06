use super::super::contracts::ConnectorAuthorization;
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
