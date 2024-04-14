use std::path::PathBuf;

use how_far_types::AgentInfo;
use redb::{Database, Error, TableDefinition};
use std::sync::LazyLock;

/// Key: u32 and Value: Byte array (postcard serialized) of AgentInfo
const TABLE: TableDefinition<u32, &[u8]> = TableDefinition::new("agents");
static DB_FILE: LazyLock<PathBuf> = LazyLock::new(|| crate::DATA_FOLDER
    .data_local_dir()
    .to_path_buf()
    .join("db.redb"));

pub struct AgentDataBase<'a> {
    pub db: Database,
    pub table: TableDefinition<'a, u32, &'static [u8]>,
}

impl AgentDataBase<'_> {
    /// May fail if DB_FILE is corrupted or invalid
    pub fn build() -> Result<Self, redb::Error> {
        let db = Database::create(DB_FILE.as_path())?;

        Ok(AgentDataBase { db, table: TABLE })
    }
}

pub fn fetch_agent(id: u32) -> anyhow::Result<Option<AgentInfo>> {
    let db = Database::create(DB_FILE.as_path())?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TABLE)?;

    match table.get(0)? {
        Some(val) => {
            let serialized: AgentInfo = postcard::from_bytes(val.value())?;
            return Ok(Some(serialized));
        }
        None => return Ok(None),
    };
}
