use eoncache::{Db, Shutdown, run_server};
use tokio::net::TcpListener;
use std::sync::Arc;

#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Create the shared database instance=
    let db = Arc::new(Db::new()); 

    // Set up the TCP listener
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Server is running at 127.0.0.1:6379");

    // Listen for the shutdown signal in another task or handling
    let shutdown = Shutdown::new();
    let shutdown_clone = shutdown.clone();
    tokio::spawn(async move {
        shutdown_clone.listen_for_ctrl_c().await;
    });
    // Run the server
    let _ = run_server(listener, db, shutdown).await;
    println!("Server has shut down");
    Ok(())
}
