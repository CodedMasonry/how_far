use std::{
    env,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let server_dest_path = Path::new(&out_dir).join("cert.der");

    let project_dir = directories::ProjectDirs::from("com", "codedmasonry", "how_far").unwrap();
    let server_cert = project_dir.data_local_dir().to_path_buf().join("certs");

    let contents =
        rustls_pemfile::certs(&mut BufReader::new(&mut File::open(server_cert.join("cert.pem"))?))
            .collect::<Result<Vec<_>, _>>()?;

    fs::write(&server_dest_path, contents.first().unwrap()).expect("Failed to write to out_dir");

    Ok(())
}
