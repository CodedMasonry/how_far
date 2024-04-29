use std::{
    env,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use how_far_types::DB_TABLE;
use how_far_types::{DATA_FOLDER, DB_FILE};
use rand::{rngs::ThreadRng, Rng};
use redb::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    save_server_cert()?;

    if cfg!(not(debug_assertions)) {
        println!("cargo:rerun-if-changed=NULL");
        let mut rng = rand::thread_rng();
        let id = generate_id(&mut rng)?;
        let id_file = Path::new(&out_dir).join("c.d");


        let obfuscated = obfuscate_id(id, &mut rng);
        fs::write(id_file, obfuscated)?;
    }

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

    fs::write(server_dest_path, contents.first().unwrap()).expect("Failed to write to out_dir");
    Ok(())
}

fn generate_id(rng: &mut ThreadRng) -> Result<u32, Box<dyn std::error::Error>> {
    let id: u32 = rng.gen();

    let init_data: how_far_types::ImplantInfo = how_far_types::ImplantInfo {
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

fn obfuscate_id(id: u32, rng: &mut ThreadRng) -> String {
    let mut str = String::new();
    str.push_str(&rng.gen::<u32>().to_string());
    str.push_str(&id.to_string());

    let mut i = rng.gen_range(0..32);
    while i > 0 {
        str.push_str(&rng.gen::<u8>().to_string());
        i -= 1;
    }

    str
}