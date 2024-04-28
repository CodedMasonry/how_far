use std::io::Read;
use std::sync::Arc;

use rustls::crypto::{aws_lc_rs as provider, CryptoProvider};
use rustls::pki_types::CertificateDer;
use rustls::RootCertStore;

pub fn run() -> anyhow::Result<()> {
    let agent = ureq::AgentBuilder::new()
        .tls_config(Arc::new(fetch_config()))
        .build();
    let response = agent.get("https://localhost:8443/").call()?;


    println!("{:?}", response);
    let mut body_bytes = Vec::with_capacity(
        response
            .header("content-length")
            .unwrap()
            .parse::<usize>()?,
    );
    response.into_reader().read_to_end(&mut body_bytes)?;

    //let body: NetJobList = postcard::from_bytes(&body_bytes)?;

    // println!("{:?}", body);
    Ok(())
}

pub fn fetch_config() -> rustls::client::ClientConfig {
    // Handle importing root cert
    let trusted = include_bytes!(concat!(env!("OUT_DIR"), "/cert.der"));
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(trusted.to_vec())).unwrap();

    // Handle rustls config
    rustls::client::ClientConfig::builder_with_provider(
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
    .with_no_client_auth()
}

pub fn as_u32_be(array: &[u8]) -> u32 {
    ((array[0] as u32) << 24)
        + ((array[1] as u32) << 16)
        + ((array[2] as u32) << 8)
        + (array[3] as u32)
}