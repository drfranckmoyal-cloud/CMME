//! Corrige une date de consultation par amendement motivé. Usage : -- <ancienne AAAA-MM-JJ> <nouvelle AAAA-MM-JJ> "<motif>"
use cmme_core::domain::values::FieldInput;
use cmme_core::keystore::{KeyStore, KeychainStore};
use cmme_core::Store;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let db = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join("Library/Application Support/fr.cmme.recueil/clinique/cmme.db");
    let key = KeychainStore { service: "fr.cmme.recueil".into() }.get("db-clinique").unwrap().unwrap();
    let mut s = Store::open(&db, &key, false, "x").unwrap();
    s.author = s.setting("practitioner_name").unwrap().unwrap_or_default();
    let rows: Vec<_> = s.all_rows(false).unwrap().into_iter().filter(|r| r.visit_date.as_deref() == Some(a[1].as_str())).collect();
    for r in rows {
        let e = s.load_encounter(&r.encounter_id, false).unwrap();
        let v = if e.meta.status == "validated" { s.start_amendment(&r.encounter_id, e.meta.version, &a[3]).unwrap().meta.version } else { e.meta.version };
        let v = s.save_fields(&r.encounter_id, v, vec![FieldInput { field: "visit_date".into(), value: Some(serde_json::json!(a[2])), source_type: Some("clinician_adjudication".into()), ..Default::default() }]).unwrap().version;
        s.validate_encounter(&r.encounter_id, v).unwrap();
        println!("{} : {} → {}", r.patient_code, a[1], a[2]);
    }
}
