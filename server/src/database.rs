use std::collections::HashMap;

use anyhow::anyhow;
use axum::http::{header, HeaderMap};
use base64::prelude::*;
use how_far_types::AgentInfo;
use how_far_types::DB_FILE;
use how_far_types::DB_TABLE;
use log::{debug, error};
use redb::Database;

pub async fn fetch_agent(id: u32) -> anyhow::Result<Option<AgentInfo>> {
    let db = Database::create(DB_FILE.as_path())?;

    let txn = db.begin_read()?;
    let table = txn.open_table(DB_TABLE)?;

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
    let mut table = txn.open_table(DB_TABLE)?;

    let serialized = postcard::to_allocvec(info)?;

    table.insert(id, &*serialized)?;
    Ok(())
}

pub async fn key_exists(id: u32) -> anyhow::Result<bool> {
    let db = Database::create(DB_FILE.as_path())?;

    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(DB_TABLE)?;

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

/// Junk Page offset set to 8 random bytes (Two u32 numbers)
pub async fn deobfuscate_id(str: &str) -> Result<u32, anyhow::Error> {
    let bytes: Vec<u8> = BASE64_STANDARD.decode(str)?;
    if bytes.len() < 12 {
        return Err(anyhow!("Invalid id size"));
    }

    let decoded = as_u32_be(&bytes[8..12]);
    debug!("id decoded: {}", decoded);

    Ok(decoded)
}

fn as_u32_be(array: &[u8]) -> u32 {
    ((array[0] as u32) << 24)
        + ((array[1] as u32) << 16)
        + ((array[2] as u32) << 8)
        + ((array[3] as u32) << 0)
}
