use std::{collections::HashMap, path::PathBuf};

use axum::http::{header, HeaderMap};
use how_far_types::AgentInfo;
use log::error;
use redb::{Database, TableDefinition};
use std::sync::LazyLock;
use base64::prelude::*;


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
pub async fn parse_agent_id(headers: &HeaderMap) -> anyhow::Result<Option<u32>> {
    let cookies_head = match headers.get(header::COOKIE) {
        Some(v) => v,
        None => return Ok(None),
    };

    let cookie_str = cookies_head.to_str()?;
    // explicit drop as quicker cleanup

    let mut mapped_cookies = HashMap::new();
    for mut cookie in cookie_str.split(';').map(|v| v.split('=')) {
        let k = cookie.next().unwrap_or_default();
        let v = cookie.next().unwrap_or_default();
        mapped_cookies.insert(k, v);
    }

    let id = match mapped_cookies.get("__secure") {
        Some(v) => v,
        None => return Ok(None),
    };

    let id = deobfuscate_id(id).await?;

    let exists = match key_exists(id).await {
        Ok(v) => v,
        Err(e) => {
            error!("Error fetching database: {:?}", e);
            return Ok(None);
        }
    };

    if exists {
        return Ok(Some(id));
    } else {
        return Ok(None);
    }
}


const _U32_BYTES: u32 = u32::BITS / 4;
pub async fn deobfuscate_id(str: &str) -> Result<u32, base64::DecodeError> {
    let decoded = BASE64_STANDARD.decode(str)?;
    Ok(as_u32_be(&decoded[0..4]))
}

fn as_u32_be(array: &[u8]) -> u32 {
    ((array[0] as u32) << 24) +
    ((array[1] as u32) << 16) +
    ((array[2] as u32) <<  8) +
    ((array[3] as u32) <<  0)
}