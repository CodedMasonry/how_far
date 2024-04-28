#![feature(fs_try_exists)]
#![feature(lazy_cell)]
pub mod database;
pub mod net;
pub mod terminal;
pub mod commands;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use how_far_types::DATA_FOLDER;
use log::{debug, error, info, Level, LevelFilter};
use nu_ansi_term::Color;
use rcgen::{date_time_ymd, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use reedline::ExternalPrinter;
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};
use std::{sync::LazyLock, time::SystemTime};

use thiserror::Error;

pub static LOG_FILE: LazyLock<String> = LazyLock::new(|| format!("{}.log", env!("CARGO_PKG_NAME")));
pub static CERTS: LazyLock<PathBuf> =
    LazyLock::new(|| DATA_FOLDER.data_local_dir().to_path_buf().join("certs"));

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

pub struct TerminalLogger {
    printer: ExternalPrinter<String>,
    filter: LevelFilter,
}

impl TerminalLogger {
    /// Creates a new Terminal Logger
    pub fn new(printer: ExternalPrinter<String>, filter: LevelFilter) -> Self {
        TerminalLogger { printer, filter }
    }

    /// Sets log max level
    pub fn init(&self) {
        log::set_max_level(self.filter);
    }
}

impl log::Log for TerminalLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.filter
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let time = humantime::format_rfc3339_seconds(SystemTime::now());
            let msg = format!(
                "{} {} {}: {}",
                time,
                color_level(record.level()),
                format_target(record.target()),
                record.args()
            );

            self.printer
                .print(msg)
                .expect("printing to terminal failed");
        }
    }

    fn flush(&self) {}
}

fn color_level<'a>(level: Level) -> nu_ansi_term::AnsiGenericString<'a, str> {
    match level {
        Level::Error => Color::Red.paint("ERROR"),
        Level::Warn => Color::Yellow.paint("WARN"),
        Level::Info => Color::Green.paint("INFO"),
        Level::Debug => Color::Blue.paint("DEBUG"),
        Level::Trace => Color::Purple.paint("TRACE"),
    }
}

fn format_target<'a>(target: &'a str) -> String {
    target.replace("::", "/")
}
