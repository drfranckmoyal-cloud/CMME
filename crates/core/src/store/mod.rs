//! Stockage chiffré SQLCipher. Aucune ouverture en clair n'est possible : sans clé, erreur.

pub mod encounters;
pub mod listing;

use crate::error::{CoreError, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};

pub const SCHEMA_VERSION: i64 = 1;
const MIGRATIONS: [&str; 1] = [include_str!("schema_v1.sql")];

pub struct Store {
    pub(crate) conn: Connection,
    pub path: PathBuf,
    pub author: String,
}

pub fn now() -> String {
    chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, false)
}

pub fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Applique la clé brute (32 octets) puis les réglages de sécurité.
pub(crate) fn apply_key(conn: &Connection, key: &[u8; 32]) -> Result<()> {
    let hex = hex::encode(key);
    conn.execute_batch(&format!("PRAGMA key = \"x'{hex}'\";"))?;
    Ok(())
}

pub(crate) fn apply_passphrase(conn: &Connection, passphrase: &str) -> Result<()> {
    conn.pragma_update(None, "key", passphrase)?;
    Ok(())
}

pub(crate) fn check_readable(conn: &Connection) -> Result<()> {
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0)).map_err(|_| CoreError::BadSecret)?;
    Ok(())
}

pub(crate) fn harden(conn: &Connection) -> Result<()> {
    // Pas de fichier temporaire en clair, journal chiffré par SQLCipher, effacement sécurisé.
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA temp_store = MEMORY;
         PRAGMA secure_delete = ON;
         PRAGMA journal_mode = DELETE;
         PRAGMA synchronous = FULL;",
    )?;
    Ok(())
}

pub(crate) fn migrate(conn: &mut Connection) -> Result<()> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if current > SCHEMA_VERSION {
        return Err(CoreError::Refused(format!(
            "base créée par une version plus récente de l'application (schéma {current})"
        )));
    }
    for (i, sql) in MIGRATIONS.iter().enumerate() {
        let v = i as i64 + 1;
        if v <= current {
            continue;
        }
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute_batch(&format!("PRAGMA user_version = {v};"))?;
        tx.commit()?;
    }
    Ok(())
}

impl Store {
    /// Ouvre (ou crée si `create`) la base chiffrée avec une clé brute.
    pub fn open(path: &Path, key: &[u8; 32], create: bool, author: &str) -> Result<Store> {
        if !create && !path.exists() {
            return Err(CoreError::NotFound("base de données".into()));
        }
        if create && path.exists() {
            return Err(CoreError::Refused("une base existe déjà à cet emplacement".into()));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut conn = Connection::open(path)?;
        apply_key(&conn, key)?;
        check_readable(&conn)?;
        harden(&conn)?;
        migrate(&mut conn)?;
        let s = Store { conn, path: path.to_path_buf(), author: author.to_string() };
        if create {
            s.set_setting("created_at", &now())?;
        }
        Ok(s)
    }

    pub fn setting(&self, key: &str) -> Result<Option<String>> {
        Ok(self.conn.query_row("SELECT value FROM app_setting WHERE key = ?1", [key], |r| r.get(0)).optional()?)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO app_setting(key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn schema_version(&self) -> Result<i64> {
        Ok(self.conn.query_row("PRAGMA user_version", [], |r| r.get(0))?)
    }

    pub fn cipher_version(&self) -> Result<String> {
        Ok(self.conn.query_row("PRAGMA cipher_version", [], |r| r.get(0))?)
    }

    pub fn audit(&self, kind: &str, entity: &str, entity_id: Option<&str>, field: Option<&str>, old: Option<&str>, new: Option<&str>, reason: Option<&str>) -> Result<()> {
        audit_on(&self.conn, &self.author, kind, entity, entity_id, field, old, new, reason)
    }

    pub fn counts(&self) -> Result<serde_json::Value> {
        let c = |sql: &str| -> Result<i64> { Ok(self.conn.query_row(sql, [], |r| r.get(0))?) };
        Ok(serde_json::json!({
            "patients": c("SELECT count(*) FROM patient WHERE merged_into IS NULL")?,
            "encounters": c("SELECT count(*) FROM encounter")?,
            "field_values": c("SELECT count(*) FROM field_value")?,
            "audit_events": c("SELECT count(*) FROM audit_event")?,
            "source_documents": c("SELECT count(*) FROM source_document")?,
            "export_snapshots": c("SELECT count(*) FROM export_snapshot")?,
        }))
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn audit_on(conn: &Connection, author: &str, kind: &str, entity: &str, entity_id: Option<&str>, field: Option<&str>, old: Option<&str>, new: Option<&str>, reason: Option<&str>) -> Result<()> {
    conn.execute(
        "INSERT INTO audit_event(at, kind, entity, entity_id, field, old_value, new_value, reason, author) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![now(), kind, entity, entity_id, field, old, new, reason, author],
    )?;
    Ok(())
}
