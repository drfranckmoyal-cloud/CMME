//! Effectifs par service et version du schéma (aucune donnée nominative).
use cmme_core::keystore::{KeyStore, KeychainStore};
use cmme_core::Store;
fn main() {
    let db = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join("Library/Application Support/fr.cmme.recueil/clinique/cmme.db");
    let key = KeychainStore { service: "fr.cmme.recueil".into() }.get("db-clinique").unwrap().unwrap();
    let s = Store::open(&db, &key, false, "lecture").unwrap();
    let st = s.stats(&Default::default()).unwrap();
    println!("schéma v{} · {} consultations · {} dossiers", s.schema_version().unwrap(), st.n_visits, st.n_patients);
    for (k, v) in st.services { println!("  {k} : {v}"); }
}
