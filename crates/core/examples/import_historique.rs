//! Import assisté de l'ancien classeur dans l'espace clinique (accord de F. Moyal du 24/09/2026).
//! Règles conservatrices : aucune valeur inventée, décisions justifiées et tracées, exclusions réversibles,
//! aucune fusion de dossiers. N'affiche que des effectifs.
//!
//! Essai à blanc : cargo run -p cmme-core --example import_historique -- <fichier.xlsx> --essai
//! Réel         : cargo run -p cmme-core --example import_historique -- <fichier.xlsx>

use cmme_core::import::{preview, SheetConfig};
use cmme_core::keystore::{generate_key, KeyStore, KeychainStore};
use cmme_core::Store;
use std::collections::BTreeMap;

const PREFIXE: &str = "Décision conservatrice de l'import assisté (accord F. Moyal, 24/09/2026)";
const CLINIQUES: [&str; 9] = ["bewe", "vomiting", "diet", "hygiene", "dmft", "care", "prevention", "other_wear", "observations"];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = std::path::PathBuf::from(&args[1]);
    let essai = args.iter().any(|a| a == "--essai");
    let tmp = tempfile::tempdir().unwrap();
    let mut s = if essai {
        Store::open(&tmp.path().join("essai.db"), &generate_key().unwrap(), true, "essai").unwrap()
    } else {
        let home = std::env::var("HOME").unwrap();
        let db = std::path::PathBuf::from(home).join("Library/Application Support/fr.cmme.recueil/clinique/cmme.db");
        let key = KeychainStore { service: "fr.cmme.recueil".into() }.get("db-clinique").unwrap().expect("clé absente du trousseau : créer d'abord l'espace clinique");
        let mut s = Store::open(&db, &key, false, "import").unwrap();
        s.author = format!("{} (import assisté)", s.setting("practitioner_name").unwrap().unwrap_or_default());
        s
    };

    // 1. Feuilles : individuelles seulement si une colonne de nom existe ; le reste = synthèse.
    let pv = preview(&s, &path).unwrap();
    if let Some(a) = &pv.already_imported {
        println!("Déjà importé le {} : arrêt.", a.imported_at);
        return;
    }
    let cfg: Vec<SheetConfig> = pv
        .sheets
        .iter()
        .map(|sh| {
            let indiv = sh.suggested_kind == "individual" && sh.suggested_mapping.iter().any(|m| m == "full_name");
            SheetConfig { name: sh.name.clone(), kind: if indiv { "individual" } else { "summary" }.into(), header_row: sh.header_row, mapping: sh.suggested_mapping.clone(), service_from_sheet: true }
        })
        .collect();
    let mapping: BTreeMap<String, Vec<String>> = cfg.iter().map(|c| (c.name.clone(), c.mapping.clone())).collect();
    let sum = s.import_stage(&path, cfg).unwrap();
    println!("Préparé : {} lignes lues, {} en attente, {} séparateurs écartés.", sum.rows_read, sum.pending, sum.excluded.values().sum::<usize>());

    // 2. Revue automatique conservatrice.
    let mut bilan: BTreeMap<&str, usize> = BTreeMap::new();
    let recs = s.import_records(&sum.document_id, Some("pending")).unwrap();
    for r in &recs {
        let map = &mapping[&r.sheet];
        let clin_vide = r.cells.iter().enumerate().all(|(i, (_, v))| !CLINIQUES.contains(&map.get(i).map(String::as_str).unwrap_or("")) || v.trim().is_empty());
        let flags: Vec<&str> = r.flags.iter().map(|f| f.0.as_str()).collect();
        if flags.contains(&"no_identity") {
            s.import_exclude(&r.id, &format!("{PREFIXE} : ligne sans nom, copie probable d'un bloc (rapport du 23/09) ; à confirmer")).unwrap();
            *bilan.entry("exclue : sans nom").or_default() += 1;
            continue;
        }
        if clin_vide {
            s.import_exclude(&r.id, &format!("{PREFIXE} : aucune donnée clinique sur la ligne")).unwrap();
            *bilan.entry("exclue : aucune donnée clinique").or_default() += 1;
            continue;
        }
        if flags.contains(&"duplicate_exact") {
            s.import_exclude(&r.id, &format!("{PREFIXE} : doublon exact d'une autre ligne (mêmes données cliniques), compté une fois")).unwrap();
            *bilan.entry("exclue : doublon exact").or_default() += 1;
            continue;
        }
        for u in &r.unresolved {
            match u.as_str() {
                "band_mismatch" => {
                    s.import_decide(&r.id, "bewe_total_historical", "accepted", None, None).unwrap();
                    *bilan.entry("BEWE : nombre entre parenthèses retenu (anomalie conservée)").or_default() += 1;
                }
                "duplicate_candidate" | "changed_since_previous_version" => {
                    s.import_link(&r.id, "new", None).unwrap();
                    *bilan.entry("identité répétée : dossier distinct, fusion laissée à Franck").or_default() += 1;
                }
                f => {
                    let target = match f {
                        "not_recorded" | "band_only" | "ambiguous_exact_value" | "out_of_range" | "unparseable" => "bewe_total_historical",
                        "date_suspect" | "date_unparseable" => "visit_date",
                        "age_band_mismatch" | "age_unparseable" => "age_years",
                        "vomiting_ambiguous" => "vomiting_legacy_code",
                        "sex_ambiguous" => "sex_recorded",
                        "hygiene_out_of_category" => "hygiene_legacy",
                        "care_out_of_category" => "care_legacy",
                        "other_wear_ambiguous" => "other_wear_legacy",
                        "dmft_to_review" => "dmft_total_historical",
                        _ => {
                            *bilan.entry("non traité").or_default() += 1;
                            continue;
                        }
                    };
                    s.import_decide(&r.id, target, "unavailable", None, Some(format!("{PREFIXE} : valeur source non interprétable, conservée dans la source brute"))).unwrap();
                    *bilan.entry("valeur marquée indisponible (source brute conservée)").or_default() += 1;
                }
            }
        }
    }
    for (k, v) in &bilan {
        println!("  {k} : {v}");
    }

    // 3. Validation : toutes les lignes résolues.
    let ids: Vec<String> = s.import_records(&sum.document_id, Some("pending")).unwrap().into_iter().filter(|r| r.unresolved.is_empty()).map(|r| r.id).collect();
    let rep = s.import_commit(&sum.document_id, Some(ids)).unwrap();
    println!("Validé : {} consultations historiques, {} dossiers ; {} ligne(s) encore en attente.", rep.encounters_created, rep.patients_created, rep.left_pending + s.import_records(&sum.document_id, Some("pending")).unwrap().len());

    let st = s.stats(&Default::default()).unwrap();
    let b = &st.bewe;
    println!("\nBase clinique : {} consultations, {} dossiers.", st.n_visits, st.n_patients);
    println!("BEWE analysable {}/{} · médiane {:?} [{:?}–{:?}] · >0 {}/{} · ≥9 {}/{} · ≥14 {}/{}", b.n_analysable, b.n_visits, b.distribution.median, b.distribution.q1, b.distribution.q3, b.gt0.k, b.gt0.n, b.ge9.k, b.ge9.n, b.ge14.k, b.ge14.n);
    println!("Classes : {:?}", b.categories);
    println!("Candidats à fusion (même identité) : {} groupe(s), à examiner dans Reprise › Doublons et fusions.", s.merge_candidates().unwrap().len());
}
