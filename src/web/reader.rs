use log::{info, debug};
use tokio::{net::TcpStream, io::{BufReader, BufWriter, AsyncBufReadExt, AsyncReadExt}, pin};
use std::io::{self, prelude::*};

pub async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<String> {


    let peer_addr = stream.peer_addr().expect("Stream has peer_addr");
    info!("Incoming from {:?}", peer_addr);

    let mut buf = [0; 1024];
    let mut string_chunks: Vec<String> = Vec::new();
    loop {
        match stream.read(&mut buf).await {
            // socket closed
            Ok(n) if n == 0 =>  {
                debug!("socket closed");
                return Ok(string_chunks.join(""));
            },
            Ok(n) => {
                debug!("read {} bytes", n);
                let s = match std::str::from_utf8(buf[0..n].as_ref()) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };
                debug!("read: {}", s);
                string_chunks.push(s.to_string());
                n
            },
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                return Ok(String::from("Error"));
            }
        };

    }




    // extract_string_buffered(&mut reader).await;
    // Ok(())
}

// pub async fn extract_string_buffered(mut buf: &mut impl tokio::io::AsyncRead) -> io::Result<String> {
//     let mut reader = BufReader::new(&mut buf);
//
//     // `fill_buf` will return a ref to the bytes pending (received by TCP)
//     // This is still a lower-level call, so we have to follow it up with a call to consume
//     let received: Vec<u8> = reader.fill_buf().await.unwrap().to_vec();  // .fill_buf()?.to_vec();
//
// `impl tokio::io::AsyncRead` cannot be unpinned
//    consider using the `pin!` macro
//
//     // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
//     reader.consume(received.len());
//
//     String::from_utf8(received).map_err(|_| {
//         io::Error::new(
//             io::ErrorKind::InvalidData,
//             "Couldn't parse received string as utf8",
//         )
//     })
// }
