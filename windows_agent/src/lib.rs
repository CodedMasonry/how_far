use std::io::{stdout, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use rustls::crypto::{ring as provider, CryptoProvider};
use rustls::pki_types::CertificateDer;
use rustls::RootCertStore;

pub fn run() -> anyhow::Result<()> {
    // Handle importing root cert
    let trusted = include_bytes!(concat!(env!("OUT_DIR"), "/cert.der"));
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(trusted.to_vec()))?;

    // Handle rustls config
    let config = rustls::ClientConfig::builder_with_provider(
        CryptoProvider {
            cipher_suites: vec![provider::cipher_suite::TLS13_CHACHA20_POLY1305_SHA256],
            kx_groups: vec![provider::kx_group::X25519],
            ..provider::default_provider()
        }
        .into(),
    )
    .with_protocol_versions(&[&rustls::version::TLS13])
    .unwrap()
    .with_root_certificates(roots)
    .with_no_client_auth();

    let server_name = "localhost".try_into().unwrap();
    let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
    let mut sock = TcpStream::connect("localhost:3000").unwrap();
    let mut tls = rustls::Stream::new(&mut conn, &mut sock);
    tls.write_all(
        concat!(
            "GET / HTTP/1.1\r\n",
            "Host: www.rust-lang.org\r\n",
            "Connection: close\r\n",
            "Accept-Encoding: identity\r\n",
            "\r\n"
        )
        .as_bytes(),
    )
    .unwrap();
    tls.flush()?;

    let ciphersuite = tls.conn.negotiated_cipher_suite().unwrap();
    writeln!(
        &mut std::io::stderr(),
        "Current ciphersuite: {:?}",
        ciphersuite.suite()
    )
    .unwrap();
    let mut plaintext = Vec::new();
    tls.read_to_end(&mut plaintext).unwrap();
    stdout().write_all(&plaintext).unwrap();

    tls.sock.shutdown(std::net::Shutdown::Both)?;

    Ok(())
}
