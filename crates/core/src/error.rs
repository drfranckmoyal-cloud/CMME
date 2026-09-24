use serde::Serialize;

/// Erreurs remontées à l'interface. Les messages sont en français et ne contiennent
/// jamais de donnée clinique : seulement des codes de champ ou des explications génériques.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Valeur refusée pour « {field} » : {reason}")]
    Validation { field: String, reason: String },
    #[error("Cette consultation a été modifiée ailleurs (version {actual}, attendue {expected}). Rechargez-la avant de continuer.")]
    Conflict { expected: i64, actual: i64 },
    #[error("Consultation validée : utilisez « Corriger » pour créer un amendement motivé.")]
    Locked,
    #[error("Introuvable : {0}")]
    NotFound(String),
    #[error("Opération refusée : {0}")]
    Refused(String),
    #[error("Erreur de stockage : {0}")]
    Storage(String),
    #[error("Erreur de fichier : {0}")]
    Io(String),
    #[error("Trousseau macOS : {0}")]
    Keychain(String),
    #[error("Mot de passe incorrect ou fichier illisible.")]
    BadSecret,
}

impl From<rusqlite::Error> for CoreError {
    fn from(e: rusqlite::Error) -> Self {
        CoreError::Storage(e.to_string())
    }
}
impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Io(e.to_string())
    }
}
impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        CoreError::Storage(format!("JSON : {e}"))
    }
}

impl CoreError {
    pub fn validation(field: &str, reason: impl Into<String>) -> Self {
        CoreError::Validation { field: field.to_string(), reason: reason.into() }
    }
    pub fn kind(&self) -> &'static str {
        match self {
            CoreError::Validation { .. } => "validation",
            CoreError::Conflict { .. } => "conflict",
            CoreError::Locked => "locked",
            CoreError::NotFound(_) => "not_found",
            CoreError::Refused(_) => "refused",
            CoreError::Storage(_) => "storage",
            CoreError::Io(_) => "io",
            CoreError::Keychain(_) => "keychain",
            CoreError::BadSecret => "bad_secret",
        }
    }
}

#[derive(Serialize)]
pub struct ErrorPayload {
    pub kind: &'static str,
    pub message: String,
}

impl From<&CoreError> for ErrorPayload {
    fn from(e: &CoreError) -> Self {
        ErrorPayload { kind: e.kind(), message: e.to_string() }
    }
}

pub type Result<T> = std::result::Result<T, CoreError>;
