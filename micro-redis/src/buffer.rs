use crate::client::Client;
use crate::Result;
use bytes::Bytes;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;

pub fn buffer(client: Client) -> Buffer {
    // Setting the message limit to a hard coded value of 32. in a real-app, the
    // buffer size should be configurable, but we don't need to do that here.
    let (tx, rx) = channel(32);

    // Spawn a task to process requests for the connection.
    tokio::spawn(async move { run(client, rx).await });

    // Return the `Buffer` handle.
    Buffer { tx }
}

// Enum used to message pass the requested command from the `Buffer` handle
#[derive(Debug)]
enum Command {
    Get(String),
    Set(String, String),
    Select(usize),
    Ping,
    Exists(String),
    RPush(String, Bytes),
    LPush(String, Bytes),
    // BLPop(String, usize),
    // BRPop(String, usize),
}

// Message type sent over the channel to the connection task.
//
// `Command` is the command to forward to the connection.
//
// `oneshot::Sender` is a channel type that sends a **single** value. It is used
// here to send the response received from the connection back to the original
// requester.
type Message = (Command, oneshot::Sender<Result<Option<Bytes>>>);

/// Receive commands sent through the channel and forward them to client. The
/// response is returned back to the caller via a `oneshot`.
async fn run(mut client: Client, mut rx: Receiver<Message>) {
    while let Some((cmd, tx)) = rx.recv().await {
        let response: Result<Option<Bytes>> = match cmd {
            Command::Get(key) => client.get(&key).await,
            Command::Set(key, value) => client.set(&key, &value).await.map(|_| None),
            Command::Select(db) => client.select(db).await.map(|_| None),
            Command::Ping => client.ping().await.map(|_| None),
            Command::Exists(key) => client.exists(&key).await,
            Command::RPush(key, value) => client.rpush(&key, value).await.map(|_| None),
            Command::LPush(key, value) => client.lpush(&key, value).await.map(|_| None),
            // Command::BLPop(key, timeout) => {
            //     client.blpop(&key, timeout).await.map(|opt| opt.map(|(k, v)| Bytes::from([k.as_ref(), v.as_ref()].concat()))
            // Command::BRPop(key, timeout) => {
            //     client.brpop(&key, timeout).await.map(|opt| opt.map(|(k, v)| Bytes::from([k.as_ref(), v.as_ref()].concat())))
            // },
        };

        let _ = tx.send(response.map_err(|e| e.into()));
    }
}


#[derive(Clone)]
pub struct Buffer {
    tx: Sender<Message>,
}

impl Buffer {
    /// Get the value of a key.
    ///
    /// Same as `Client::get` but requests are **buffered** until the associated
    /// connection has the ability to send the request.
    pub async fn get(&mut self, key: &str) -> Result<Option<Bytes>> {
        // Initialize a new `Get` command to send via the channel.
        let get = Command::Get(key.into());

        // Initialize a new oneshot to be used to receive the response back from the connection.
        let (tx, rx) = oneshot::channel();

        // Send the request
        self.tx.send((get, tx)).await?;

        // Await the response
        match rx.await {
            Ok(res) => res,
            Err(err) => Err(err.into()),
        }
    }

    /// Set `key` to hold the given `value`.
    ///
    /// Same as `Client::set` but requests are **buffered** until the associated
    /// connection has the ability to send the request
    pub async fn set(&mut self, key: &str, value: Bytes) -> Result<()> {
        // Initialize a new `Set` command to send via the channel.
        let set = Command::Set(key.into(), String::from_utf8(value.to_vec()).unwrap());

        // Initialize a new oneshot to be used to receive the response back from the connection.
        let (tx, rx) = oneshot::channel();

        // Send the request
        self
            .tx
            .send((set, tx))
            .await?;

        // Await the response

        match rx.await {
            Ok(res) => res.map(|_| ()),
            Err(err) => Err(err.into()),
        }
    }

    /// Select the given database.
    /// 
    /// Same as `Client::select` but requests are **buffered** until the associated
    /// connection has the ability to send the request
    pub async fn select(&mut self, db: usize) -> Result<()> {
        // Initialize a new `Select` command to send via the channel.
        let select = Command::Select(db);

        // Initialize a new oneshot to be used to receive the response back from the connection.
        let (tx, rx) = oneshot::channel();

        // Send the request
        self.tx.send((select, tx)).await?;

        // Await the response
        match rx.await {
            Ok(res) => res.map(|_| ()),
            Err(err) => Err(err.into()),
        }
    }

    /// Ping the server.
    pub async fn ping(&mut self) -> Result<()> {
        // Initialize a new `Ping` command to send via the channel.
        let ping = Command::Ping;

        // Initialize a new oneshot to be used to receive the response back from the connection.
        let (tx, rx) = oneshot::channel();

        // Send the request
        self.tx.send((ping, tx)).await?;

        // Await the response
        match rx.await {
            Ok(res) => res.map(|_| ()),
            Err(err) => Err(err.into()),
        }
    }


    /// Check if a key exists.
    pub async fn exists(&mut self, key: &str) -> Result<Option<Bytes>> {
        // Initialize a new `Exists` command to send via the channel.
        let exists = Command::Exists(key.into());

        // Initialize a new oneshot to be used to receive the response back from the connection.
        let (tx, rx) = oneshot::channel();

        // Send the request
        self.tx.send((exists, tx)).await?;

        // Await the response
        match rx.await {
            Ok(res) => res,
            Err(err) => Err(err.into()),
        }
    }


    /// Push a value to the right end of a list.
    pub async fn rpush(&mut self, key: &str, value: Bytes) -> Result<Option<Bytes>> {
        // Initialize a new `RPush` command to send via the channel.
        let rpush = Command::RPush(key.into(), value);

        // Initialize a new oneshot to be used to receive the response back from the connection.
        let (tx, rx) = oneshot::channel();

        // Send the request
        self.tx.send((rpush, tx)).await?;

        // Await the response
        match rx.await {
            Ok(res) => res,
            Err(err) => Err(err.into()),
        }
    }

    /// Push a value to the left end of a list.
    pub async fn lpush(&mut self, key: &str, value: Bytes) -> Result<Option<Bytes>> {
        // Initialize a new `LPush` command to send via the channel.
        let lpush = Command::LPush(key.into(), value);

        // Initialize a new oneshot to be used to receive the response back from the connection.
        let (tx, rx) = oneshot::channel();

        // Send the request
        self.tx.send((lpush, tx)).await?;

        // Await the response
        match rx.await {
            Ok(res) => res,
            Err(err) => Err(err.into()),
        }
    }

    // pub async fn blpop(&mut self, key: &str, timeout: usize) -> Result<Option<Bytes>> {
    //     // Initialize a new `BLPop` command to send via the channel.
    //     let blpop = Command::BLPop(key.into(), timeout);

    //     // Initialize a new oneshot to be used to receive the response back from the connection.
    //     let (tx, rx) = oneshot::channel();

    //     // Send the request
    //     self.tx.send((blpop, tx)).await?;

    //     // Await the response
    //     match rx.await {
    //         Ok(res) => res,
    //         Err(err) => Err(err.into()),
    //     }
    // }

    // pub async fn brpop(&mut self, key: &str, timeout: usize) -> Result<Option<Bytes>> {
    //     // Initialize a new `BRPop` command to send via the channel.
    //     let brpop = Command::BRPop(key.into(), timeout);

    //     // Initialize a new oneshot to be used to receive the response back from the connection.
    //     let (tx, rx) = oneshot::channel();

    //     // Send the request
    //     self.tx.send((brpop, tx)).await?;

    //     // Await the response
    //     match rx.await {
    //         Ok(res) => res,
    //         Err(err) => Err(err.into()),
    //     }
    // }
}
