use serde::Serialize;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use super::Worker;

impl Worker {
    pub(in crate::ai::local_runtime) async fn send_request<P: Serialize>(
        &mut self,
        request_id: &str,
        method: &str,
        params: &P,
        deadline_ms: u64,
        cancellation: &CancellationToken,
        exit: &CancellationToken,
    ) -> Result<Instant, String> {
        let sent = async {
            let deadline = Instant::now() + crate::ai::local_runtime::deadline_duration(deadline_ms)?;
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
