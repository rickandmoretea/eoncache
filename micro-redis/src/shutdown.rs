use tokio::sync::broadcast;

#[derive(Debug)]
pub(crate) struct Shutdown {
    is_shutdown: bool,
    notify: broadcast::Sender<()>,
}

impl Shutdown {
    pub(crate) fn new(notify: broadcast::Sender<()>) -> Shutdown {
        Shutdown {
            is_shutdown: false,
            notify,
        }
    }

    pub(crate) fn is_shutdown(&self) -> bool {
        self.is_shutdown
    }

    pub(crate) async fn recv(&mut self) {
        if self.is_shutdown {
            return;
        }
        // Cannot receive a "lag error" as only one value is ever sent.
        let _ = self.notify.subscribe().recv().await;
        // Update the shutdown flag
        self.is_shutdown = true;
    }
}