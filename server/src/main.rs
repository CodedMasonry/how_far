use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use how_far_server::{get_cert, terminal};
use log::{debug, error};
use tokio::io::{self, copy, sink, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

#[derive(Parser, Debug)]
#[clap(name = "server")]
struct Opt {
    /// TLS private key in PEM format
    #[arg(short = 'k', long = "key", requires = "cert")]
    key: Option<PathBuf>,
    /// TLS certificate in PEM format
    #[arg(short = 'c', long = "cert", requires = "key")]
    cert: Option<PathBuf>,
    /// Address to listen on
    #[arg(long = "listen", default_value = "0.0.0.0:3000")]
    listen: SocketAddr,
}

#[tokio::main]
async fn main() {
    debug!("starting");
    env_logger::init();

    let opt = Opt::parse();

    // Server
    tokio::spawn(async move {
        if let Err(e) = run_listener(opt).await {
            error!("ERROR: {e}");
        }
    });

    terminal::tui().await.unwrap();
}

async fn run_listener(options: Opt) -> anyhow::Result<()> {
    let (certs, key) = get_cert()?;
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    debug!("Starting server");

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(options.listen).await.unwrap();
    debug!("listening");

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();

        let fut = async move {
            let mut stream = acceptor.accept(stream).await?;

            let mut output = sink();
            stream
                .write_all(
                    &b"HTTP/1.0 200 ok\r\n\
                    Connection: close\r\n\
                    Content-length: 12\r\n\
                    \r\n\
                    Hello world!"[..],
                )
                .await?;
            stream.shutdown().await?;
            copy(&mut stream, &mut output).await?;
            println!("Hello: {}", peer_addr);

            Ok(()) as io::Result<()>
        };

        tokio::spawn(async move {
            if let Err(err) = fut.await {
                error!("[*] Error with stream: {:?}", err);
            }
        });
    }
}
