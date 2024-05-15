#![feature(fs_try_exists)]
#![feature(lazy_cell)]
pub mod commands;
pub mod database;
pub mod net;
pub mod terminal;
pub use database::DataBase;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use how_far_types::{ImplantInfo, DATA_FOLDER};
use log::{debug, error, info, Level, LevelFilter};
use nu_ansi_term::Color;
use rcgen::{date_time_ymd, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use reedline::ExternalPrinter;
use tokio::sync::Mutex;
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};
use std::sync::LazyLock;

use thiserror::Error;

pub static LOG_FILE: LazyLock<String> = LazyLock::new(|| format!("{}.log", env!("CARGO_PKG_NAME")));
pub static CERTS: LazyLock<PathBuf> =
    LazyLock::new(|| DATA_FOLDER.data_local_dir().to_path_buf().join("certs"));
pub static SELECTED_AGENT: LazyLock<Mutex<Option<(u32, ImplantInfo)>>> = LazyLock::new(|| Mutex::new(None));

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
    target: Option<String>,
}

impl TerminalLogger {
    /// Creates a new Terminal Logger
    pub fn new(printer: ExternalPrinter<String>, filter: LevelFilter) -> Self {
        TerminalLogger { printer, filter, target: None }
    }

    /// Creates a new Terminal Logger with specific target
    pub fn wtih_target(printer: ExternalPrinter<String>, filter: LevelFilter, target: String) -> Self {
        TerminalLogger { printer, filter, target: Some(target) }
    }

    /// Sets log max level
    pub fn init(&self) {
        log::set_max_level(self.filter);
    }
}

impl log::Log for TerminalLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // Anything more verbose than set filter is ignored
        if metadata.level() > self.filter {
            return false;
        }
        
        match &self.target {
            Some(target) => metadata.target().contains(target),
            None => true,
        }
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let msg = format!(
                "{} {}: {}",
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

pub fn color_level<'a>(level: Level) -> nu_ansi_term::AnsiGenericString<'a, str> {
    match level {
        Level::Error => Color::Red.paint("[-]"),
        Level::Warn => Color::Yellow.paint("[*]"),
        Level::Info => Color::Green.paint("[+]"),
        Level::Debug => Color::Blue.paint("[*]"),
        Level::Trace => Color::Purple.paint("[*]"),
    }
}

fn format_target<'a>(target: &str) -> nu_ansi_term::AnsiGenericString<'a, str> {
    let target = target.replace("::", "/");
    Color::DarkGray.paint(target)
}
