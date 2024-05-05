use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use anyhow::anyhow;
use axum::http::{header, HeaderMap};
use base64::prelude::*;
use how_far_types::ImplantInfo;
use how_far_types::DB_FILE;
use how_far_types::DB_TABLE;
use log::debug;
use log::error;
use redb::ReadableTable;

pub static IMPLANT_DB: LazyLock<DataBase> = LazyLock::new(|| {
    let db = DataBase::build(DB_FILE.to_path_buf()).expect("failed to create database");
    db.init_table().expect("failed to create table");

    db
});

pub struct DataBase {
    pub db_path: PathBuf,
}

impl DataBase {
    pub fn build(db_path: PathBuf) -> Result<Self, anyhow::Error> {
        Ok(DataBase { db_path })
    }

    fn lock(&self) -> Result<redb::Database, redb::DatabaseError> {
        redb::Database::create(self.db_path.as_path())
    }

    /// helper function to initialize the table if it doesn't exist
    pub fn init_table(&self) -> Result<(), anyhow::Error> {
        let txn = self.lock()?.begin_write()?;
        txn.open_table(DB_TABLE)?;
        txn.commit()?;
        Ok(())
    }

    pub async fn list_implants(&self) -> anyhow::Result<Vec<(u32, ImplantInfo)>> {
        let txn = self.lock()?.begin_read()?;
        let table = txn.open_table(DB_TABLE)?;
        let mut result = Vec::new();

        for implant in table.iter()? {
            if let Ok(record) = implant {
                if let Ok(info) =
                    postcard::from_bytes::<how_far_types::ImplantInfo>(record.1.value())
                {
                    result.push((record.0.value(), info));
                }
            }
        }

        Ok(result)
    }

    pub async fn fetch_implant(&self, id: u32) -> anyhow::Result<Option<ImplantInfo>> {
        let txn = self.lock()?.begin_read()?;
        let table = txn.open_table(DB_TABLE)?;
        match table.get(id)? {
            Some(val) => {
                let serialized: ImplantInfo = postcard::from_bytes(val.value())?;
                Ok(Some(serialized))
            }
            None => Ok(None),
        }
    }

    pub async fn update_implant(&self, id: u32, info: &ImplantInfo) -> anyhow::Result<()> {
        let txn = self.lock()?.begin_write()?;
        {
            let mut table = txn.open_table(DB_TABLE)?;

            let serialized = postcard::to_allocvec(info)?;

            table.insert(id, &*serialized)?;
        }

        txn.commit()?;
        Ok(())
    }

    pub async fn key_exists(&self, id: u32) -> anyhow::Result<bool> {
        let read_txn = self.lock()?.begin_read()?;
        let table = read_txn.open_table(DB_TABLE)?;

        match table.get(id)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Parse the request for Implant Id
    /// Returns Ok(None) if Implant doesn't exist
    pub async fn parse_implant_id(&self, headers: &HeaderMap) -> anyhow::Result<Option<u32>> {
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

        let exists = match self.key_exists(id).await {
            Ok(v) => v,
            Err(e) => {
                error!("Error reading database: {:?}", e);
                return Ok(None);
            }
        };

        if exists {
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }
}

/// Junk Page offset set to 8 random bytes (Two u32 numbers)
pub async fn deobfuscate_id(str: &str) -> Result<u32, anyhow::Error> {
    let bytes: Vec<u8> = BASE64_URL_SAFE_NO_PAD.decode(str)?;
    if bytes.len() < 8 {
        return Err(anyhow!("Invalid id size"));
    }

    let decoded = as_u32_be(&bytes[4..8]);
    debug!("id decoded: {}", decoded);

    Ok(decoded)
}

fn as_u32_be(array: &[u8]) -> u32 {
    ((array[0] as u32) << 24)
        + ((array[1] as u32) << 16)
        + ((array[2] as u32) << 8)
        + (array[3] as u32)
}
