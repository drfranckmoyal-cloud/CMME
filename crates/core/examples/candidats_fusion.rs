//! Liste les dossiers portant exactement le même nom (codes, dates, services, BEWE ; sans nom).
use cmme_core::keystore::{KeyStore, KeychainStore};
use cmme_core::Store;
fn main() {
    let db = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join("Library/Application Support/fr.cmme.recueil/clinique/cmme.db");
    let key = KeychainStore { service: "fr.cmme.recueil".into() }.get("db-clinique").unwrap().unwrap();
    let s = Store::open(&db, &key, false, "lecture").unwrap();
    let rows = s.all_rows(false).unwrap();
    for g in s.merge_candidates().unwrap() {
        println!("Groupe :");
        for (pid, code) in g {
            for r in rows.iter().filter(|r| r.patient_id == pid) {
                println!("  {code} · {} · {:?} · BEWE {:?} · âge {:?}", r.visit_date.clone().unwrap_or("sans date".into()), r.service_code, r.bewe.value, r.age_years);
            }
        }
    }
}
