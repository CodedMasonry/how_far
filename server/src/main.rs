use std::path::PathBuf;
use std::{net::SocketAddr, process};

use clap::Parser;
use how_far_server::{get_cert, terminal, TerminalLogger};
use log::{debug, error, info};
use reedline::ExternalPrinter;
use tower::ServiceBuilder;

use axum::{
    extract::ConnectInfo,
    http::{HeaderMap, StatusCode},
    routing::get,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::compression::CompressionLayer;

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
    let opt = Opt::parse();
    let printer = ExternalPrinter::new(20);
    let logger = Box::new(TerminalLogger::new(printer.clone(), log::LevelFilter::Debug));
    logger.init();
    log::set_boxed_logger(logger).expect("Failed to set logger");

    // Server
    tokio::spawn(async move {
        if let Err(e) = run_listener(opt).await {
            error!("ERROR: {e}");
        }
    });

    match terminal::tui(printer).await {
        Ok(_) => process::exit(0),
        Err(e) => {
            error!("{}", e);
            process::exit(0)
        }
    };
}

async fn run_listener(_options: Opt) -> anyhow::Result<()> {
    let (certs, key) = get_cert().await?;
    let certs: Vec<Vec<u8>> = certs.into_iter().map(|v| v.to_vec()).collect();

    debug!("Starting server");

    let config = RustlsConfig::from_der(certs, key.secret_der().to_vec()).await?;
    let app = Router::new()
        .route("/", get(root))
        .layer(ServiceBuilder::new().layer(CompressionLayer::new()));

    // run https server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8443));
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

async fn root(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> (StatusCode, Vec<u8>) {
    info!("{}: Connection Established", addr);
    let result = how_far_server::net::fetch_queue(&headers).await;

    match result {
        Ok(v) => (StatusCode::OK, v),
        Err(e) => {
            error!("Error: {:?}", e);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "something went wrong".into(),
            )
        }
    }
}
