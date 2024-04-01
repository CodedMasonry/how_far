#![feature(fs_try_exists)]

use directories::ProjectDirs;
use lazy_static::lazy_static;
use log::debug;
use rcgen::{date_time_ymd, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

lazy_static! {
    static ref CERTS: PathBuf = ProjectDirs::from("com", "codedmasonry", "how_far")
        .unwrap()
        .data_local_dir()
        .to_path_buf()
        .join("certs");
}

pub fn generate_cert() -> anyhow::Result<()> {
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

pub fn get_cert() -> anyhow::Result<(
    Vec<rustls_pki_types::CertificateDer<'static>>,
    rustls_pki_types::PrivateKeyDer<'static>,
)> {
    // Check if certs already generated
    fs::create_dir_all(CERTS.as_os_str())?;
    if !fs::try_exists(CERTS.join("cert.pem"))? {
        debug!("Certs don't exist; generating...");
        generate_cert()?;
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
