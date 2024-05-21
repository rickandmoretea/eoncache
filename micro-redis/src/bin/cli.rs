use micro_redis::{client, Connection, Frame};
use bytes::Bytes;
use std::str;
use std::time::Duration;
use structopt::StructOpt;
use micro_redis::Error;

#[derive(StructOpt, Debug)]
#[structopt(name = "mini-redis-cli", about = "A CLI for Redis operations")]
struct Cli {
    #[structopt(subcommand)]
    command: Command,

    #[structopt(long = "--host", default_value = "127.0.0.1")]
    host: String,

    #[structopt(long = "--port", default_value = "6379")]
    port: u16,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Select the given database
    Select { index: usize },
    /// Get the value of a key
    Get { key: String },
    /// Set a key to hold the specified string value
    Set { key: String, value: String },
    /// Ping the server
    Ping,
    /// Check existence of key
    Exists { key: String },
}

    // /// Append one or several values to a list
    // RPush { key: String, values: Vec<String> },
    // /// Prepend one or several values to a list
    // LPush { key: String, values: Vec<String> },
    // /// Remove and get the first element in a list, or block until one is available
    // BLPop { keys: Vec<String>, timeout: u64 },
    // /// Remove and get the last element in a list, or block until one is available
    // BRPop { keys: Vec<String>, timeout: u64 },

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::from_args();
    let addr = format!("{}:{}", cli.host, cli.port);
    let mut client = client::connect(&addr).await?;

    match cli.command {
        Command::Select { index } => {
            let db = client.select(index).await?;
            println!("Selected database: {}", db);
        },
        Command::Get { key } => {
            let value = client.get(&key).await?;
            match value {
                Some(value) => println!("{}", str::from_utf8(&value)?),
                None => println!("(nil)"),
            }
        },
        Command::Set { key, value } => {
            client.set(&key, &value).await?;
            println!("OK");
        },
        Command::Ping => {
            client.ping().await?;
            println!("PONG");
        },
        Command::Exists { key } => {
            let exist = client.exists(&key).await?;
            match exist {
                Some(value) => println!("{}", str::from_utf8(&value)?),
                None => println!("(nil)"),
            }
        }, 
    }

    Ok(())
}

