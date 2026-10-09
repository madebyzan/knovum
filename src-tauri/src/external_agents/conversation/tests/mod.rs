use super::*;
use agent_client_protocol::{Channel, RawJsonRpcMessage, TransportFrame};
use futures::StreamExt;
use serde_json::json;
use std::time::Duration;

async fn respond(peer: &mut Channel, method: &str, outcome: serde_json::Value) {
    let TransportFrame::Single(RawJsonRpcMessage::Request(request)) = peer.rx.next().await.unwrap()
    else {
        panic!("expected an ACP request");
    };
    let request = serde_json::to_value(request).unwrap();
    assert_eq!(request["method"], method);
    let response = serde_json::from_value(json!({
        "jsonrpc": "2.0", "id": request["id"], "result": outcome,
    }))
    .unwrap();
    peer.tx
        .unbounded_send(TransportFrame::Single(response))
        .unwrap();
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
async fn explicit_finish_and_cancellation_keep_their_distinct_outcomes() {
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
                requests.send(ConversationRequest::Finish).await.unwrap();
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
        on_update: Arc::new(|_| Ok(())),
        mcp_servers: vec![],
        keepalive: Arc::new(()),
        completion,
    }
}
