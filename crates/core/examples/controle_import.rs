//! Import à blanc d'un classeur dans une base chiffrée temporaire, aussitôt supprimée.
//! N'affiche que des effectifs et des types d'anomalies : aucun nom, aucune valeur de cellule.
//! Usage : cargo run -p cmme-core --example controle_import -- <fichier.xlsx>

use cmme_core::import::{preview, SheetConfig};
use cmme_core::keystore::generate_key;
use cmme_core::Store;
use std::collections::BTreeMap;

fn main() {
    let path = std::path::PathBuf::from(std::env::args().nth(1).expect("chemin du classeur"));
    let dir = tempfile::tempdir().unwrap();
    let key = generate_key().unwrap();
    let mut s = Store::open(&dir.path().join("essai.db"), &key, true, "controle").unwrap();
    let pv = preview(&s, &path).unwrap();
    let mut cfg = vec![];
    for sh in &pv.sheets {
        let kind = if sh.suggested_kind == "individual" && sh.headers.iter().any(|h| h.to_lowercase().contains("nom")) { "individual" } else { "summary" };
        println!("Feuille « {} » : {} lignes, proposée « {} », retenue « {} »", sh.name, sh.n_rows, sh.suggested_kind, kind);
        cfg.push(SheetConfig { name: sh.name.clone(), kind: kind.into(), header_row: sh.header_row, mapping: sh.suggested_mapping.clone(), service_from_sheet: true });
    }
    let sum = s.import_stage(&path, cfg).unwrap();
    println!("\nLignes lues : {}  · en attente : {}  · sans anomalie : {}  · à revoir : {}", sum.rows_read, sum.pending, sum.ready, sum.needs_review);
    println!("Exclues d'office : {:?}", sum.excluded);
    let recs = s.import_records(&sum.document_id, None).unwrap();
    let mut flags: BTreeMap<String, usize> = BTreeMap::new();
    for r in recs.iter().filter(|r| r.status == "pending") {
        for f in &r.unresolved {
            *flags.entry(f.clone()).or_default() += 1;
        }
    }
    println!("Anomalies à trancher (par type) : {flags:?}");
    let rep = s.import_commit(&sum.document_id, None).unwrap();
    let st = s.stats(&Default::default()).unwrap();
    println!("Validation en lot à blanc : {} consultations ; {} lignes laissées en revue", rep.encounters_created, rep.left_pending);
    println!("BEWE des lignes validées en lot : N={} médiane={:?} ≥9={}/{}", st.bewe.n_analysable, st.bewe.distribution.median, st.bewe.ge9.k, st.bewe.ge9.n);
    drop(s);
    drop(dir); // base temporaire supprimée
}
