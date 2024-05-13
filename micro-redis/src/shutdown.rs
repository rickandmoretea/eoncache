use tokio::sync::broadcast;

#[derive(Debug)]
pub struct Shutdown {
    sender: broadcast::Sender<()>,
}

impl Clone for Shutdown {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl Shutdown {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1);
        Shutdown { sender }
    }

    // Method to create a new subscriber
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.sender.subscribe()
    }

    // Method to trigger the shutdown signal
    pub async fn shutdown_signal(&self) {
        let _ = self.sender.send(()).expect("Failed to send shutdown signal");
    }

    // Optional: Listen for CTRL+C directly within this struct
    pub async fn listen_for_ctrl_c(&self) {
        tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
        self.shutdown_signal().await;  // Propagate the shutdown signal
    }
}
