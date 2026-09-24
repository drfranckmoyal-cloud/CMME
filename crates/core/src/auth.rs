//! Mot de passe de l'application : contrôle d'accès local, distinct de la clé du trousseau.
//! Empreinte Argon2id stockée dans la base chiffrée.

use crate::error::{CoreError, Result};
use crate::store::Store;
use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

pub const MIN_LEN: usize = 8;

impl Store {
    pub fn has_app_password(&self) -> Result<bool> {
        Ok(self.setting("app_password")?.is_some())
    }

    pub fn set_app_password(&self, current: Option<&str>, new_pw: &str) -> Result<()> {
        if self.has_app_password()? {
            let ok = current.map(|c| self.verify_app_password(c)).transpose()?.unwrap_or(false);
            if !ok {
                return Err(CoreError::BadSecret);
            }
        }
        if new_pw.chars().count() < MIN_LEN {
            return Err(CoreError::validation("mot de passe", format!("{MIN_LEN} caractères au moins")));
        }
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default().hash_password(new_pw.as_bytes(), &salt).map_err(|e| CoreError::Storage(e.to_string()))?.to_string();
        self.set_setting("app_password", &hash)?;
        self.audit("set_password", "app", None, None, None, None, None)?;
        Ok(())
    }

    pub fn verify_app_password(&self, pw: &str) -> Result<bool> {
        let Some(h) = self.setting("app_password")? else { return Ok(false) };
        let parsed = PasswordHash::new(&h).map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(Argon2::default().verify_password(pw.as_bytes(), &parsed).is_ok())
    }
}
