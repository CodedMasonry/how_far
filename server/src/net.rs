use std::{borrow::BorrowMut, collections::HashMap, net::SocketAddr};

use crate::database;
use httparse::Header;
use log::{debug, info};
use rustls::crypto::hash::Hash;
use thiserror::Error;
use tokio::{
    io::{copy, sink, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
use tokio_rustls::server::TlsStream;

#[derive(Clone, Debug, PartialEq)]
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
    pub method: RequestMethod,
    pub path: &'a str,
    pub headers: HashMap<&'a str, &'a [u8]>,
    pub body: Option<Vec<u8>>,
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

    let mut req = parse_request(req.as_bytes()).await?;
    if req.method == RequestMethod::POST {
        let content_length =
            String::from_utf8(req.headers.get("content-length").unwrap().to_vec())?
                .parse::<usize>()?;
        let mut body = vec![0; content_length];
        reader.read_exact(&mut body);
        req.body = Some(body);
    }

    stream
        .write_all(
            concat!(
                "HTTP/2.0 200 ok\r\n",
                "Content-Type: text/html;\r\n",
                "Accept-Encoding: br\r\n",
                "\r\n",
            )
            .as_bytes(),
        )
        .await?;

    stream.flush().await?;
    stream.shutdown().await?;
    debug!("{}: {:?}",peer_addr req);

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
async fn parse_request(data: &[u8]) -> anyhow::Result<RequestData> {
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
        Some(v) => v,
        None => {
            return Err(NetError::InvalidRequest("No Path Specified".to_string()).into());
        }
    };

    let mut mapped_headers = HashMap::new();
    for header in req.headers {
        mapped_headers.insert(header.name, header.value);
    }

    Ok(RequestData {
        method,
        headers: mapped_headers,
        body: None,
        path,
    })
}

/// Attempts to get command queue for the request
/// Returns Hex encoded JobData OR empty string 
async fn fetch_queue(
    request: &RequestData,
    stream: &mut TlsStream<TcpStream>,
) -> anyhow::Result<String> {
    let id = match database::parse_agent_id(request).await? {
        Some(v) => v,
        None => return Ok(String::new()),
    };



    return Ok(String::new());
}
