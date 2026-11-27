use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::future::BoxFuture;
use serde_json::{json, Value};

use crate::agent_runtime::process::{pool::HostSlot, Host};

struct CountingHost {
    id: &'static str,
    notifications: AtomicUsize,
}

impl CountingHost {
    fn new(id: &'static str) -> Self {
        Self {
            id,
            notifications: AtomicUsize::new(0),
        }
    }
}

impl Host for CountingHost {
    fn dispatch(&self, method: String, _params: Value) -> BoxFuture<'_, Result<Value, String>> {
        let id = self.id;
        Box::pin(async move { Ok(json!({ "host": id, "method": method })) })
    }

    fn notify(&self, _method: &str, _params: Value) -> Result<(), String> {
        self.notifications.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn host_slot_rejects_when_unbound_and_routes_cleanly_across_rebinds() {
    let slot = HostSlot::new();
    assert_eq!(
        slot.notify("agentRuntime.contentDelta", Value::Null)
            .unwrap_err(),
        "agent-runtime-host-unbound"
    );
    assert_eq!(
        slot.dispatch("agentRuntime.admitModel".into(), Value::Null)
            .await
            .unwrap_err(),
        "agent-runtime-host-unbound"
    );

    let first = Arc::new(CountingHost::new("turn-1"));
    slot.bind(first.clone());
    assert_eq!(
        slot.dispatch("agentRuntime.admitModel".into(), Value::Null)
            .await
            .unwrap(),
        json!({ "host": "turn-1", "method": "agentRuntime.admitModel" })
    );
    slot.notify("agentRuntime.contentDelta", Value::Null)
        .unwrap();
    assert_eq!(first.notifications.load(Ordering::SeqCst), 1);

    let second = Arc::new(CountingHost::new("turn-2"));
    slot.bind(second.clone());
    assert_eq!(
        slot.dispatch("agentRuntime.admitModel".into(), Value::Null)
            .await
            .unwrap(),
        json!({ "host": "turn-2", "method": "agentRuntime.admitModel" })
    );
    slot.notify("agentRuntime.contentDelta", Value::Null)
        .unwrap();
    assert_eq!(first.notifications.load(Ordering::SeqCst), 1);
    assert_eq!(second.notifications.load(Ordering::SeqCst), 1);

    slot.unbind();
    assert_eq!(
        slot.notify("agentRuntime.contentDelta", Value::Null)
            .unwrap_err(),
        "agent-runtime-host-unbound"
    );
}
