use std::{
    env,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("cert.der");

    let binding = directories::ProjectDirs::from("com", "codedmasonry", "how_far").unwrap();
    let cert = binding.data_local_dir().to_path_buf().join("certs");

    let contents =
        rustls_pemfile::certs(&mut BufReader::new(&mut File::open(cert.join("cert.pem"))?))
            .collect::<Result<Vec<_>, _>>()?;

    fs::write(&dest_path, contents.first().unwrap()).expect("Failed to write to out_dir");

    Ok(())
}
