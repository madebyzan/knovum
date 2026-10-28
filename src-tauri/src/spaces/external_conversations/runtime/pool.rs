//! The desktop Agent runtimes kept for Space conversations. A runtime — one Agent process, one
//! ACP session, one Knovum MCP server and the workspace the Agent works in — serves every turn
//! of its conversation, so a follow-up costs one prompt instead of a launch, an initialisation
//! and a replay of the whole conversation.
//!
//! A runtime continues a turn only when it holds exactly the conversation the turn continues:
//! the same Agent, the same branch, the same extension selection (its MCP session offers that
//! catalogue), and the branch's last message is the one its own previous turn wrote. Anything
//! else — an edited or regenerated message, a built-in Agent reply in between, another branch —
//! means the Agent remembers a different conversation, so it is closed and a new one starts from
//! the stored history. Runtimes nobody talks to close after [`IDLE_TIMEOUT`].
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError},
    time::Duration,
};

use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    ai::contracts::ExternalAgentKind,
    extensions::contracts::AgentExtensionSnapshot,
    spaces::mcp::agent_server::{self, AgentMcpBinding},
};

const IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// The conversation a turn continues, as the runtime it runs on must hold it.
pub(super) struct Wanted<'a> {
    pub(super) space_id: &'a str,
    pub(super) conversation_id: &'a str,
    pub(super) branch_id: &'a str,
    pub(super) kind: ExternalAgentKind,
    /// The branch's last message before this turn; `None` for a conversation that has none.
    pub(super) head: Option<&'a str>,
    pub(super) extensions: &'a Arc<AgentExtensionSnapshot>,
}

/// What the Agent reported about its session when it was created.
pub(super) struct AgentSession {
    pub(super) runtime_session_id: String,
    pub(super) capabilities: serde_json::Value,
}

struct Runtime {
    id: String,
    conversation_id: String,
    kind: ExternalAgentKind,
    branch_id: String,
    digest: String,
    workspace: PathBuf,
    mcp: Arc<AgentMcpBinding>,
    session: AgentSession,
    state: Mutex<RuntimeState>,
}

struct RuntimeState {
    /// The branch's last message once the runtime's latest turn ended.
    head: Option<String>,
    turn: Option<Turn>,
    /// Bumped on every release, so an idle timer only closes the idleness it was started for.
    releases: u64,
}

struct Turn {
    binding_id: String,
    cancellation: CancellationToken,
    /// Cancelled when the turn's lease is released.
    ended: CancellationToken,
}

impl Runtime {
    fn state(&self) -> MutexGuard<'_, RuntimeState> {
        // Every write replaces fields whole, so a panic elsewhere never leaves the state torn.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn holds(&self, wanted: &Wanted<'_>) -> bool {
        self.kind == wanted.kind
            && self.branch_id == wanted.branch_id
            && self.digest == wanted.extensions.digest
            && self.state().head.as_deref() == wanted.head
            && crate::external_agents::conversation::is_active(&self.id)
    }
}

/// One turn's hold on a runtime. The runtime runs nothing else until the lease is dropped, and
/// dropping it — however the turn ended — ends the turn's MCP authority and starts the idle clock.
pub(super) struct Lease {
    runtime: Arc<Runtime>,
    /// The runtime already holds the conversation: the prompt must not replay it.
    pub(super) continues: bool,
    cancellation: CancellationToken,
    head: Option<String>,
}

impl Lease {
    pub(super) fn runtime_id(&self) -> &str {
        &self.runtime.id
    }

    pub(super) fn session(&self) -> &AgentSession {
        &self.runtime.session
    }

    pub(super) fn workspace(&self) -> &std::path::Path {
        &self.runtime.workspace
    }

    /// What stops this turn. Cancelling it stops the Agent's turn, never the runtime.
    pub(super) fn cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    pub(super) async fn authorize(
        &self,
        app: &AppHandle,
        binding_id: &str,
        run_id: &str,
        extensions: Arc<AgentExtensionSnapshot>,
    ) -> Result<(), String> {
        self.runtime
            .mcp
            .begin_turn(app, binding_id, run_id, extensions)
            .await
    }

    /// The Agent has seen this turn: the branch now ends with `head`.
    pub(super) fn settle(&mut self, head: Option<String>) {
        self.head = head;
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.runtime.mcp.end_turn();
        let releases = {
            let mut state = self.runtime.state();
            if let Some(head) = self.head.take() {
                state.head = Some(head);
            }
            if let Some(turn) = state.turn.take() {
                turn.ended.cancel();
            }
            state.releases += 1;
            state.releases
        };
        let runtime = self.runtime.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(IDLE_TIMEOUT).await;
            let idle = {
                let state = runtime.state();
                state.turn.is_none() && state.releases == releases
            };
            if idle {
                retire(&runtime.conversation_id, &runtime.id);
            }
        });
    }
}

static RUNTIMES: OnceLock<Mutex<HashMap<String, Arc<Runtime>>>> = OnceLock::new();

fn runtimes() -> MutexGuard<'static, HashMap<String, Arc<Runtime>>> {
    RUNTIMES
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// Leases the conversation's runtime for the turn `binding_id`, continuing the one that holds the
/// conversation or replacing it with a new one.
pub(super) async fn acquire(
    app: &AppHandle,
    wanted: Wanted<'_>,
    binding_id: &str,
) -> Result<Lease, String> {
    let current = runtimes().get(wanted.conversation_id).cloned();
    if let Some(runtime) = current {
        if runtime.state().turn.is_some() {
            return Err(
                "conversation-branch-busy: finish or cancel the active task before sending a follow-up"
                    .into(),
            );
        }
        if runtime.holds(&wanted) {
            return Ok(lease(runtime, binding_id, true));
        }
        retire(&runtime.conversation_id, &runtime.id);
    }
    let runtime = Arc::new(start(app, &wanted).await?);
    runtimes().insert(wanted.conversation_id.to_owned(), runtime.clone());
    Ok(lease(runtime, binding_id, false))
}

fn lease(runtime: Arc<Runtime>, binding_id: &str, continues: bool) -> Lease {
    let cancellation = CancellationToken::new();
    runtime.state().turn = Some(Turn {
        binding_id: binding_id.to_owned(),
        cancellation: cancellation.clone(),
        ended: CancellationToken::new(),
    });
    Lease {
        runtime,
        continues,
        cancellation,
        head: None,
    }
}

/// A new runtime starts from an empty workspace: what the Agent wrote for an earlier runtime was
/// published by its turns, and a new session does not know those files.
async fn start(app: &AppHandle, wanted: &Wanted<'_>) -> Result<Runtime, String> {
    let workspace = crate::spaces::agent_artifacts::workspace::desktop_agent_directory(
        app,
        wanted.space_id,
        wanted.conversation_id,
    )
    .map_err(|error| error.to_string())?;
    crate::external_agents::artifacts::prepare(&workspace).map_err(|error| error.to_string())?;
    let mcp = Arc::new(
        agent_server::start(
            app.clone(),
            wanted.space_id.to_owned(),
            wanted.extensions.clone(),
        )
        .await?,
    );
    let id = uuid::Uuid::new_v4().to_string();
    let keepalive: Arc<dyn Send + Sync> = mcp.clone();
    let session = crate::external_agents::conversation::start(
        id.clone(),
        wanted.kind,
        workspace.clone(),
        vec![mcp.server()],
        keepalive,
    )
    .await
    .map_err(|error| error.to_string())?;
    let conversation_id = wanted.conversation_id.to_owned();
    let completion = session.completion;
    tauri::async_runtime::spawn({
        let conversation_id = conversation_id.clone();
        let id = id.clone();
        async move {
            if let Ok(Err(error)) = completion.await {
                log::warn!("Desktop Agent runtime {id} ended: {error}");
            }
            forget(&conversation_id, &id);
        }
    });
    Ok(Runtime {
        id,
        conversation_id,
        kind: wanted.kind,
        branch_id: wanted.branch_id.to_owned(),
        digest: wanted.extensions.digest.clone(),
        workspace,
        mcp,
        session: AgentSession {
            runtime_session_id: session.runtime_session_id,
            capabilities: session.capabilities,
        },
        state: Mutex::new(RuntimeState {
            head: wanted.head.map(str::to_owned),
            turn: None,
            releases: 0,
        }),
    })
}

/// Stops the turn `binding_id` if it is running and resolves once its lease is released. That
/// is bounded: an Agent that does not stop a cancelled turn in time loses its runtime, which
/// ends the turn.
pub(in crate::spaces::external_conversations) async fn stop(binding_id: &str) {
    let ended = runtimes().values().find_map(|runtime| {
        let state = runtime.state();
        let turn = state.turn.as_ref()?;
        (turn.binding_id == binding_id).then(|| {
            turn.cancellation.cancel();
            turn.ended.clone()
        })
    });
    if let Some(ended) = ended {
        ended.cancelled().await;
    }
}

/// Whether the turn `binding_id` is running on some runtime.
pub(in crate::spaces::external_conversations) fn is_running(binding_id: &str) -> bool {
    runtimes().values().any(|runtime| {
        runtime
            .state()
            .turn
            .as_ref()
            .is_some_and(|turn| turn.binding_id == binding_id)
    })
}

/// Closes the conversation's runtime, if it has one.
pub(in crate::spaces::external_conversations) fn close(conversation_id: &str) {
    let current = runtimes()
        .get(conversation_id)
        .map(|runtime| runtime.id.clone());
    if let Some(id) = current {
        retire(conversation_id, &id);
    }
}

/// Closes every runtime and waits for them. A process that exits runs no destructors, so the
/// Agents — each its own process group — are stopped while the app can still wait for them.
pub(in crate::spaces::external_conversations) async fn shutdown() {
    let closing: Vec<String> = runtimes()
        .drain()
        .map(|(_, runtime)| runtime.id.clone())
        .collect();
    futures::future::join_all(closing.iter().map(|id| stop_process(id))).await;
}

/// Takes a runtime out of service at once and stops its process in the background: nothing
/// waits for an Agent that is being replaced or has gone idle.
fn retire(conversation_id: &str, runtime_id: &str) {
    forget(conversation_id, runtime_id);
    let runtime_id = runtime_id.to_owned();
    tauri::async_runtime::spawn(async move { stop_process(&runtime_id).await });
}

async fn stop_process(runtime_id: &str) {
    if crate::external_agents::conversation::is_active(runtime_id) {
        if let Err(error) = crate::external_agents::conversation::close(runtime_id).await {
            log::warn!("Desktop Agent runtime {runtime_id} did not close cleanly: {error}");
        }
    }
}

fn forget(conversation_id: &str, runtime_id: &str) {
    let mut runtimes = runtimes();
    if runtimes
        .get(conversation_id)
        .is_some_and(|runtime| runtime.id == runtime_id)
    {
        runtimes.remove(conversation_id);
    }
}
