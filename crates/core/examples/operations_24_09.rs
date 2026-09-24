//! Opérations demandées par Franck le 24/09/2026 : fusion H-0245 → H-0029, suppression des brouillons.
//! Affiche des codes seulement.
use cmme_core::keystore::{KeyStore, KeychainStore};
use cmme_core::Store;
fn main() {
    let db = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join("Library/Application Support/fr.cmme.recueil/clinique/cmme.db");
    let key = KeychainStore { service: "fr.cmme.recueil".into() }.get("db-clinique").unwrap().unwrap();
    let mut s = Store::open(&db, &key, false, "x").unwrap();
    s.author = s.setting("practitioner_name").unwrap().unwrap_or_default();
    let rows = s.all_rows(false).unwrap();
    let pid = |code: &str| rows.iter().find(|r| r.patient_code == code).map(|r| r.patient_id.clone());
    match (pid("H-0245"), pid("H-0029")) {
        (Some(a), Some(b)) => {
            s.merge_patients(&a, &b, "Même personne (confirmé par le praticien le 24/09/2026)").unwrap();
            println!("Fusion H-0245 → H-0029 faite (annulable dans Reprise › Doublons et fusions)");
        }
        _ => println!("Fusion : dossiers introuvables, rien fait"),
    }
    for r in rows.iter().filter(|r| r.status == "draft") {
        s.delete_encounter(&r.encounter_id, "Brouillon abandonné à la demande du praticien (24/09/2026)").unwrap();
        println!("Brouillon supprimé : {}", r.patient_code);
    }
}
