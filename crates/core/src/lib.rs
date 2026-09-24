//! Cœur métier CMME. Aucune dépendance réseau : tout est local.

pub mod auth;
pub mod backup;
pub mod demo;
pub mod domain;
pub mod error;
pub mod import;
pub mod keystore;
pub mod research;
pub mod stats;
pub mod store;

pub use error::{CoreError, Result};
pub use store::Store;
