use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;
use how_far_server::{get_cert, terminal};
use log::{debug, error, warn};

use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri},
    response::Redirect,
    routing::get,
    BoxError, Router,
};
use axum_server::tls_rustls::RustlsConfig;

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

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    // Server
    tokio::spawn(async move {
        if let Err(e) = run_listener(opt).await {
            error!("ERROR: {e}");
        }
    });

    terminal::tui().await.unwrap();
}

async fn run_listener(_options: Opt) -> anyhow::Result<()> {
    let (certs, key) = get_cert().await?;
    let certs: Vec<Vec<u8>> = certs.into_iter().map(|v| v.to_vec()).collect();

    debug!("Starting server");

    let ports = Ports {
        http: 8080,
        https: 8443,
    };
    // optional: spawn a second server to redirect http requests to this server
    tokio::spawn(redirect_http_to_https(ports));

    let config =
        RustlsConfig::from_der(certs, key.secret_der().to_vec()).await?;
    let app = Router::new().route("/", get(handler));

    // run https server
    let addr = SocketAddr::from(([0, 0, 0, 0], ports.https));
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn handler() -> &'static str {
    "Hello, World!"
}

async fn redirect_http_to_https(ports: Ports) {
    fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, ports) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                warn!("{} {}", error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.http));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, redirect.into_make_service())
        .await
        .unwrap();
}
