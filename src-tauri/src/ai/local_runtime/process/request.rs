use serde::Serialize;
use std::time::Duration;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use super::Worker;

impl Worker {
    /// Dispatches a request and returns the instant its response is due.
    ///
    /// The execution budget starts here, at dispatch, because this is the first moment the work
    /// can actually proceed: the slot is held and the worker is initialized.
    pub(in crate::ai::local_runtime) async fn send_request<P: Serialize>(
        &mut self,
        request_id: &str,
        method: &str,
        params: &P,
        budget_ms: u64,
        cancellation: &CancellationToken,
        exit: &CancellationToken,
    ) -> Result<Instant, String> {
        let sent = async {
            let deadline = Instant::now() + Duration::from_millis(budget_ms);
            tokio::select! {
                biased;
                _ = exit.cancelled() => Err("cancelled".to_owned()),
                _ = cancellation.cancelled() => Err("cancelled".to_owned()),
                response = tokio::time::timeout_at(deadline, self.protocol.send_request(request_id, method, params)) => {
                    response.map_err(|_| "deadline_exceeded".to_owned())?.map(|()| deadline)
                }
            }
        }.await;
        if sent.is_err() {
            self.terminate().await;
        }
        sent
    }
}
