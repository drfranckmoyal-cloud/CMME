//! Intègre les fiches Pages (JSON produit par research/scripts/lire_fiches_pages.py) dans l'espace clinique.
//! N'affiche que des effectifs et des codes de dossier, jamais de nom ni de texte clinique.
//! Essai à blanc sur une copie : --essai <copie.db>

use cmme_core::fiches::Fiche;
use cmme_core::keystore::{KeyStore, KeychainStore};
use cmme_core::Store;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let json = std::fs::read_to_string(&args[1]).expect("fichier JSON");
    let mut fiches: Vec<Fiche> = serde_json::from_str(&json).expect("JSON de fiches");
    // --seulement <liste.txt> : ne traiter que ces fichiers ; --garder-bewe-tableau : décision du praticien.
    if let Some(i) = args.iter().position(|a| a == "--seulement") {
        let keep: Vec<String> = std::fs::read_to_string(&args[i + 1]).unwrap().lines().map(String::from).collect();
        fiches.retain(|f| keep.contains(&f.file));
    }
    let keep_bewe = args.iter().any(|a| a == "--garder-bewe-tableau");
    let key = KeychainStore { service: "fr.cmme.recueil".into() }.get("db-clinique").unwrap().expect("clé absente");
    let db = match args.iter().position(|a| a == "--essai") {
        Some(i) => std::path::PathBuf::from(&args[i + 1]),
        None => std::path::PathBuf::from(std::env::var("HOME").unwrap()).join("Library/Application Support/fr.cmme.recueil/clinique/cmme.db"),
    };
    let mut s = Store::open(&db, &key, false, "import").unwrap();
    // --conflits-bewe : reprendre seulement les fiches écartées pour BEWE divergent lors d'un passage précédent.
    if args.iter().any(|a| a == "--conflits-bewe" || a == "--conflits-nom") {
        let reason = if args.iter().any(|a| a == "--conflits-nom") { "plusieurs dossiers portent ce nom" } else { "BEWE divergent" };
        let mut files = vec![];
        for d in s.import_documents().unwrap().iter().filter(|d| d.format == "pages-txt") {
            for r in s.import_records(&d.id, Some("excluded")).unwrap() {
                if r.exclusion_reason.as_deref() == Some(reason) {
                    files.push(r.cells[0].1.clone());
                }
            }
        }
        fiches.retain(|f| files.contains(&f.file));
        println!("Fiches à reprendre ({reason}) : {}", fiches.len());
    }
    s.author = format!("{} (fiches Pages)", s.setting("practitioner_name").unwrap().unwrap_or_default());
    let before = s.all_rows(false).unwrap();
    let r = s.integrate_fiches(keep_bewe, fiches).unwrap();
    let after = s.all_rows(false).unwrap();
    println!("Fiches : {}", r.fiches);
    println!("  consultations du tableau complétées (même consultation) : {}", r.enriched.len());
    println!("  nouvelles consultations de dossiers existants (autre date) : {}", r.followups.len());
    println!("  nouveaux dossiers : {}", r.new_patients);
    println!("  rattachées malgré une orthographe voisine (même service) : {}", r.near_matches.len());
    for (f, code) in &r.near_matches {
        println!("     - {code}  [{}]", f.rsplit('/').nth(1).unwrap_or(""));
    }
    println!("  conflits laissés à Franck : {}", r.conflicts.len());
    for (f, code, why) in &r.conflicts {
        println!("     - {code} : {why}  [{}]", f.rsplit('/').nth(1).unwrap_or(""));
    }
    println!("  fiches écartées : {}", r.skipped.len());
    for (f, why) in &r.skipped {
        println!("     - {why}  [{}]", f.rsplit('/').nth(1).unwrap_or(""));
    }
    let dated = after.iter().filter(|x| x.visit_date.is_some()).count();
    println!("Base : {} → {} consultations ; {} dossiers ; {} consultations datées", before.len(), after.len(), after.iter().map(|x| &x.patient_id).collect::<std::collections::HashSet<_>>().len(), dated);
    println!("Candidats à fusion (même identité) : {}", s.merge_candidates().unwrap().len());
}
