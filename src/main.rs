use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use crate::requests::RequestType;

pub mod logging;
pub mod requests;

const BIND_ADDR: &'static str = "127.0.0.1:8080";
const REQUEST_TAIL: &'static str = "\r\n\r\n";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind(BIND_ADDR).await?;
    println!("Listening on {BIND_ADDR}");

    loop {
        let (sock, addr) = listener.accept().await?;

        tokio::spawn(async move {
            println!("New connection: {addr}");

            handle_connection(sock).await.unwrap_or_else(|err| {
                panic!("{:?}", err)
            });
        });
    }
}

async fn handle_connection(
    mut client: TcpStream,
) -> anyhow::Result<()> {
    let mut buf = [0u8; 4096];
    let n = client.read(&mut buf).await?;
    let raw_request = String::from_utf8_lossy(&buf[..n]);
    println!("AsyncReadExt read the following: {raw_request}");

    // parse request header
    let san_request = {
        if let Some(substr) = raw_request.split_once(REQUEST_TAIL) {
            substr.0
        } else {
            println!("Request did not have REQUEST_TAIL, so will assume as the entire header");
            &raw_request
        }
    };
    if let Some(first_line) = san_request.split("\n").into_iter().next() {
        let parts: Vec<_> = first_line.split_whitespace().collect();
        if parts.len() != 3 {
            panic!("Request was malformed");
        }
        let req_type = parts[0];
        let req_ip = parts[1];
        let req_protocol = parts[2];
        println!("Type: {req_type}, IP/Domain: {req_ip}, Protocol: {req_protocol}");

        let mut server = TcpStream::connect(req_ip).await?;
        match RequestType::from_str(req_type) {
            RequestType::Connect => {
                println!("Writing 200 connection established");
                client.write_all(&format!("{req_protocol} 200 Connection Established{REQUEST_TAIL}").into_bytes()).await?;

                println!("Tunnel starting");
                io::copy_bidirectional(&mut client, &mut server).await?;
                println!("Tunnel ended");
            },
            RequestType::Unsupported(something_else) => {
                panic!("Unsupported protocol: {something_else}")
            },
        }
    } else {
        panic!("Request was empty");
    };

    Ok(())
}