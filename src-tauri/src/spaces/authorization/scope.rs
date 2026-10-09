use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub(super) const LOCAL_USER_GRANT_TTL_MS: i64 = 7 * 24 * 60 * 60 * 1_000;
pub(super) const EXTERNAL_AGENT_GRANT_TTL_MS: i64 = 8 * 60 * 60 * 1_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrincipalKind {
    User,
    BuiltinAgent,
    ExternalAgent,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/RunScope.ts")]
pub enum RunScope {
    Space {
        #[serde(rename = "spaceId")]
        space_id: String,
    },
    Application,
}

impl RunScope {
    pub fn require_space_id(&self) -> Result<&str, String> {
        match self {
            Self::Space { space_id } => Ok(space_id),
            Self::Application => Err("permission-denied: Run requires Space scope".into()),
        }
    }

    pub fn require_application(&self) -> Result<(), String> {
        match self {
            Self::Application => Ok(()),
            Self::Space { .. } => Err("permission-denied: Run requires application scope".into()),
        }
    }

    pub(crate) fn db_values(&self) -> (&'static str, Option<&str>) {
        match self {
            Self::Space { space_id } => ("space", Some(space_id)),
            Self::Application => ("application", None),
        }
    }

    pub(crate) fn from_db(scope_kind: &str, home_space_id: Option<String>) -> Result<Self, String> {
        match (scope_kind, home_space_id) {
            ("space", Some(space_id)) if !space_id.trim().is_empty() => {
                Ok(Self::Space { space_id })
            }
            ("application", None) => Ok(Self::Application),
            _ => Err("scope-invalid: invalid Run scope database values".into()),
        }
    }
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
    pub scope: RunScope,
    pub read_space_ids: Vec<String>,
    pub write_space_ids: Vec<String>,
    pub action_ids: Vec<String>,
    pub valid_until: i64,
}

pub(crate) fn local_user_grant(
    grant_id: String,
    scope: RunScope,
    action_ids: Vec<String>,
    now: i64,
) -> Result<AuthorizationContext, String> {
    let (read_space_ids, write_space_ids, action_ids) = match &scope {
        RunScope::Space { space_id } => {
            (vec![space_id.clone()], vec![space_id.clone()], action_ids)
        }
        RunScope::Application if action_ids.is_empty() => (Vec::new(), Vec::new(), Vec::new()),
        RunScope::Application => {
            return Err(
                "permission-denied: application grants cannot authorize Space actions".into(),
            )
        }
    };
    Ok(AuthorizationContext {
        principal: Principal {
            kind: PrincipalKind::User,
            id: "local-user".into(),
        },
        grant_id,
        scope,
        read_space_ids,
        write_space_ids,
        action_ids,
        valid_until: now + LOCAL_USER_GRANT_TTL_MS,
    })
}

pub(crate) fn external_agent_grant(
    grant_id: String,
    space_id: String,
    actor_id: String,
    action_ids: Vec<String>,
    now: i64,
) -> AuthorizationContext {
    let scope = RunScope::Space {
        space_id: space_id.clone(),
    };
    AuthorizationContext {
        principal: Principal {
            kind: PrincipalKind::ExternalAgent,
            id: actor_id,
        },
        grant_id,
        read_space_ids: vec![space_id.clone()],
        write_space_ids: vec![space_id],
        scope,
        action_ids,
        valid_until: now + EXTERNAL_AGENT_GRANT_TTL_MS,
    }
}

impl AuthorizationContext {
    pub fn require_read(&self, space_id: &str, now: i64) -> Result<(), String> {
        self.require_active(now)?;
        if self.scope.require_space_id()? != space_id {
            return Err("permission-denied: authorization grant belongs to another Space".into());
        }
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
        if self.scope.require_space_id()? != space_id {
            return Err("permission-denied: authorization grant belongs to another Space".into());
        }
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

    pub fn require_application(&self, now: i64) -> Result<(), String> {
        self.require_active(now)?;
        self.scope.require_application()?;
        if self.principal.kind == PrincipalKind::User
            && self.read_space_ids.is_empty()
            && self.write_space_ids.is_empty()
            && self.action_ids.is_empty()
        {
            Ok(())
        } else {
            Err("permission-denied: application grant is invalid".into())
        }
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
