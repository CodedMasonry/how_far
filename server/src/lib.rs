#![feature(fs_try_exists)]
#![feature(lazy_cell)]
pub mod database;
pub mod net;
pub mod terminal;

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::{debug, error, info};
use rcgen::{date_time_ymd, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use std::sync::LazyLock;
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    str::SplitWhitespace,
};
use thiserror::Error;
use how_far_types::DATA_FOLDER;

pub static CERTS: LazyLock<PathBuf> =
    LazyLock::new(|| DATA_FOLDER.data_local_dir().to_path_buf().join("certs"));
static COMMANDS_SET: LazyLock<Vec<Box<dyn Command>>> = LazyLock::new(|| {
    let temp_set: Vec<Box<dyn Command>> = vec![];

    //temp_set.append();
    temp_set
});

/// Error for terminal
#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("Command Doesn't Exist")]
    NonExistant,

}

/// Error for terminal
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Database Error")]
    DatabaseError(#[from] redb::Error),

    #[error("Parsing Error")]
    ParsingError(String),
}


/// Generic error casting for http returns
pub struct GenericError(anyhow::Error);

/// Command is the default template for command modules
/// Sub commands are EXPECTED to be handled by the run fn
/// help fn expects module to print it's own help message (default help message functions will be provided soon)
/// name fn is simply for indexing purposes (should return name of command)
#[async_trait]
pub trait Command: Send + Sync {
    async fn run(&self, args: SplitWhitespace<'_>) -> Result<()>;
    fn description(&self) -> String;
    fn name(&self) -> String;
}

/// Parsing generic errors into axum errors
impl IntoResponse for GenericError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, GenericError>` for axum returns
impl<E> From<E> for GenericError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub async fn generate_cert() -> anyhow::Result<()> {
    info!("Generating self-signed certificates");
    let mut params: CertificateParams = Default::default();
    params.not_before = date_time_ymd(1975, 1, 1);
    params.not_after = date_time_ymd(4096, 1, 1);
    params.distinguished_name = DistinguishedName::new();
    params
        .distinguished_name
        .push(DnType::OrganizationName, "hardcoded analytics");
    params
        .distinguished_name
        .push(DnType::CommonName, "data server");
    params.subject_alt_names = vec![SanType::DnsName("localhost".try_into()?)];

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    let pem_serialized = cert.pem();
    fs::create_dir_all(CERTS.as_os_str())?;
    fs::write(CERTS.join("cert.pem"), pem_serialized.as_bytes())?;
    fs::write(CERTS.join("key.pem"), key_pair.serialize_pem().as_bytes())?;

    debug!("cert written to {}", CERTS.to_string_lossy());
    Ok(())
}

pub async fn get_cert() -> anyhow::Result<(
    Vec<rustls_pki_types::CertificateDer<'static>>,
    rustls_pki_types::PrivateKeyDer<'static>,
)> {
    // Check if certs already generated
    fs::create_dir_all(CERTS.as_os_str())?;
    if !fs::try_exists(CERTS.join("cert.pem"))? {
        debug!("Certs don't exist; generating...");
        generate_cert().await?;
    }

    let certs = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(
        CERTS.join("cert.pem"),
    )?))
    .collect::<Result<Vec<_>, _>>()?;
    let private_key =
        rustls_pemfile::private_key(&mut BufReader::new(&mut File::open(CERTS.join("key.pem"))?))?
            .unwrap();

    Ok((certs, private_key))
}
