// use micro_redis::{Client, Buffer};
// use bytes::Bytes;
// use std::io::{self, Write};

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let mut client = Client::connect("127.0.0.1:6379").await?;
//     let buffer = Buffer::new(client);

//     loop {
//         print!("redis> ");
//         io::stdout().flush()?;
//         let mut command = String::new();
//         io::stdin().read_line(&mut command)?;
//         let parts: Vec<&str> = command.trim().split_whitespace().collect();
//         match parts[0] {
//             "SET" if parts.len() == 3 => {
//                 buffer.set(parts[1].to_string(), Bytes::from(parts[2])).await?;
//             },
//             "GET" if parts.len() == 2 => {
//                 if let Some(value) = buffer.get(parts[1].to_string()).await? {
//                     println!("{}", String::from_utf8_lossy(&value));
//                 } else {
//                     println!("(nil)");
//                 }
//             },
//             _ => println!("Unknown command or incorrect format"),
//         }
//     }
// }
