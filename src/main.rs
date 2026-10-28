pub mod logging;
pub mod requests;

use time::Timestamp;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use crate::logging::datatypes::{LogFrom, LogType, NpLog};
use crate::requests::RequestType;

const BIND_ADDR: &'static str = "127.0.0.1:8080";
const REQUEST_TAIL: &'static str = "\r\n\r\n";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_file = logging::fileio::LogFile::from_date_time(Timestamp::now()).await?;

    let listener = TcpListener::bind(BIND_ADDR).await?;
    println!("Listening on {BIND_ADDR}");

    loop {
        let (sock, addr) = listener.accept().await?;
        let log_clo = log_file.try_clone().await?;

        tokio::spawn(async move {
            println!("New connection: {addr}");
            handle_connection(sock, log_clo).await.unwrap_or_else(|err| {
                panic!("{:?}", err) // TODO: How do we handle this?
            });
        });
    }
}

async fn handle_connection(
    mut client: TcpStream,
    mut log_file: logging::fileio::LogFile,
) -> anyhow::Result<()> {
    let mut buf = [0u8; 4096];
    let n = client.read(&mut buf).await?;
    let raw_request = String::from_utf8_lossy(&buf[..n]);
    log_file.write_log(NpLog::now(
        LogFrom::Net, LogType::Verbose,
        format!("Received the following request: {raw_request}"),
    )).await?;

    // parse request header
    let san_request = {
        if let Some(substr) = raw_request.split_once(REQUEST_TAIL) {
            substr.0
        } else {
            log_file.write_log(NpLog::now(
                LogFrom::Net, LogType::Warning,
                "Request did not have REQUEST_TAIL or was truncated, so will assume as the entire header".to_string(),
            )).await?;
            &raw_request
        }
    };
    if let Some(first_line) = san_request.split("\n").into_iter().next() {
        let parts: Vec<_> = first_line.split_whitespace().collect();
        if parts.len() != 3 {
            log_file.write_log(NpLog::now(
                LogFrom::Net, LogType::Error,
                "Request was malformed".to_string(),
            )).await?;
            anyhow::bail!("Request was malformed"); // TODO: These &'static str error messages should be changed to error enums
        }
        let req_type = parts[0];
        let req_ip = parts[1];
        let req_protocol = parts[2];
        log_file.write_log(NpLog::now(
            LogFrom::Net, LogType::Info,
            format!("Type: {req_type}, IP/Domain: {req_ip}, Protocol: {req_protocol}"),
        )).await?;

        let mut server = TcpStream::connect(req_ip).await?;
        match RequestType::from_str(req_type) {
            RequestType::Connect => {
                log_file.write_log(NpLog::now(
                    LogFrom::Net, LogType::Verbose,
                    "Writing 200 connection established".to_string(),
                )).await?;
                client.write_all(&format!("{req_protocol} 200 Connection Established{REQUEST_TAIL}").into_bytes()).await?;

                log_file.write_log(NpLog::now(
                    LogFrom::Net, LogType::Verbose,
                    format!("Tunnel starting: {req_ip}"),
                )).await?;
                io::copy_bidirectional(&mut client, &mut server).await?;
                log_file.write_log(NpLog::now(
                    LogFrom::Net, LogType::Verbose,
                    format!("Tunnel ended: {req_ip}"),
                )).await?;
            },
            RequestType::Unsupported(something_else) => {
                 log_file.write_log(NpLog::now(
                    LogFrom::Net, LogType::Error,
                    format!("Unsupported protocol: {something_else}"),
                )).await?;
                anyhow::bail!("Unsupported protocol: {something_else}");
            },
        }
    } else {
        log_file.write_log(NpLog::now(
            LogFrom::Net, LogType::Error,
            "Request was empty".to_string(),
        )).await?;
        anyhow::bail!("Request was empty");
    };

    drop(log_file);
    Ok(())
}