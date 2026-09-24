//! Crée le projet d'étude « ancien + nouveau, toutes périodes », fige un export et l'écrit sur disque.
//! Affiche des effectifs uniquement.
use cmme_core::keystore::{KeyStore, KeychainStore};
use cmme_core::research::ProjectInput;
use cmme_core::Store;
fn main() {
    let out = std::path::PathBuf::from(std::env::args().nth(1).expect("dossier de destination"));
    let db = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join("Library/Application Support/fr.cmme.recueil/clinique/cmme.db");
    let key = KeychainStore { service: "fr.cmme.recueil".into() }.get("db-clinique").unwrap().unwrap();
    let mut s = Store::open(&db, &key, false, "x").unwrap();
    s.author = s.setting("practitioner_name").unwrap().unwrap_or_default();
    let p = s.create_project(ProjectInput {
        name: "CMME — ancien + nouveau recueil, toutes périodes".into(),
        period_start: None, period_end: None, include_legacy: true, include_prospective: true,
        exclude_open_anomalies: true, plan_version: Some("PLAN-2026-09-v0 (à valider)".into()),
    }).unwrap();
    let sel = s.selection(&p.id).unwrap();
    println!("Dossiers dans la base : {} · consultations : {}", sel.patients_total, sel.encounters_total);
    for (k, v) in &sel.excluded_encounters { println!("  consultations exclues ({k}) : {v}"); }
    let mut ex: std::collections::BTreeMap<String, usize> = Default::default();
    for x in &sel.excluded_patients { *ex.entry(x.2.clone()).or_default() += 1; }
    for (k, v) in &ex { println!("  dossiers exclus ({k}) : {v}"); }
    println!("Dossiers inclus : {} · BEWE analysable à la visite index : {}", sel.included.len(), sel.bewe_analysable_index);
    let snap = s.freeze_export(&p.id, false, env!("CARGO_PKG_VERSION")).unwrap();
    std::fs::create_dir_all(&out).unwrap();
    let dir = s.write_snapshot(&snap.id, &out).unwrap();
    println!("Export figé écrit : {dir}");
}
