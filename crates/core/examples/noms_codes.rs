//! Affiche code et nom de quelques dossiers (vérification ponctuelle demandée par le praticien).
use cmme_core::keystore::{KeyStore, KeychainStore};
use cmme_core::Store;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let key = KeychainStore { service: "fr.cmme.recueil".into() }.get("db-clinique").unwrap().unwrap();
    let s = Store::open(std::path::Path::new(&a[1]), &key, false, "lecture").unwrap();
    for r in s.all_rows(true).unwrap() {
        if a[2..].contains(&r.patient_code) {
            println!("{} | {} | {:?} | {:?}", r.patient_code, r.display_name.unwrap_or_default(), r.visit_date, r.service_code);
        }
    }
}
