//! Reprise de l'historique : CSV synthétique fourni et classeur XLSX fictif multi-feuilles.

mod common;
use cmme_core::import::{self, SheetConfig};
use cmme_core::store::listing::Filter;
use cmme_core::{CoreError, Store};
use serde_json::json;
use std::path::{Path, PathBuf};

fn csv_path() -> PathBuf {
    common::specs_dir().join("data/import_synthetique.csv")
}

fn config_from_preview(s: &Store, p: &Path) -> Vec<SheetConfig> {
    let pv = import::preview(s, p).unwrap();
    pv.sheets.iter().map(|sh| SheetConfig { name: sh.name.clone(), kind: sh.suggested_kind.clone(), header_row: sh.header_row, mapping: sh.suggested_mapping.clone(), service_from_sheet: true }).collect()
}

fn rec<'a>(rows: &'a [import::RecordRow], code: &str) -> &'a import::RecordRow {
    rows.iter().find(|r| r.cells.iter().any(|c| c.1 == code)).unwrap()
}

#[test]
fn csv_synthetique_de_bout_en_bout() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let pv = import::preview(&s, &csv_path()).unwrap();
    assert_eq!(pv.sheets.len(), 1);
    assert_eq!(pv.sheets[0].suggested_mapping, vec!["patient_code", "age", "service", "vomiting", "bewe", "prevention"]);
    assert_eq!(pv.sheets[0].n_rows, 8);
    let cfg = config_from_preview(&s, &csv_path());
    let sum = s.import_stage(&csv_path(), cfg.clone()).unwrap();
    assert_eq!((sum.rows_read, sum.pending, sum.ready, sum.needs_review), (8, 8, 4, 4));
    let doc = sum.document_id.clone();

    let rows = s.import_records(&doc, None).unwrap();
    let flags = |c: &str| rec(&rows, c).flags.iter().map(|f| f.0.clone()).collect::<Vec<_>>();
    assert!(flags("SYN-004").contains(&"band_mismatch".to_string()));
    assert!(flags("SYN-004").contains(&"age_band_only".to_string()));
    assert!(flags("SYN-006").contains(&"band_only".to_string()));
    assert!(flags("SYN-007").contains(&"unparseable".to_string()));
    assert!(flags("SYN-008").contains(&"ambiguous_exact_value".to_string()));
    assert!(!flags("SYN-001").iter().any(|f| f == "not_recorded"), "0 est un score, pas un manque");

    // Validation en lot : seules les lignes sans anomalie.
    let r = s.import_commit(&doc, None).unwrap();
    assert_eq!((r.encounters_created, r.left_pending), (4, 4));

    let all = s.all_rows(false).unwrap();
    let row = |code: &str| all.iter().find(|x| x.patient_code == code).unwrap().clone();
    assert_eq!(row("SYN-001").bewe.value, Some(0));
    assert_eq!(row("SYN-001").bewe.origin, "historical");
    assert_eq!(row("SYN-003").bewe.value, Some(11));
    assert_eq!(row("SYN-001").collection_mode, "legacy_retrospective");
    assert_eq!(row("SYN-001").status, "validated");
    let e3 = s.load_encounter(&row("SYN-003").encounter_id, false).unwrap();
    assert!(e3.prevention.is_empty(), "ancien « Avancé » : aucune composante reconstruite");
    assert!(e3.bewe.is_empty(), "aucun sextant reconstruit");
    assert_eq!(e3.values.iter().find(|v| v.field == "prevention_legacy").unwrap().value_text.as_deref(), Some("advanced"));
    assert_eq!(e3.values.iter().find(|v| v.field == "prevention_protocol_legacy_raw").unwrap().value_text.as_deref(), Some("Avancé"));
    assert!(e3.values.iter().all(|v| v.field != "prevention_protocol"));
    let e2 = s.load_encounter(&row("SYN-002").encounter_id, false).unwrap();
    assert_eq!(e2.values.iter().find(|v| v.field == "vomiting_legacy_code").unwrap().value_text.as_deref(), Some("no"));
    assert!(e2.values.iter().all(|v| v.field != "vomiting_lifetime"), "« Non » ne devient pas « jamais rapportés »");
    assert!(e2.values.iter().all(|v| v.source_type.as_deref() == Some("legacy_import")));

    // Revue individuelle.
    let rows = s.import_records(&doc, Some("pending")).unwrap();
    let r4 = rec(&rows, "SYN-004");
    assert_eq!(r4.unresolved, vec!["band_mismatch"]);
    assert!(s.import_commit(&doc, Some(vec![r4.id.clone()])).unwrap().encounters_created == 0, "non résolu : reste en attente");
    s.import_decide(&r4.id, "bewe_total_historical", "accepted", None, None).unwrap();
    let r6 = rec(&rows, "SYN-006");
    assert!(s.import_decide(&r6.id, "bewe_total_historical", "accepted", None, None).is_err(), "rien à accepter pour une tranche seule");
    assert!(s.import_decide(&r6.id, "bewe_total_historical", "corrected", Some(json!(8)), None).is_err(), "correction sans justification refusée");
    s.import_decide(&r6.id, "bewe_total_historical", "unavailable", None, None).unwrap();
    let r7 = rec(&rows, "SYN-007");
    s.import_decide(&r7.id, "bewe_total_historical", "unavailable", None, None).unwrap();
    let r8 = rec(&rows, "SYN-008");
    s.import_decide(&r8.id, "bewe_total_historical", "corrected", Some(json!(9)), Some("Vérifié dans la fiche papier (fictif)".into())).unwrap();
    let ids: Vec<String> = [r4, r6, r7, r8].iter().map(|r| r.id.clone()).collect();
    let r = s.import_commit(&doc, Some(ids)).unwrap();
    assert_eq!(r.encounters_created, 4);

    let all = s.all_rows(false).unwrap();
    let row = |code: &str| all.iter().find(|x| x.patient_code == code).unwrap().clone();
    assert_eq!(row("SYN-004").bewe.value, Some(7), "convention du nombre entre parenthèses");
    let e4 = s.load_encounter(&row("SYN-004").encounter_id, false).unwrap();
    assert_eq!(e4.meta.bewe_historical_band, Some((1, 5)), "tranche d'origine conservée");
    assert_eq!(e4.meta.bewe_legacy_raw.as_deref(), Some("1–5 (7)"));
    assert!(e4.flags.iter().any(|f| f.flag == "band_mismatch" && f.status == "accepted"), "exception visible");
    assert_eq!(e4.values.iter().find(|v| v.field == "age_band_min").unwrap().value_num, Some(20.0));
    assert!(e4.values.iter().all(|v| v.field != "age_years"), "aucun âge exact inventé");
    assert_eq!(row("SYN-006").bewe.value, None, "pas de point milieu");
    let e6 = s.load_encounter(&row("SYN-006").encounter_id, false).unwrap();
    assert_eq!(e6.meta.bewe_historical_band, Some((6, 11)));
    assert_eq!(row("SYN-008").bewe.value, Some(9));
    let e8 = s.load_encounter(&row("SYN-008").encounter_id, false).unwrap();
    assert_eq!(e8.flags.iter().find(|f| f.flag == "ambiguous_exact_value").unwrap().detail.as_deref(), Some("Vérifié dans la fiche papier (fictif)"));
    let e7 = s.load_encounter(&row("SYN-007").encounter_id, false).unwrap();
    assert_eq!(e7.values.iter().find(|v| v.field == "age_band_max").unwrap().value_num, Some(19.0));
    assert_eq!(s.list_encounters(&Filter { bewe: Some("missing".into()), ..Default::default() }, false).unwrap().len(), 2);

    // Réimport identique : aucune duplication.
    assert!(matches!(s.import_stage(&csv_path(), cfg.clone()), Err(CoreError::Refused(_))));
    assert_eq!(s.all_rows(false).unwrap().len(), 8);

    // Nouvelle version du fichier : lignes identiques reconnues, changement soumis à revue.
    let text = std::fs::read_to_string(csv_path()).unwrap().replace("SYN-002;31;SAS;Non;1–5 (3)", "SYN-002;31;SAS;Non;1–5 (4)");
    let v2 = d.path().join("import_synthetique_v2.csv");
    std::fs::write(&v2, text).unwrap();
    let sum2 = s.import_stage(&v2, cfg).unwrap();
    assert_eq!(sum2.unchanged, 7);
    assert_eq!(pv.sheets[0].name, "CSV");
    let rows2 = s.import_records(&sum2.document_id, Some("pending")).unwrap();
    assert_eq!(rows2.len(), 1);
    let changed = &rows2[0];
    assert!(changed.flags.iter().any(|f| f.0 == "changed_since_previous_version"));
    let ch = &changed.diff.as_ref().unwrap()["changes"][0];
    assert_eq!((ch["old"].as_str(), ch["new"].as_str()), (Some("1–5 (3)"), Some("1–5 (4)")));
    let r = s.import_commit(&sum2.document_id, None).unwrap();
    assert_eq!(r.encounters_created, 0, "pas de nouvelle cohorte par défaut");
    assert_eq!(s.all_rows(false).unwrap().len(), 8);
}

fn build_workbook(path: &Path) {
    use rust_xlsxwriter::Workbook;
    let mut wb = Workbook::new();
    let ce = wb.add_worksheet();
    ce.set_name("Centre Expert").unwrap();
    let rows: Vec<Vec<&str>> = vec![
        vec!["Nom", "Prénom", "Âge", "Vomissements", "BEWE", "Observations", "FDI"],
        vec!["Fictif", "Alpha", "24", "Oui", "6–11 (9)", "Obs fictive A", "16"],
        vec!["mars 2025"],
        vec!["Homonyme", "Marie", "30", "Non", "0", "Obs fictive B", ""],
        vec!["Homonyme", "Marie", "45", "Oui", "12–18 (15)", "Obs fictive C", ""],
        vec!["Nom", "Prénom", "Âge", "Vomissements", "BEWE", "Observations", "FDI"],
        vec!["", "", "29", "Oui", "3", "Ligne sans nom", ""],
    ];
    for (r, row) in rows.iter().enumerate() {
        for (c, v) in row.iter().enumerate() {
            ce.write_string(r as u32, c as u16, *v).unwrap();
        }
    }
    let sas = wb.add_worksheet();
    sas.set_name("SAS").unwrap();
    let rows: Vec<Vec<&str>> = vec![
        vec!["Nom", "Prénom", "Âge", "Vomissements", "BEWE", "Observations"],
        vec!["Fictif", "Beta", "22", "Non", "1–5 (2)", "Obs fictive D"],
    ];
    for (r, row) in rows.iter().enumerate() {
        for (c, v) in row.iter().enumerate() {
            sas.write_string(r as u32, c as u16, *v).unwrap();
        }
    }
    sas.write_number(1, 6, 0.0).unwrap();
    let syn = wb.add_worksheet();
    syn.set_name("Synthèse").unwrap();
    syn.write_string(0, 0, "Total").unwrap();
    syn.write_string(0, 1, "Moyenne").unwrap();
    syn.write_number(1, 0, 245.0).unwrap();
    syn.write_number(1, 1, 4.6).unwrap();
    wb.save(path).unwrap();
}

#[test]
fn classeur_multi_feuilles_synthese_homonymes_fusion_annulable() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let x = d.path().join("classeur_fictif.xlsx");
    build_workbook(&x);
    let pv = import::preview(&s, &x).unwrap();
    let kinds: Vec<(&str, &str)> = pv.sheets.iter().map(|sh| (sh.name.as_str(), sh.suggested_kind.as_str())).collect();
    assert_eq!(kinds, vec![("Centre Expert", "individual"), ("SAS", "individual"), ("Synthèse", "summary")]);
    assert_eq!(pv.sheets[0].suggested_mapping[6], "ignore", "ancienne rubrique FDI non reprise");
    let cfg = config_from_preview(&s, &x);
    let sum = s.import_stage(&x, cfg).unwrap();
    assert_eq!(sum.summary_sheets, vec!["Synthèse"]);
    assert_eq!(sum.excluded.get("separator_row"), Some(&1));
    assert_eq!(sum.excluded.get("repeated_header"), Some(&1));
    assert_eq!(sum.pending, 5);
    let rows = s.import_records(&sum.document_id, Some("pending")).unwrap();
    let homs: Vec<_> = rows.iter().filter(|r| r.cells.iter().any(|c| c.1 == "Homonyme")).collect();
    assert_eq!(homs.len(), 2);
    assert!(homs.iter().all(|r| r.unresolved.contains(&"duplicate_candidate".to_string())), "homonymes : revue, pas de fusion");
    let noname = rows.iter().find(|r| r.cells.iter().any(|c| c.1 == "Ligne sans nom")).unwrap();
    assert!(noname.unresolved.contains(&"no_identity".to_string()));
    // La feuille de synthèse ne crée aucun patient ; le lot ne valide que les lignes sans anomalie.
    let r = s.import_commit(&sum.document_id, None).unwrap();
    assert_eq!(r.encounters_created, 2);
    assert_eq!(s.all_rows(false).unwrap().len(), 2);
    // Les homonymes sont des personnes différentes.
    for h in &homs {
        s.import_link(&h.id, "different_person", None).unwrap();
    }
    s.import_exclude(&noname.id, "Copie probable d'un bloc (fictif)").unwrap();
    let r = s.import_commit(&sum.document_id, Some(homs.iter().map(|h| h.id.clone()).collect())).unwrap();
    assert_eq!((r.encounters_created, r.patients_created), (2, 2));
    let all = s.all_rows(true).unwrap();
    assert_eq!(all.len(), 4);
    let hom_rows: Vec<_> = all.iter().filter(|r| r.display_name.as_deref() == Some("Homonyme Marie")).collect();
    assert_eq!(hom_rows.len(), 2);
    assert_ne!(hom_rows[0].patient_id, hom_rows[1].patient_id, "aucune fusion automatique");
    // Candidats signalés, fusion manuelle erronée puis annulée.
    let cands = s.merge_candidates().unwrap();
    assert_eq!(cands.len(), 1);
    let m = s.merge_patients(&hom_rows[0].patient_id, &hom_rows[1].patient_id, "Test de fusion erronée").unwrap();
    assert_eq!(s.all_rows(true).unwrap().iter().filter(|r| r.patient_id == hom_rows[1].patient_id).count(), 2);
    s.undo_merge(&m).unwrap();
    let after = s.all_rows(true).unwrap();
    assert_eq!(after.iter().filter(|r| r.patient_id == hom_rows[0].patient_id).count(), 1);
    assert_eq!(after.iter().filter(|r| r.patient_id == hom_rows[1].patient_id).count(), 1);
    assert!(s.undo_merge(&m).is_err());
    // La source brute est conservée chiffrée : la ligne exclue reste consultable avec son motif.
    let ex = s.import_records(&sum.document_id, Some("excluded")).unwrap();
    assert_eq!(ex.len(), 3);
}

#[test]
fn formats_refuses_clairement() {
    let d = common::tmpdir();
    let (s, _) = common::store_in(&d);
    let p = d.path().join("tableau.numbers");
    std::fs::write(&p, b"x").unwrap();
    assert!(matches!(import::preview(&s, &p), Err(CoreError::Refused(m)) if m.contains("XLSX")));
    let p = d.path().join("latin1.csv");
    std::fs::write(&p, b"nom;age\n\xe9l\xe8ve;20\n").unwrap();
    assert!(matches!(import::preview(&s, &p), Err(CoreError::Refused(m)) if m.contains("UTF-8")));
}
