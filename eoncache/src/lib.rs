pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
pub const DEFAULT_PORT: u16 = 6379;

// command
pub mod command;

// server
pub mod server;
pub use server::run_server;

//db 
pub mod db;
pub use db::Db;


// parse 

mod parse;
use parse::Parse;

// frame
pub mod frame;
pub use frame::Frame;

// connection
mod connection;
pub use connection::Connection;


// shutdown
mod shutdown;
pub use shutdown::Shutdown;


// client
pub mod client;
pub use client::Client;

// buffer
pub mod buffer;
pub use buffer::Buffer;




