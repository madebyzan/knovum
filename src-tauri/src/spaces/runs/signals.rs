use std::sync::OnceLock;

use tokio::sync::watch;

static CHANGES: OnceLock<watch::Sender<u64>> = OnceLock::new();

pub(crate) fn subscribe() -> watch::Receiver<u64> {
    channel().subscribe()
}

pub(crate) fn announce() {
    channel().send_modify(|version| *version = version.wrapping_add(1));
}

fn channel() -> &'static watch::Sender<u64> {
    CHANGES.get_or_init(|| watch::channel(0).0)
}
