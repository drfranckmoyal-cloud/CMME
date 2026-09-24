//! Supprime les consultations dont le service est « autre » ou non renseigné (demande de Franck du 24/09/2026).
//! Sans --confirmer : liste seulement (codes). N'affiche aucun nom.
use cmme_core::keystore::{KeyStore, KeychainStore};
use cmme_core::Store;
fn main() {
    let go = std::env::args().any(|a| a == "--confirmer");
    let db = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join("Library/Application Support/fr.cmme.recueil/clinique/cmme.db");
    let key = KeychainStore { service: "fr.cmme.recueil".into() }.get("db-clinique").unwrap().unwrap();
    let mut s = Store::open(&db, &key, false, "Suppression demandée par le praticien").unwrap();
    let rows: Vec<_> = s.all_rows(false).unwrap().into_iter().filter(|r| r.service_code.as_deref().map(|c| c == "autre").unwrap_or(true)).collect();
    for r in &rows {
        println!("{} · service {} · {} · {}", r.patient_code, r.service_code.clone().unwrap_or("non renseigné".into()), r.collection_mode, r.visit_date.clone().unwrap_or("sans date".into()));
        if go {
            s.delete_encounter(&r.encounter_id, "Service « autre » ou non renseigné : suppression demandée par le praticien (24/09/2026)").unwrap();
        }
    }
    let st = s.stats(&Default::default()).unwrap();
    println!("{} consultation(s) {} · base : {} consultations, {} dossiers", rows.len(), if go { "supprimée(s)" } else { "concernée(s)" }, st.n_visits, st.n_patients);
}
