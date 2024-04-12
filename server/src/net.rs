use std::net::SocketAddr;

use tokio::{
    io::{copy, sink, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::server::TlsStream;

use crate::database;

#[derive(Clone, Debug)]
enum RequestMethod {
    GET,
    POST,
}

#[derive(Clone, Debug)]
struct RequestData<'a> {
    method: RequestMethod,
    headers: Vec<String>,
    body: Option<&'a [u8]>,
}

pub async fn handle_request(
    mut stream: TlsStream<TcpStream>,
    peer_addr: SocketAddr,
) -> anyhow::Result<()> {
    let mut request = [0; 4096];
    stream.read(&mut request).await?;

    stream
        .write_all(
            concat!(
                "HTTP/1.0 200 ok\r\n",
                "Content-Type: text/html;\r\n",
                "Accept-Encoding: br\r\n",
                "\r\n",
            )
            .as_bytes(),
        )
        .await?;

    stream.flush().await?;
    stream.shutdown().await?;
    println!("Hello: {}", peer_addr);

    Ok(())
}

async fn parse_request(data: &mut [u8]) -> anyhow::Result<RequestData> {
    let header: Vec<&[u8]> = data.split(|byte| byte == &b'\n').collect::<Vec<&[u8]>>();

    Ok(RequestData {
        method: RequestMethod::GET,
        headers: Vec::new(),
        body: None,
    })
}

/*
async fn process_request(
    printer: &Option<String>,
    settings: Arc<Settings>,
    recv: RecvStream,
) -> Result<Vec<u8>> {
    let mut reader = BufReader::new(recv);
    let mut name = String::new();
    loop {
        let r = reader.read_line(&mut name).await.unwrap();
        if r < 3 {
            break;
        }
    }

    let mut extension = String::new();
    let mut session_id = String::new();
    let mut request_context = String::new();
    let linesplit = name.split("\n");
    // Parse some headers
    for l in linesplit {
        if l.starts_with("Extension") {
            // Extension Header
            let sizeplit = l.split(":");
            for s in sizeplit {
                if !(s.starts_with("Extension")) {
                    extension = s.trim().parse::<String>().unwrap();
                }
            }
        } else if l.starts_with("Session") {
            // Session Header
            let sizeplit = l.split(":");
            for s in sizeplit {
                if !(s.starts_with("Session")) {
                    session_id = s.trim().parse::<String>().unwrap();
                }
            }
        } else if l.starts_with("POST") {
            // if POST
            request_context = String::from("print")
        } else if l.starts_with("GET") && l.contains("auth") {
            // if AUTH
            request_context = String::from("auth")
        }
    }
}
*/

async fn handle_queue(stream: &mut TlsStream<TcpStream>) -> anyhow::Result<String> {
    let agent = database::fetch_agent(0)?;
    return Ok(String::new());
}
