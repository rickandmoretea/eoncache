use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use std::sync::Arc;
use crate::Db;
use crate::connection::Connection;
use crate::shutdown::Shutdown;
use crate::parse::Parse;
use crate::command::handle_command;

pub async fn run_server(listener: TcpListener, db: Arc<Db>, shutdown: Shutdown) -> crate::Result<()> {
    // Listen for CTRL+C in a separate task
    let shutdown_clone = shutdown.clone();
    tokio::spawn(async move {
        shutdown_clone.listen_for_ctrl_c().await;
    });

    let mut receiver = shutdown.subscribe();  // Get a subscriber for the main loop

    loop {
        tokio::select! {
            Ok((socket, _)) = listener.accept() => {
                let db_clone = db.clone();
                let shutdown_clone_for_connection = shutdown.subscribe();  // Each connection gets a new subscriber
                tokio::spawn(async move {
                    process_connection(socket, db_clone, shutdown_clone_for_connection).await;
                });
            },
            _ = receiver.recv() => {
                println!("Server shutdown initiated...");
                break;
            },
        }
    }

    Ok(())
}

async fn process_connection(socket: TcpStream, db: Arc<Db>, mut shutdown_recv: broadcast::Receiver<()>) {
    let mut connection = Connection::new(socket);

    while let Ok(Some(frame)) = connection.read_frame().await {
        let mut parse = Parse::new(frame).unwrap();
        match handle_command(&mut parse, &db).await {
            Ok(response) => {
                if connection.write_frame(&response).await.is_err() {
                    eprintln!("Error sending response");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error handling command: {}", e);
                break;
            }
        }

        // Check for shutdown signal
        if shutdown_recv.try_recv().is_ok() {
            println!("Connection received shutdown signal...");
            break;
        }
    }
}
