use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    User,
    Assistant,
    ExternalAgent,
}

impl PrincipalKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::ExternalAgent => "external_agent",
        }
    }

    pub fn event_actor_kind(self) -> &'static str {
        match self {
            Self::User => "human",
            Self::Assistant => "assistant",
            Self::ExternalAgent => "agent",
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
