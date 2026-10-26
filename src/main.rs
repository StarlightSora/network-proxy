use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

const BIND_ADDR: &'static str = "127.0.0.1:8080";

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
    //Ok(())
}

async fn handle_connection(
    mut client: TcpStream,
) -> anyhow::Result<()> {
    let mut buf = [0u8; 4096];
    let n = client.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);
    println!("AsyncReadExt read the following: {request}");

    // parse request
    if let Some(first_line) = request.split("\n").into_iter().next() {
        let parts: Vec<_> = first_line.split_whitespace().collect();
        if parts.len() != 3 {
            panic!("Request was malformed");
        }
        let req_type = parts[0];
        let req_ip = parts[1];
        let req_protocol = parts[2];
        println!("Type: {req_type}, IP/Domain: {req_ip}, Protocol: {req_protocol}");

        let mut server = TcpStream::connect(req_ip).await?;
        println!("Writing 200 connection established");
        client.write_all(&format!("{req_protocol} 200 Connection Established\r\n\r\n").into_bytes()).await?;

        println!("Tunnel starting");
        io::copy_bidirectional(&mut client, &mut server).await?;
        println!("Tunnel ended");
    } else {
        panic!("Request was empty");
    };

    Ok(())
}