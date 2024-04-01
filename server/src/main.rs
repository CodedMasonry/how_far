use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use how_far_server::get_cert;
use log::debug;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

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
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "how_far_server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let opt = Opt::parse();
    let code = {
        if let Err(e) = run(opt).await {
            eprintln!("ERROR: {e}");
            1
        } else {
            0
        }
    };

    std::process::exit(code);
}
async fn run(options: Opt) -> anyhow::Result<()> {
    debug!("generating certs");
    let (certs, key) = get_cert()?;
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    debug!("Starting server");

    let listener = TcpListener::bind(options.listen).unwrap();
    debug!("listening");
    let (mut stream, _) = listener.accept()?;
    debug!("conn. accepted");

    let mut conn = rustls::ServerConnection::new(Arc::new(config))?;
    conn.complete_io(&mut stream)?;

    conn.writer().write_all(b"Hello from the server")?;
    conn.complete_io(&mut stream)?;
    let mut buf = [0; 64];
    let len = conn.reader().read(&mut buf)?;
    println!("Received message from client: {:?}", &buf[..len]);

    Ok(())
}
