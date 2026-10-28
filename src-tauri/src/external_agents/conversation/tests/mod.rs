use super::*;
use agent_client_protocol::{Channel, RawJsonRpcMessage, TransportFrame};
use futures::StreamExt;
use serde_json::json;
use std::time::Duration;

async fn respond(
    peer: &mut Channel,
    method: &str,
    outcome: serde_json::Value,
) -> serde_json::Value {
    let request = request(peer, method).await;
    answer(peer, &request, outcome);
    request
}

async fn initialized_peer(peer: &mut Channel) {
    respond(
        peer,
        "initialize",
        json!({"protocolVersion":1,"agentCapabilities":{}}),
    )
    .await;
    respond(peer, "session/new", json!({"sessionId":"session"})).await;
}

#[tokio::test]
async fn authorized_http_mcp_binding_is_supplied_to_the_acp_session() {
    tokio::time::timeout(Duration::from_secs(3), async {
        let (transport, mut peer) = Channel::duplex();
        let (requests, receiver) = mpsc::channel(1);
        let (ready, session) = oneshot::channel();
        let (_settled, completion) = oneshot::channel();
        let mut context = context(completion);
        context.mcp_servers = vec![McpServer::Http(
            agent_client_protocol::schema::v1::McpServerHttp::new(
                "knovum",
                "http://127.0.0.1:49152/mcp",
            )
            .headers(vec![agent_client_protocol::schema::v1::HttpHeader::new(
                "Authorization",
                "Bearer session-only-fixture",
            )]),
        )];
        let task = tokio::spawn(serve_connection(
            transport,
            context,
            receiver,
            CancellationToken::new(),
            ready,
        ));
        respond(
            &mut peer,
            "initialize",
            json!({
                "protocolVersion":1,"agentCapabilities":{"mcpCapabilities":{"http":true}}
            }),
        )
        .await;
        let request = respond(&mut peer, "session/new", json!({"sessionId":"session"})).await;
        assert_eq!(
            request["params"]["mcpServers"],
            json!([{
                "type":"http", "name":"knovum", "url":"http://127.0.0.1:49152/mcp",
                "headers":[{"name":"Authorization","value":"Bearer session-only-fixture"}]
            }])
        );
        session.await.unwrap().unwrap();
        drop(requests);
        task.await.unwrap().unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn unsupported_http_mcp_fails_before_session_creation() {
    tokio::time::timeout(Duration::from_secs(3), async {
        let (transport, mut peer) = Channel::duplex();
        let (_requests, receiver) = mpsc::channel(1);
        let (ready, session) = oneshot::channel();
        let (_settled, completion) = oneshot::channel();
        let mut context = context(completion);
        context.mcp_servers = vec![McpServer::Http(
            agent_client_protocol::schema::v1::McpServerHttp::new(
                "knovum",
                "http://127.0.0.1:49152/mcp",
            ),
        )];
        let task = tokio::spawn(serve_connection(
            transport,
            context,
            receiver,
            CancellationToken::new(),
            ready,
        ));
        respond(
            &mut peer,
            "initialize",
            json!({
                "protocolVersion":1,"agentCapabilities":{}
            }),
        )
        .await;
        let Err(error) = session.await.unwrap() else {
            panic!("unsupported MCP transport must reject session creation");
        };
        assert!(error.to_string().contains("does not support HTTP MCP"));
        task.await.unwrap().unwrap();
        assert!(peer.rx.next().await.is_none());
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn idle_transport_disconnect_ends_the_session_and_notifies_its_owner() {
    tokio::time::timeout(Duration::from_secs(3), async {
        let (transport, mut peer) = Channel::duplex();
        let (_requests, receiver) = mpsc::channel(1);
        let (ready, session) = oneshot::channel();
        let (settled, completion) = oneshot::channel();
        let task = tokio::spawn(async move {
            let outcome = serve_connection(
                transport,
                context(completion),
                receiver,
                CancellationToken::new(),
                ready,
            )
            .await;
            let _ = settled.send(outcome);
        });
        initialized_peer(&mut peer).await;
        let session = session.await.unwrap().unwrap();
        assert_eq!(session.runtime_session_id, "session");
        drop(peer);
        let error = session.completion.await.unwrap().unwrap_err();
        assert!(error
            .to_string()
            .contains("External Agent connection closed"));
        task.await.unwrap();
    })
    .await
    .expect("idle disconnection must settle without another prompt");
}

#[tokio::test]
async fn a_released_runtime_and_a_closed_one_keep_their_distinct_outcomes() {
    for cancel in [false, true] {
        tokio::time::timeout(Duration::from_secs(3), async {
            let (transport, mut peer) = Channel::duplex();
            let (requests, receiver) = mpsc::channel(1);
            let (ready, session) = oneshot::channel();
            let (_settled, completion) = oneshot::channel();
            let cancellation = CancellationToken::new();
            let task = tokio::spawn(serve_connection(
                transport,
                context(completion),
                receiver,
                cancellation.clone(),
                ready,
            ));
            initialized_peer(&mut peer).await;
            let _session = session.await.unwrap().unwrap();
            if cancel {
                cancellation.cancel();
            } else {
                drop(requests);
            }
            let outcome = task.await.unwrap();
            if cancel {
                assert_eq!(outcome.unwrap_err().code, "cancelled");
            } else {
                outcome.unwrap();
            }
        })
        .await
        .unwrap();
    }
}

fn turn(
    requests: &mpsc::Sender<TurnRequest>,
    cancellation: CancellationToken,
) -> oneshot::Receiver<AppResult<()>> {
    let (reply, outcome) = oneshot::channel();
    let prompt = crate::external_agents::prompt::AgentPrompt::from_authorized_assets(
        "Hi".into(),
        vec![],
        |_| Ok(()),
    )
    .unwrap();
    requests
        .try_send(TurnRequest {
            prompt,
            on_update: Arc::new(|_| Ok(())),
            cancellation,
            reply,
        })
        .unwrap_or_else(|_| panic!("the runtime must accept a turn"));
    outcome
}

async fn request(peer: &mut Channel, method: &str) -> serde_json::Value {
    let TransportFrame::Single(RawJsonRpcMessage::Request(request)) = peer.rx.next().await.unwrap()
    else {
        panic!("expected an ACP request");
    };
    let request = serde_json::to_value(request).unwrap();
    assert_eq!(request["method"], method);
    request
}

fn answer(peer: &mut Channel, request: &serde_json::Value, outcome: serde_json::Value) {
    let response = serde_json::from_value(json!({
        "jsonrpc": "2.0", "id": request["id"], "result": outcome,
    }))
    .unwrap();
    peer.tx
        .unbounded_send(TransportFrame::Single(response))
        .unwrap();
}

#[tokio::test]
async fn one_session_serves_every_turn_of_a_conversation() {
    // The Agent is started and its session created once; a follow-up is one more prompt.
    tokio::time::timeout(Duration::from_secs(3), async {
        let (transport, mut peer) = Channel::duplex();
        let (requests, receiver) = mpsc::channel(1);
        let (ready, session) = oneshot::channel();
        let (_settled, completion) = oneshot::channel();
        let task = tokio::spawn(serve_connection(
            transport,
            context(completion),
            receiver,
            CancellationToken::new(),
            ready,
        ));
        initialized_peer(&mut peer).await;
        let _session = session.await.unwrap().unwrap();
        for _ in 0..2 {
            let outcome = turn(&requests, CancellationToken::new());
            let prompt = request(&mut peer, "session/prompt").await;
            assert_eq!(prompt["params"]["sessionId"], "session");
            answer(&mut peer, &prompt, json!({"stopReason":"end_turn"}));
            outcome.await.unwrap().unwrap();
        }
        drop(requests);
        task.await.unwrap().unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn a_cancelled_turn_stops_the_agent_and_leaves_the_session_for_the_next() {
    tokio::time::timeout(Duration::from_secs(3), async {
        let (transport, mut peer) = Channel::duplex();
        let (requests, receiver) = mpsc::channel(1);
        let (ready, session) = oneshot::channel();
        let (_settled, completion) = oneshot::channel();
        let task = tokio::spawn(serve_connection(
            transport,
            context(completion),
            receiver,
            CancellationToken::new(),
            ready,
        ));
        initialized_peer(&mut peer).await;
        let _session = session.await.unwrap().unwrap();
        let stop = CancellationToken::new();
        let outcome = turn(&requests, stop.clone());
        let prompt = request(&mut peer, "session/prompt").await;
        stop.cancel();
        let TransportFrame::Single(RawJsonRpcMessage::Notification(cancel)) =
            peer.rx.next().await.unwrap()
        else {
            panic!("a cancelled turn must ask the Agent to stop");
        };
        assert_eq!(
            serde_json::to_value(cancel).unwrap()["method"],
            "session/cancel"
        );
        answer(&mut peer, &prompt, json!({"stopReason":"cancelled"}));
        assert_eq!(outcome.await.unwrap().unwrap_err().code, "cancelled");

        let outcome = turn(&requests, CancellationToken::new());
        let prompt = request(&mut peer, "session/prompt").await;
        answer(&mut peer, &prompt, json!({"stopReason":"end_turn"}));
        outcome.await.unwrap().unwrap();
        drop(requests);
        task.await.unwrap().unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn close_settles_when_the_registered_conversation_is_removed() {
    let id = uuid::Uuid::new_v4().to_string();
    let cancellation = CancellationToken::new();
    let _requests = requests::register(&id, cancellation.clone()).unwrap();
    let closing_id = id.clone();
    let closing = tokio::spawn(async move { requests::close(&closing_id).await });

    tokio::time::timeout(Duration::from_secs(1), cancellation.cancelled())
        .await
        .unwrap();
    assert!(!closing.is_finished());
    requests::remove(&id);
    tokio::time::timeout(Duration::from_secs(1), closing)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn disconnect_during_initialization_fails_without_a_ready_session() {
    let (transport, peer) = Channel::duplex();
    let (_requests, receiver) = mpsc::channel(1);
    let (ready, session) = oneshot::channel();
    let (_settled, completion) = oneshot::channel();
    drop(peer);
    let outcome = tokio::time::timeout(
        Duration::from_secs(3),
        serve_connection(
            transport,
            context(completion),
            receiver,
            CancellationToken::new(),
            ready,
        ),
    )
    .await
    .unwrap();
    assert!(outcome.is_err());
    assert!(session.await.is_err());
}

fn context(completion: oneshot::Receiver<AppResult<()>>) -> ServeContext {
    ServeContext {
        kind: ExternalAgentKind::GeminiCli,
        workspace: std::env::current_dir().unwrap(),
        mcp_servers: vec![],
        keepalive: Arc::new(()),
        completion,
    }
}
