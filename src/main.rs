pub mod components;
pub mod requests;

use std::net::SocketAddr;

use time::Timestamp;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;

use crate::components::logging::datatypes::{LogFrom, LogType, NpLog};
use crate::requests::RequestType;

const LOG_BUFFER_SIZE: usize = 1024;
const BIND_ADDR: &'static str = "127.0.0.1:8080";
const REQUEST_TAIL: &'static str = "\r\n\r\n";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut log_file = components::logging::fileio::LogFile::from_date_time(Timestamp::now()).await?;
    let (log_channel_send, mut log_channel_recv) = mpsc::channel(LOG_BUFFER_SIZE);

    tokio::spawn(async move {
        while let Some(incoming_log) = log_channel_recv.recv().await {
            match log_file.write_log(incoming_log).await {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("Failed to log message");
                },
            };
        };
    });

    let listener = TcpListener::bind(BIND_ADDR).await?;
    log_channel_send.send(NpLog::now(
        LogFrom::Net, LogType::Info,
        format!("Listening on {BIND_ADDR}"),
    )).await?;

    loop {
        let log_channel_send = log_channel_send.clone();
        let (sock, addr) = listener.accept().await?;

        tokio::spawn(async move {
            handle_connection(addr, sock, log_channel_send).await.unwrap_or_else(|err| {
                panic!("{:?}", err) // TODO: How do we handle this?
            });
        });
    }
}

async fn handle_connection(
    addr: SocketAddr,
    mut client: TcpStream,
    log_channel_send: mpsc::Sender<NpLog>,
) -> anyhow::Result<()> {
    let mut buf = [0u8; 4096];
    let n = client.read(&mut buf).await?;
    let raw_request = String::from_utf8_lossy(&buf[..n]);
    log_channel_send.send(NpLog::now(
        LogFrom::Net, LogType::Verbose,
        format!("New connection from {addr}, received the following request: {raw_request}"),
    )).await?;

    // parse request header
    let san_request = {
        if let Some(substr) = raw_request.split_once(REQUEST_TAIL) {
            substr.0
        } else {
            log_channel_send.send(NpLog::now(
                LogFrom::Net, LogType::Warning,
                "Request did not have REQUEST_TAIL or was truncated, so will assume as the entire header".to_string(),
            )).await?;
            &raw_request
        }
    };
    if let Some(first_line) = san_request.split("\n").into_iter().next() {
        let parts: Vec<_> = first_line.split_whitespace().collect();
        if parts.len() != 3 {
            log_channel_send.send(NpLog::now(
                LogFrom::Net, LogType::Error,
                "Request was malformed".to_string(),
            )).await?;
            anyhow::bail!("Request was malformed"); // TODO: These &'static str error messages should be changed to error enums
        }
        let req_type = parts[0];
        let req_ip = parts[1];
        let req_protocol = parts[2];
        log_channel_send.send(NpLog::now(
            LogFrom::Net, LogType::Info,
            format!("Type: {req_type}, IP/Domain: {req_ip}, Protocol: {req_protocol}"),
        )).await?;

        let mut server = TcpStream::connect(req_ip).await?;
        match RequestType::try_from(req_type) {
            Ok(req_type) => {
                match req_type {
                    RequestType::Connect => {
                        log_channel_send.send(NpLog::now(
                            LogFrom::Net, LogType::Verbose,
                            "Writing 200 connection established".to_string(),
                        )).await?;
                        client.write_all(&format!("{req_protocol} 200 Connection Established{REQUEST_TAIL}").into_bytes()).await?;

                        log_channel_send.send(NpLog::now(
                            LogFrom::Net, LogType::Verbose,
                            format!("Tunnel starting: {req_ip}"),
                        )).await?;
                        io::copy_bidirectional(&mut client, &mut server).await?;
                        log_channel_send.send(NpLog::now(
                            LogFrom::Net, LogType::Verbose,
                            format!("Tunnel ended: {req_ip}"),
                        )).await?;
                    },
                }
            },
            Err(something_else) => {
                log_channel_send.send(NpLog::now(
                    LogFrom::Net, LogType::Error,
                    format!("Unsupported protocol: {something_else}"),
                )).await?;
                anyhow::bail!("Unsupported protocol: {something_else}");
            },
        }
    } else {
        log_channel_send.send(NpLog::now(
            LogFrom::Net, LogType::Error,
            "Request was empty".to_string(),
        )).await?;
        anyhow::bail!("Request was empty");
    };

    drop(log_channel_send);
    Ok(())
}