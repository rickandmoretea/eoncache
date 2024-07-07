use micro_redis::client;
use std::{str, io};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::io::Write; 
use micro_redis::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut select_used = false;
    println!("Connecting to Redis...");
    let addr = "127.0.0.1:6379";
    let client = client::connect(addr).await?;

    let client = Arc::new(Mutex::new(client));
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    println!("Connected to Redis at {}. Type commands (type 'exit' to quit):", addr);
    print!("> ");
    io::stdout().flush().unwrap(); // Make sure the prompt is displayed immediately

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
                if select_used {
                    println!("Error: SELECT can only be called at the start of the session");
                }
                else if let Some(index_str) = parts.get(1) {
                    if let Ok(index) = index_str.parse::<usize>() {
                        let client = client.clone();
                        tokio::spawn(async move {
                            let res = client.lock().await.select(index).await;
                            match res {
                                Ok(_) => println!("Database {} selected", index),
                                Err(e) => println!("Error: {}", e),
                            }
                        });
                        select_used = true;
                    } else {
                        println!("Invalid database index.");
                    }
                } else {
                    println!("Usage: SELECT <index> (0-15) default 0");
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
                    let key = key.to_string();  // Clone the key here to create an owned String
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
            "rpush" => {
                if parts.len() > 2 {
                    let key = parts[1].to_string();
                    let values = parts[2..].join(" ");
                    let client = client.clone();
                    tokio::spawn(async move {
                        let res = client.lock().await.rpush(&key, values.into()).await;
                        match res {
                            Ok(len) => println!("{:?}", len),
                            Err(e) => println!("Error: {}", e),
                        }
                    });
                } else {
                    println!("Usage: RPUSH <key> <value>");
                }
            },
            "lpush" => {
                if parts.len() > 2 {
                    let key = parts[1].to_string();
                    let values = parts[2..].join(" ");
                    let client = client.clone();
                    tokio::spawn(async move {
                        let res = client.lock().await.lpush(&key, values.into()).await;
                        match res {
                            Ok(len) => println!("{:?}", len),
                            Err(e) => println!("Error: {}", e),
                        }
                    });
                } else {
                    println!("Usage: LPUSH <key> <value>");
                }
            },


            "help" => {
                println!("Available commands: SELECT, GET, SET, PING, EXISTS, RPUSH, LPUSH");
            },

            _ => {
                println!("Unsupported command. Available commands: SELECT, GET, SET, PING, EXISTS, RPUSH, LPUSH");
            }
        }

        print!("> "); // Prompt for the next command
        io::stdout().flush().unwrap(); // Make sure the prompt is displayed immediately
    }

    Ok(())
}
