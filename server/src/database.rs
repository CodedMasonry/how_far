use std::{collections::HashMap, path::PathBuf};

use crate::net::RequestData;
use how_far_types::AgentInfo;
use redb::{Database, TableDefinition};
use std::sync::LazyLock;
use tokio::io::AsyncReadExt;

/// Key: u32 and Value: Byte array (postcard serialized) of AgentInfo
const TABLE: TableDefinition<u32, &[u8]> = TableDefinition::new("agents");
static DB_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    crate::DATA_FOLDER
        .data_local_dir()
        .to_path_buf()
        .join("db.redb")
});

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

pub async fn fetch_agent(id: u32) -> anyhow::Result<Option<AgentInfo>> {
    let db = Database::create(DB_FILE.as_path())?;

    let txn = db.begin_read()?;
    let table = txn.open_table(TABLE)?;

    match table.get(id)? {
        Some(val) => {
            let serialized: AgentInfo = postcard::from_bytes(val.value())?;
            return Ok(Some(serialized));
        }
        None => return Ok(None),
    };
}

pub async fn update_agent(id: u32, info: &AgentInfo) -> anyhow::Result<()> {
    let db = Database::create(DB_FILE.as_path())?;

    let txn = db.begin_write()?;
    let mut table = txn.open_table(TABLE)?;

    let serialized = postcard::to_allocvec(info)?;

    table.insert(id, &*serialized)?;
    Ok(())
}

pub async fn key_exists(id: u32) -> anyhow::Result<bool> {
    let db = Database::create(DB_FILE.as_path())?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(TABLE)?;

    match table.get(id)? {
        Some(_) => return Ok(true),
        None => return Ok(false),
    };
}

/// Parse the request for agent Id
/// Returns Ok(None) if agent doesn't exist
pub async fn parse_agent_id(req: &mut RequestData<'_>) -> anyhow::Result<Option<u32>> {
    let cookies_head = match req.headers.get_mut("Cookie") {
        Some(v) => v,
        None => return Ok(None),
    };

    let mut cookie_str = String::new();
    cookies_head.read_to_string(&mut cookie_str).await?;
    // explicit drop as quicker cleanup

    let mut mapped_cookies = HashMap::new();
    for mut cookie in cookie_str.split(';').map(|v| v.split('=')) {
        let k = cookie.next().unwrap();
        let v = cookie.next().unwrap();
        mapped_cookies.insert(k, v);
    }

    let id = mapped_cookies.get("__secure");
    let id = match id {
        Some(v) => v,
        None => return Ok(None),
    }
    .parse::<u32>()?;

    if key_exists(id).await? {
        return Ok(Some(id));
    } else {
        return Ok(None);
    }
}
