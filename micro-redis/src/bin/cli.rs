use micro_redis::client;
use std::{str, io};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::io::Write; 
use micro_redis::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Connecting to Redis...");
    let addr = "127.0.0.1:6379"; // Example, replace with actual configuration if needed
    let client = client::connect(addr).await?;

    let client = Arc::new(Mutex::new(client));
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    println!("Connected to Redis at {}. Type commands (type 'exit' to quit):", addr);
    print!("> ");
    io::stdout().flush().unwrap(); 

    while let Ok(Some(line)) = lines.next_line().await {
        if line.eq_ignore_ascii_case("exit") {
            break;
        }
        
        let parts: Vec<String> = line.split_whitespace().map(str::to_string).collect();
        if parts.is_empty() {
            continue;
        }
        

        match parts[0].to_lowercase().as_str() {
            "select" => {
                if let Some(index_str) = parts.get(1) {
                    if let Ok(index) = index_str.parse::<usize>() {
                        let client = client.clone();
                        tokio::spawn(async move {
                            let res = client.lock().await.select(index).await;
                            match res {
                                Ok(_) => println!("Database {} selected", index),
                                Err(e) => println!("Error: {}", e),
                            }
                        });
                    } else {
                        println!("Invalid database index.");
                    }
                } else {
                    println!("Usage: SELECT <index>");
                }
            },
            "get" => {
                if let Some(key) = parts.get(1) {
                    let key = key.to_string();  
                    let client = client.clone();
                    tokio::spawn(async move {
                        let res = client.lock().await.get(&key).await;
                        match res {
                            Ok(Some(value)) => println!("\"{}\"", String::from_utf8_lossy(&value)),
                            Ok(None) => println!("Key does not exist."),
                            Err(e) => println!("Error retrieving key: {}", e),
                        }
                    });
                } else {
                    println!("Usage: GET <key>");
                }
            },
            "set" => {
                if parts.len() > 2 {
                    let key = parts[1].to_string();
                    let value = parts[2..].join(" ");
                    let client = client.clone();
                    tokio::spawn(async move {
                        let res = client.lock().await.set(&key, &value).await;
                        match res {
                            Ok(_) => println!("OK"),
                            Err(e) => println!("Error: {}", e),
                        }
                    });
                } else {
                    println!("Usage: SET <key> <value>");
                }
            },
            "ping" => {
                let client = client.clone();
                tokio::spawn(async move {
                    let res = client.lock().await.ping().await;
                    match res {
                        Ok(_) => println!("PONG"),
                        Err(e) => println!("Error: {}", e),
                    }
                });
            },
            "exists" => {
                if let Some(key) = parts.get(1) {
                    let key = key.to_string();  
                    let client = client.clone();
                    tokio::spawn(async move {
                        let res = client.lock().await.exists(&key).await;
                        match res {
                            Ok(Some(value)) => println!("{}", String::from_utf8_lossy(&value)),
                            Ok(None) => println!("(nil)"),
                            Err(e) => println!("Error: {}", e),
                        }
                    });
                } else {
                    println!("Usage: EXISTS <key>");
                }
            },
            _ => {
                println!("Unsupported command. Available commands: SELECT, GET, SET, PING, EXISTS");
            }
        }

        print!("> "); // Prompt for the next command
        io::stdout().flush().unwrap(); // Make sure the prompt is displayed immediately
    }

    Ok(())
}
