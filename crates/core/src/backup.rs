//! Sauvegarde et restauration chiffrées.
//! Une sauvegarde est une base SQLCipher autonome, chiffrée par une phrase de récupération
//! (dérivation PBKDF2-HMAC-SHA512 de SQLCipher) : elle se restaure sur un autre Mac sans le trousseau
//! d'origine. Aucune cryptographie maison.

use crate::error::{CoreError, Result};
use crate::store::{self, now, Store, SCHEMA_VERSION};
use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};

pub const MIN_PASSPHRASE: usize = 10;
pub const EXTENSION: &str = "cmmebak";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackupMeta {
    pub format: String,
    pub created_at: String,
    pub app_version: String,
    pub schema_version: i64,
    pub counts: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub path: String,
    pub size_bytes: u64,
    pub meta: BackupMeta,
}

fn raw_key_literal(key: &[u8; 32]) -> String {
    format!("x'{}'", hex::encode(key))
}

fn counts_of(conn: &Connection, schema: &str) -> Result<serde_json::Value> {
    let c = |t: &str, w: &str| -> Result<i64> { Ok(conn.query_row(&format!("SELECT count(*) FROM {schema}.{t} {w}"), [], |r| r.get(0))?) };
    Ok(serde_json::json!({
        "patients": c("patient", "WHERE merged_into IS NULL")?,
        "encounters": c("encounter", "")?,
        "field_values": c("field_value", "")?,
        "audit_events": c("audit_event", "")?,
        "source_documents": c("source_document", "")?,
        "export_snapshots": c("export_snapshot", "")?,
    }))
}

fn integrity(conn: &Connection) -> Result<()> {
    let mut st = conn.prepare("PRAGMA cipher_integrity_check")?;
    let errs: Vec<String> = st.query_map([], |r| r.get::<_, String>(0))?.collect::<rusqlite::Result<_>>()?;
    if !errs.is_empty() {
        return Err(CoreError::Refused("sauvegarde endommagée (contrôle d'intégrité du chiffrement)".into()));
    }
    let ok: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if ok != "ok" {
        return Err(CoreError::Refused("sauvegarde endommagée (contrôle d'intégrité)".into()));
    }
    Ok(())
}

fn fsync_dir(dir: &Path) {
    if let Ok(f) = File::open(dir) {
        let _ = f.sync_all();
    }
}

impl Store {
    /// Crée une sauvegarde cohérente (export SQLCipher depuis la connexion ouverte), vérifiée puis
    /// renommée atomiquement. En cas d'échec, aucun fichier final n'est laissé.
    pub fn create_backup(&self, dest_dir: &Path, passphrase: &str, app_version: &str) -> Result<BackupInfo> {
        if passphrase.chars().count() < MIN_PASSPHRASE {
            return Err(CoreError::validation("phrase de récupération", format!("{MIN_PASSPHRASE} caractères au moins")));
        }
        if !dest_dir.is_dir() {
            return Err(CoreError::Io("dossier de destination introuvable".into()));
        }
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        let final_path = dest_dir.join(format!("CMME-sauvegarde-{stamp}.{EXTENSION}"));
        if final_path.exists() {
            return Err(CoreError::Refused("une sauvegarde porte déjà ce nom ; réessayez dans une seconde".into()));
        }
        let tmp = dest_dir.join(format!(".cmme-sauvegarde-{}.tmp", store::new_id()));
        let result = (|| -> Result<BackupMeta> {
            let counts = counts_of(&self.conn, "main")?;
            let meta = BackupMeta { format: "CMME-BACKUP-1".into(), created_at: now(), app_version: app_version.into(), schema_version: self.schema_version()?, counts };
            self.conn.execute("ATTACH DATABASE ?1 AS bk KEY ?2", params![tmp.to_string_lossy(), passphrase])?;
            let r = (|| -> Result<()> {
                self.conn.query_row("SELECT sqlcipher_export('bk')", [], |_| Ok(()))?;
                self.conn.execute_batch(&format!("PRAGMA bk.user_version = {};", meta.schema_version))?;
                self.conn.execute_batch("CREATE TABLE bk.backup_meta (json TEXT NOT NULL);")?;
                self.conn.execute("INSERT INTO bk.backup_meta(json) VALUES (?1)", [serde_json::to_string(&meta)?])?;
                Ok(())
            })();
            self.conn.execute_batch("DETACH DATABASE bk;")?;
            r?;
            // Relecture complète avec la phrase : preuve que la sauvegarde est restaurable.
            let check = inspect_backup(&tmp, passphrase)?;
            if check.counts != meta.counts {
                return Err(CoreError::Refused("vérification de la sauvegarde : effectifs différents".into()));
            }
            File::open(&tmp)?.sync_all()?;
            Ok(meta)
        })();
        match result {
            Ok(meta) => {
                std::fs::rename(&tmp, &final_path)?;
                fsync_dir(dest_dir);
                let size = std::fs::metadata(&final_path)?.len();
                self.set_setting("last_backup_at", &meta.created_at)?;
                self.set_setting("last_backup_dir", &dest_dir.to_string_lossy())?;
                self.audit("backup", "app", None, None, None, Some(&final_path.file_name().unwrap().to_string_lossy()), None)?;
                Ok(BackupInfo { path: final_path.to_string_lossy().into(), size_bytes: size, meta })
            }
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                Err(e)
            }
        }
    }
}

/// Ouvre la sauvegarde en lecture seule, vérifie phrase, intégrité et version. Ne modifie rien.
pub fn inspect_backup(path: &Path, passphrase: &str) -> Result<BackupMeta> {
    if !path.is_file() {
        return Err(CoreError::NotFound("fichier de sauvegarde".into()));
    }
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    store::apply_passphrase(&conn, passphrase)?;
    store::check_readable(&conn)?;
    integrity(&conn)?;
    let json: String = conn.query_row("SELECT json FROM backup_meta", [], |r| r.get(0)).map_err(|_| CoreError::Refused("ce fichier n'est pas une sauvegarde CMME".into()))?;
    let meta: BackupMeta = serde_json::from_str(&json)?;
    if meta.format != "CMME-BACKUP-1" {
        return Err(CoreError::Refused("format de sauvegarde inconnu".into()));
    }
    if meta.schema_version > SCHEMA_VERSION {
        return Err(CoreError::Refused("sauvegarde créée par une version plus récente de l'application".into()));
    }
    let counts = counts_of(&conn, "main")?;
    if counts != meta.counts {
        return Err(CoreError::Refused("sauvegarde incohérente (effectifs)".into()));
    }
    Ok(meta)
}

/// Prépare la restauration dans un fichier temporaire chiffré avec la clé de ce poste.
/// L'état courant n'est pas touché : l'appelant ferme la base puis appelle `commit_restore`.
pub fn prepare_restore(backup: &Path, passphrase: &str, target_db: &Path, target_key: &[u8; 32]) -> Result<(PathBuf, BackupMeta)> {
    let meta = inspect_backup(backup, passphrase)?;
    let dir = target_db.parent().ok_or_else(|| CoreError::Io("dossier de la base introuvable".into()))?;
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join(format!(".restauration-{}.tmp", store::new_id()));
    let r = (|| -> Result<()> {
        // Lecture-écriture requise pour attacher la destination ; aucune écriture dans la sauvegarde.
        let src = Connection::open_with_flags(backup, OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)?;
        store::apply_passphrase(&src, passphrase)?;
        src.execute("ATTACH DATABASE ?1 AS dst KEY ?2", params![tmp.to_string_lossy(), raw_key_literal(target_key)])?;
        src.query_row("SELECT sqlcipher_export('dst')", [], |_| Ok(()))?;
        src.execute_batch(&format!("DROP TABLE dst.backup_meta; PRAGMA dst.user_version = {};", meta.schema_version))?;
        src.execute_batch("DETACH DATABASE dst;")?;
        drop(src);
        // Ouverture de contrôle avec la clé du poste, migration éventuelle, intégrité, effectifs.
        let mut c = Connection::open(&tmp)?;
        store::apply_key(&c, target_key)?;
        store::check_readable(&c)?;
        store::migrate(&mut c)?;
        integrity(&c)?;
        if counts_of(&c, "main")? != meta.counts {
            return Err(CoreError::Refused("restauration incohérente (effectifs)".into()));
        }
        drop(c);
        File::open(&tmp)?.sync_all()?;
        Ok(())
    })();
    if let Err(e) = r {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok((tmp, meta))
}

/// Copie de sécurité de l'état courant (même clé), avant remplacement.
pub fn safety_copy(current: &Store, dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let p = dir.join(format!("avant-restauration-{stamp}.db"));
    std::fs::copy(&current.path, &p)?;
    File::open(&p)?.sync_all()?;
    Ok(p)
}

/// Remplacement atomique (renommage) une fois la base courante fermée.
pub fn commit_restore(prepared: &Path, target_db: &Path) -> Result<()> {
    std::fs::rename(prepared, target_db)?;
    if let Some(d) = target_db.parent() {
        fsync_dir(d);
    }
    Ok(())
}
