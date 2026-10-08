use serde::{Deserialize, Serialize};

pub(super) const LOCAL_USER_GRANT_TTL_MS: i64 = 7 * 24 * 60 * 60 * 1_000;
pub(super) const EXTERNAL_AGENT_GRANT_TTL_MS: i64 = 8 * 60 * 60 * 1_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    User,
    BuiltinAgent,
    ExternalAgent,
}

impl PrincipalKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::BuiltinAgent => "builtin_agent",
            Self::ExternalAgent => "external_agent",
        }
    }

    pub fn event_actor_kind(self) -> &'static str {
        match self {
            Self::User => "human",
            Self::BuiltinAgent => "builtin_agent",
            Self::ExternalAgent => "external_agent",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Principal {
    pub kind: PrincipalKind,
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuthorizationContext {
    pub principal: Principal,
    pub grant_id: String,
    pub home_space_id: String,
    pub read_space_ids: Vec<String>,
    pub write_space_ids: Vec<String>,
    pub action_ids: Vec<String>,
    pub valid_until: i64,
}

pub(crate) fn local_user_grant(
    grant_id: String,
    home_space_id: String,
    action_ids: Vec<String>,
    now: i64,
) -> AuthorizationContext {
    AuthorizationContext {
        principal: Principal {
            kind: PrincipalKind::User,
            id: "local-user".into(),
        },
        grant_id,
        read_space_ids: vec![home_space_id.clone()],
        write_space_ids: vec![home_space_id.clone()],
        home_space_id,
        action_ids,
        valid_until: now + LOCAL_USER_GRANT_TTL_MS,
    }
}

pub(crate) fn external_agent_grant(
    grant_id: String,
    home_space_id: String,
    actor_id: String,
    action_ids: Vec<String>,
    now: i64,
) -> AuthorizationContext {
    AuthorizationContext {
        principal: Principal {
            kind: PrincipalKind::ExternalAgent,
            id: actor_id,
        },
        grant_id,
        read_space_ids: vec![home_space_id.clone()],
        write_space_ids: vec![home_space_id.clone()],
        home_space_id,
        action_ids,
        valid_until: now + EXTERNAL_AGENT_GRANT_TTL_MS,
    }
}

impl AuthorizationContext {
    pub fn require_read(&self, space_id: &str, now: i64) -> Result<(), String> {
        self.require_active(now)?;
        if self
            .read_space_ids
            .iter()
            .any(|candidate| candidate == space_id)
        {
            Ok(())
        } else {
            Err("permission-denied: Space read scope is not granted".into())
        }
    }

    pub fn require_action(&self, space_id: &str, action_id: &str, now: i64) -> Result<(), String> {
        self.require_active(now)?;
        if !self
            .write_space_ids
            .iter()
            .any(|candidate| candidate == space_id)
        {
            return Err("permission-denied: Space write scope is not granted".into());
        }
        if !self
            .action_ids
            .iter()
            .any(|candidate| candidate == action_id)
        {
            return Err("permission-denied: SpaceAction is not granted".into());
        }
        Ok(())
    }

    fn require_active(&self, now: i64) -> Result<(), String> {
        if self.valid_until < now {
            Err("grant-expired: authorization grant has expired".into())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "tests/scope.rs"]
mod tests;
