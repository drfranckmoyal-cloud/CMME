//! Active ou désactive la demande du mot de passe à l'ouverture de l'espace clinique (à la demande du praticien).
//! Usage : cargo run -p cmme-core --example mot_de_passe_ouverture -- oui|non

use cmme_core::keystore::{KeyStore, KeychainStore};
use cmme_core::Store;

fn main() {
    let required = std::env::args().nth(1).as_deref() == Some("oui");
    let db = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join("Library/Application Support/fr.cmme.recueil/clinique/cmme.db");
    let key = KeychainStore { service: "fr.cmme.recueil".into() }.get("db-clinique").unwrap().expect("clé absente");
    let s = Store::open(&db, &key, false, "réglage à la demande du praticien").unwrap();
    s.set_password_required(required).unwrap();
    println!("Mot de passe à l'ouverture : {}", if s.password_required().unwrap() { "demandé" } else { "non demandé" });
}
