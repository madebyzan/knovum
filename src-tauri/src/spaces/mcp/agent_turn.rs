use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rmcp::ErrorData as McpError;

use crate::extensions::contracts::AgentExtensionSnapshot;

/// What one turn of a desktop Agent acts for: the grant its calls are checked against, the Run
/// they are recorded under, and the extensions selected for it.
pub(crate) struct AgentTurn {
    pub(crate) grant_id: String,
    pub(crate) run_id: String,
    pub(crate) extensions: Arc<AgentExtensionSnapshot>,
}

/// The turns an Agent's MCP session serves. The session lives as long as the Agent's runtime;
/// each turn is bound before its prompt and unbound when it ends, and nothing is answered for a
/// turn that is not running. What the session offers — its tools, resources and prompts — is
/// the extension selection it was started with: a runtime is only continued by a turn that
/// selected the same extensions.
#[derive(Clone)]
pub(crate) struct AgentTurns(Arc<Mutex<Turns>>);

struct Turns {
    current: Option<Arc<AgentTurn>>,
    catalog: Arc<AgentExtensionSnapshot>,
}

impl AgentTurns {
    pub(crate) fn new(catalog: Arc<AgentExtensionSnapshot>) -> Self {
        Self(Arc::new(Mutex::new(Turns {
            current: None,
            catalog,
        })))
    }

    pub(crate) fn begin(&self, turn: AgentTurn) -> Result<(), String> {
        let mut turns = self.lock();
        if turns.catalog.digest != turn.extensions.digest {
            return Err(
                "agent-turn-extensions-changed: the Agent session offers other extensions".into(),
            );
        }
        turns.catalog = turn.extensions.clone();
        turns.current = Some(Arc::new(turn));
        Ok(())
    }

    pub(crate) fn end(&self) {
        self.lock().current = None;
    }

    pub(super) fn current(&self) -> Result<Arc<AgentTurn>, McpError> {
        self.lock().current.clone().ok_or_else(|| {
            McpError::invalid_request(
                "This Agent turn has ended; Knovum tools answer only during a turn",
                None,
            )
        })
    }

    pub(super) fn catalog(&self) -> Arc<AgentExtensionSnapshot> {
        self.lock().catalog.clone()
    }

    /// Every write replaces one field whole, so a panic elsewhere never leaves the state torn.
    fn lock(&self) -> MutexGuard<'_, Turns> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
