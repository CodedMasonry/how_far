use std::{
    env,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use how_far_types::DB_TABLE;
use how_far_types::{DATA_FOLDER, DB_FILE};
use redb::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    save_server_cert()?;

    let id = generate_id()?;
    let id_file = Path::new(&out_dir).join("c_id");
    fs::write(&id_file, id.to_string())?;

    Ok(())
}

fn save_server_cert() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let server_dest_path = Path::new(&out_dir).join("cert.der");

    let server_cert = DATA_FOLDER.data_local_dir().join("certs");

    let contents = rustls_pemfile::certs(&mut BufReader::new(&mut File::open(
        server_cert.join("cert.pem"),
    )?))
    .collect::<Result<Vec<_>, _>>()?;

    fs::write(&server_dest_path, contents.first().unwrap()).expect("Failed to write to out_dir");
    Ok(())
}

fn generate_id() -> Result<u32, Box<dyn std::error::Error>> {
    let id: u32 = get_random_u32()?;
    let init_data: how_far_types::AgentInfo = how_far_types::AgentInfo {
        last_check: None,
        queue: Vec::new(),
    };
    let db = Database::create(DB_FILE.as_path())?;

    let txn = db.begin_write()?;
    let mut table = txn.open_table(DB_TABLE)?;

    let serialized: Vec<u8> = postcard::to_allocvec(&init_data)?;

    table.insert(id, &*serialized)?;

    Ok(id)
}

fn get_random_u32() -> Result<u32, getrandom::Error> {
    let mut buf = [0u8; 4];
    getrandom::getrandom(&mut buf)?;

    Ok(as_u32_be(&buf))
}

fn as_u32_be(array: &[u8]) -> u32 {
    ((array[0] as u32) << 24)
        + ((array[1] as u32) << 16)
        + ((array[2] as u32) << 8)
        + ((array[3] as u32) << 0)
}
