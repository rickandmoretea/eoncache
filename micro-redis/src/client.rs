use tokio::net::{TcpStream, ToSocketAddrs};
use bytes::Bytes;
use tracing::debug;
use crate::{Connection, Frame};
use std::io::{Error, ErrorKind};

pub struct Client {
    connection: Connection,
}

pub async fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<Client> 
    where T: 
{
    let socket = TcpStream::connect(addr).await?;
    let connection = Connection::new(socket);

    Ok(Client { connection })
}

impl Client {


    pub async fn select(&mut self, db: usize) -> crate::Result<usize> {
        // Create the command parts as bulk strings
        let command_part = Frame::Bulk(Bytes::from_static(b"SELECT"));
        let db_part = Frame::Bulk(Bytes::from(db.to_string()));
    
        // Send the command as an array of bulk strings
        let cmd = Frame::Array(vec![command_part, db_part]);
        self.connection.write_frame(&cmd).await?;
    
        // Wait for and process the response
        let response = self.read_response().await?;
        match response {
            Frame::Simple(msg) if msg == "OK" => Ok(db),
            Frame::Error(err) => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, err))),
            _ => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Unexpected response type"))),
        }
    }
    

    pub async fn get(&mut self, key: &str) -> crate::Result<Option<Bytes>> {
        let command_part = Frame::Bulk(Bytes::from_static(b"GET"));
        let key_part = Frame::Bulk(Bytes::from(key.to_owned()));
        let cmd = Frame::Array(vec![command_part, key_part]);

        self.connection.write_frame(&cmd).await?;
        match self.read_response().await? {
            Frame::Bulk(data) => Ok(Some(data)),
            Frame::Null => Ok(None),
            frame => Err(Error::new(ErrorKind::Other, format!("Unexpected frame type: {:?}", frame)).into()),
        }

    }

    pub async fn set(&mut self, key: &str, value: &str) -> crate::Result<()> {
        // Correctly format a Redis command with multiple parts
        let cmd = format!("*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n", key.len(), key, value.len(), value);
        
        // Write the command to the connection
        self.connection.write_frame(&Frame::Bulk(Bytes::from(cmd))).await?;
        
        // Wait for and process the response
        self.read_response().await.map(|_| ())
    }
    

    pub async fn ping(&mut self) -> crate::Result<()> {
        self.connection.write_frame(&Frame::Array(vec![Frame::Bulk(Bytes::from("PING"))])).await?;
        self.read_response().await.map(|_| ())
    }

    pub async fn exists(&mut self, key: &str) -> crate::Result<Option<Bytes>> {
        let cmd = format!("EXISTS {}\r\n", key);
        self.connection.write_frame(&Frame::Bulk(Bytes::from(cmd))).await?;
        match self.read_response().await? {
            Frame::Integer(n) => Ok(Some(Bytes::from(n.to_string()))),
            Frame::Null => Ok(None),
            frame => Err(Error::new(ErrorKind::Other, format!("Unexpected frame type: {:?}", frame)).into()),
        }
    }

    pub async fn rpush(&mut self, key: &str, value: Bytes) -> crate::Result<Option<Bytes>> {
        match std::str::from_utf8(&value) {
            Ok(value_str) => {
                let cmd = format!("RPUSH {} {}\r\n", key, value_str);
                self.connection.write_frame(&Frame::Bulk(Bytes::from(cmd))).await?;
                match self.read_response().await? {
                    Frame::Integer(n) => Ok(Some(Bytes::from(n.to_string()))),
                    Frame::Null => Ok(None),
                    frame => Err(Error::new(ErrorKind::Other, format!("Unexpected frame type: {:?}", frame)).into()),
                }
            },
            Err(_) => Err("Value is not valid UTF-8".into()),
        }
    }

    pub async fn lpush(&mut self, key: &str, value: Bytes) -> crate::Result<Option<Bytes>> {
        match std::str::from_utf8(&value) {
            Ok(value_str) => {
                let cmd = format!("LPUSH {} {}\r\n", key, value_str);
                self.connection.write_frame(&Frame::Bulk(Bytes::from(cmd))).await?;
                match self.read_response().await? {
                    Frame::Integer(n) => Ok(Some(Bytes::from(n.to_string()))),
                    Frame::Null => Ok(None),
                    frame => Err(Error::new(ErrorKind::Other, format!("Unexpected frame type: {:?}", frame)).into()),
                }
            },
            Err(_) => Err("Value is not valid UTF-8".into()),
        }
    }

    pub async fn blpop(&mut self, keys: &str, timeout: usize) -> crate::Result<Option<(Bytes, Bytes)>> {
        let cmd = format!("BLPOP {} {}\r\n", keys, timeout);
        self.connection.write_frame(&Frame::Bulk(Bytes::from(cmd))).await?;
        match self.read_response().await? {
            Frame::Array(frames) => {
                if frames.len() == 2 {
                    match (&frames[0], &frames[1]) {
                        (Frame::Bulk(key), Frame::Bulk(value)) => Ok(Some((key.clone(), value.clone()))),
                        _ => Err(Error::new(ErrorKind::Other, "Unexpected frame type").into()),
                    }
                } else {
                    Err(Error::new(ErrorKind::Other, "Unexpected frame length").into())
                }
            },
            Frame::Null => Ok(None),
            frame => Err(Error::new(ErrorKind::Other, format!("Unexpected frame type: {:?}", frame)).into()),
        }
    }
    
    pub async fn brpop(&mut self, key: &str, timeout: usize) -> crate::Result<Option<(Bytes, Bytes)>> {
        let cmd = format!("BRPOP {} {}\r\n", key, timeout);
        self.connection.write_frame(&Frame::Bulk(Bytes::from(cmd))).await?;
        match self.read_response().await? {
            Frame::Array(frames) => {
                if frames.len() == 2 {
                    match (&frames[0], &frames[1]) {
                        (Frame::Bulk(key), Frame::Bulk(value)) => Ok(Some((key.clone(), value.clone()))),
                        _ => Err(Error::new(ErrorKind::Other, "Unexpected frame type").into()),
                    }
                } else {
                    Err(Error::new(ErrorKind::Other, "Unexpected frame length").into())
                }
            },
            Frame::Null => Ok(None),
            frame => Err(Error::new(ErrorKind::Other, format!("Unexpected frame type: {:?}", frame)).into()),
        }
    }




    async fn read_response(&mut self) -> crate::Result<Frame> {
        let response = self.connection.read_frame().await?;

        debug!(?response);

        match response {
            // Error frames are converted to `Err`
            Some(Frame::Error(msg)) => Err(msg.into()),
            Some(frame) => Ok(frame),
            None => {
                // Receiving `None` here indicates the server has closed the
                // connection without sending a frame. This is unexpected and is
                // represented as a "connection reset by peer" error.
                let err = Error::new(ErrorKind::ConnectionReset, "connection reset by server");

                Err(err.into())
            }
        }
    }
}