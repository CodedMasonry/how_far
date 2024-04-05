use std::path::PathBuf;

use redb::{Database, Error, TableDefinition};
use lazy_static::lazy_static;

/// Key: u32 and Value: Byte array (postcard serialized) of AgentInfo
const TABLE: TableDefinition<u32, &[u8]> = TableDefinition::new("agents");

lazy_static! {
    static ref DB_FILE: PathBuf = crate::DATA_FOLDER.data_local_dir().to_path_buf().join("db.redb");
}


pub struct AgentDataBase<'a> {
    pub db: Database,
    pub table: TableDefinition<'a, u32, &'static [u8]>,
}

impl AgentDataBase<'_> {
    /// May fail if DB_FILE is corrupted or invalid
    pub fn build() -> Result<Self, Error> {
        let db = Database::create(DB_FILE.as_path())?;
        
        Ok(AgentDataBase {
            db,
            table: TABLE,
        })
    }
}

pub fn add_client() -> Result<(), Error> {
    let db = Database::create(DB_FILE.as_path())?;
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(TABLE)?;
        table.insert(0, b"test".as_ref())?;
    }
    write_txn.commit()?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TABLE)?;
    assert_eq!(table.get(0)?.unwrap().value(), b"test");

    Ok(())
}


pub fn remove_client() -> Result<(), Error> {
    let db = Database::create(DB_FILE.as_path())?;
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(TABLE)?;
        table.insert(0, b"test".as_ref())?;
    }
    write_txn.commit()?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TABLE)?;
    assert_eq!(table.get(0)?.unwrap().value(), b"test");

    Ok(())
}

pub fn list_clients() -> Result<(), Error> {
    let db = Database::create(DB_FILE.as_path())?;
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(TABLE)?;
        table.insert(0, b"test".as_ref())?;
    }
    write_txn.commit()?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TABLE)?;
    assert_eq!(table.get(0)?.unwrap().value(), b"test");

    Ok(())
}