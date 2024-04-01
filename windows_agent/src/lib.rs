use std::io::{stdout, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use rustls::pki_types::CertificateDer;
use rustls::RootCertStore;

pub fn run() -> anyhow::Result<()> {
    // Handle importing root cert
    let trusted = include_bytes!(concat!(env!("OUT_DIR"), "/cert.der"));
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(trusted.to_vec()))?;

    // Handle rustls config
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(RootCertStore::from(roots))
        .with_no_client_auth();

    let server_name = "localhost".try_into().unwrap();
    let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
    let mut sock = TcpStream::connect("localhost:3000")?;
    let mut tls = rustls::Stream::new(&mut conn, &mut sock);
    tls.write_all(
        concat!(
            "GET / HTTP/1.1\r\n",
            "Host: localhost\r\n",
            "Connection: close\r\n",
            "Accept-Encoding: identity\r\n",
            "\r\n"
        )
        .as_bytes(),
    )
    .unwrap();
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

    Ok(())
}
