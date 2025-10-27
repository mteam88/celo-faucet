use anyhow::{Context, Result};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Store {
    db: sled::Db,
}

impl Store {
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path).context("Failed to open sled database")?;
        Ok(Self { db })
    }

    pub fn has_received(&self, address: &str) -> Result<bool> {
        let key = format!("addr:{}", address.to_lowercase());
        Ok(self.db.contains_key(key)?)
    }

    pub fn mark_received(&self, address: &str) -> Result<()> {
        let key = format!("addr:{}", address.to_lowercase());
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .to_string();
        self.db.insert(key, timestamp.as_bytes())?;
        Ok(())
    }
}

