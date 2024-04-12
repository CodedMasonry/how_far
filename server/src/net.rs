use std::{borrow::BorrowMut, net::SocketAddr};

use httparse::Header;
use thiserror::Error;
use tokio::{
    io::{copy, sink, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use tokio_rustls::server::TlsStream;

use crate::database;

#[derive(Clone, Debug)]
pub enum RequestMethod {
    GET,
    POST,
    PUT,
}

#[derive(Error, Debug)]
pub enum NetError {
    #[error("Invalid request structure, {0}")]
    InvalidRequest(String),
}

#[derive(Clone, Debug)]
pub struct RequestData<'a> {
    method: RequestMethod,
    path: String,
    headers: Vec<Header<'a>>,
    body: Option<&'a [u8]>,
}

pub async fn handle_request(
    mut stream: TlsStream<TcpStream>,
    peer_addr: SocketAddr,
) -> anyhow::Result<()> {
    // Reader becomes body after request is parsed out
    let mut reader = BufReader::new(stream.borrow_mut());
    let mut req = String::new();
    loop {
        let r = reader.read_line(&mut req).await?;
        // If there are less than 3 chars in line
        if r < 3 {
            break;
        }
    }

    let req = parse_request(&mut req.into_bytes()).await?;

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

fn find_body_index(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|w| matches!(w, b"\r\n\r\n"))
        .map(|ix| ix + 4)
}

/// Parses requests except body
/// Body returned as None in all scenarios
async fn parse_request(data: &mut [u8]) -> anyhow::Result<RequestData> {
    let mut headers = [httparse::EMPTY_HEADER; 8];
    let mut req = httparse::Request::new(&mut headers);
    let mut res = req.parse(data)?;

    let method = match req.method {
        Some(v) => {
            if v == "GET" {
                RequestMethod::GET
            } else if v == "POST" {
                RequestMethod::POST
            } else {
                // Temporary catch-all
                RequestMethod::PUT
            }
        }
        None => {
            return Err(NetError::InvalidRequest("No Method Specified".to_string()).into());
        }
    };

    let path = match req.path {
        Some(v) => v.to_string(),
        None => {
            return Err(NetError::InvalidRequest("No Path Specified".to_string()).into());
        }
    };

    Ok(RequestData {
        method,
        headers: req.headers.to_vec(),
        body: None,
        path,
    })
}

async fn handle_queue(stream: &mut TlsStream<TcpStream>) -> anyhow::Result<String> {
    let agent = database::fetch_agent(0)?;
    return Ok(String::new());
}
