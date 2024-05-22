use std::sync::Arc;
use crate::{Db, Frame, Parse};
use tokio::time::Duration;
use tracing::{info, warn};

pub async fn handle_command(parse: &mut Parse, db: &Arc<Db>) -> crate::Result<Frame> {
    println!("Received command: {:?}", parse);  // Debug output for incoming frames
    let command = parse.next_string()?.to_uppercase();
    match command.as_str() {
        "SELECT" => handle_select(parse, db).await,
        "SET" => handle_set(parse, db).await,
        "GET" => handle_get(parse, db).await,
        "PING" => handle_ping().await,
        "EXISTS" => handle_exists(parse, db).await,
        "RPUSH" => handle_rpush(parse, db).await,
        "LPUSH" => handle_lpush(parse, db).await,
        "BLPOP" => {
            let timeout = parse.next_string()?.parse().map_err(|_| "Invalid timeout")?;
            handle_blpop(parse, db, timeout).await
        },
        "BRPOP" => {
            let timeout = parse.next_string()?.parse().map_err(|_| "Invalid timeout")?;
            handle_brpop(parse, db, timeout).await
        },
        _ => Err("Unsupported command".into()),
    }
}
pub async fn handle_select(parse: &mut Parse, db: &Arc<Db>) -> crate::Result<Frame> {
    match parse.next_int() {  // Directly parse as integer
        Ok(index) if index < 16 => {  // Validate index range if there are 16 namespaces
            match db.select_namespace(index as usize) {
                Ok(_) => {
                    println!("Selected namespace: {}", index);
                    Ok(Frame::Simple("OK".to_string()))
                },
                Err(e) => {
                    println!("Failed to select namespace {}: {}", index, e);
                    Err(e.into())
                }
            }
        },
        Ok(_) => {
            let err_msg = format!("Invalid index: index out of allowed range (0-15)");
            warn!("{}", err_msg);
            Err(err_msg.into())
        },
        Err(_) => {
            warn!("Failed to parse index for SELECT command");
            Err("Failed to parse index".into())
        }
    }
}


async fn handle_get(parse: &mut Parse, db: &Db) -> crate::Result<Frame> {
    println!("Attempting to handle GET command");  // Debug print
    if let Ok(key) = parse.next_string() {
        println!("Parsed key for GET: {}", key);  // Debug print
        parse.finish()?;

        match db.get(&key) {
            Some(value) => {
                println!("Found value for key '{}'", key);  // Debug print
                Ok(Frame::Bulk(value))
            },
            None => {
                println!("No value found for key '{}'", key);  // Debug print
                Ok(Frame::Null)
            },
        }
    } else {
        println!("Failed to parse key for GET command");  // Debug print
        Err("Failed to parse key for GET command".into())
    }
}

async fn handle_set(parse: &mut Parse, db:  &Arc<Db>) -> crate::Result<Frame> {
    let key = parse.next_string()?;
    let value = parse.next_bytes()?;
    parse.finish()?;
    db.set(key, value);
    Ok(Frame::Simple("OK".to_string()))
}

async fn handle_ping() -> crate::Result<Frame> {
    Ok(Frame::Simple("PONG".to_string()))
}

async fn handle_exists(parse: &mut Parse, db: &Arc<Db>) -> crate::Result<Frame> {
    let key = parse.next_string()?;
    parse.finish()?;

    Ok(Frame::Integer(db.exists(&key) as u64))
}

async fn handle_rpush(parse: &mut Parse, db: &Arc<Db>) -> crate::Result<Frame> {
    let key = parse.next_string()?;
    let value = parse.next_bytes()?;
    let _ = parse.finish();
    db.rpush(key, value);
    Ok(Frame::Simple("OK".to_string()))
}

async fn handle_lpush(parse: &mut Parse, db: &Arc<Db>) -> crate::Result<Frame> {
    let key = parse.next_string()?;
    let value = parse.next_bytes()?;
    let _ = parse.finish();
    db.lpush(key, value);
    Ok(Frame::Simple("OK".to_string()))
}

async fn handle_blpop(parse: &mut Parse, db: &Arc<Db>, timeout: f64) -> crate::Result<Frame> {
    let mut keys = Vec::new();
    while let Ok(key) = parse.next_string() {
        keys.push(key);
    }

    if keys.is_empty() {
        return Err("BLPOP requires keys specified".into());
    }

    let timeout_duration = Duration::from_secs_f64(timeout);

    match db.blpop(keys, timeout_duration).await {
        Some((key, value)) => Ok(Frame::Array(vec![Frame::Bulk(key.into()), Frame::Bulk(value)])),
        None => Ok(Frame::Null),
    }
}

async fn handle_brpop(parse: &mut Parse, db: &Arc<Db>, timeout: f64) -> crate::Result<Frame> {
    let mut keys = Vec::new();
    while let Ok(key) = parse.next_string() {
        keys.push(key);
    }

    if keys.is_empty() {
        return Err("BRPOP requires keys specified".into());
    }

    let timeout_duration = Duration::from_secs_f64(timeout);

    match db.brpop(keys, timeout_duration).await {
        Some((key, value)) => Ok(Frame::Array(vec![Frame::Bulk(key.into()), Frame::Bulk(value)])),
        None => Ok(Frame::Null),
    }
}

