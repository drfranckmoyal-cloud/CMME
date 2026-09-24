//! Intégration de fiches Pages fictives : complément d'une ligne du tableau, conflit BEWE, nouveau dossier.

mod common;
use cmme_core::fiches::Fiche;
use cmme_core::import::{self, SheetConfig};

fn fiche(file: &str, name: &str, date: &str, bewe: &str) -> Fiche {
    Fiche {
        file: file.into(),
        raw: format!("Évaluation dentaire CMME\nNOM Prénom : {name}\nScore BEWE = {bewe}\n"),
        date: Some(date.into()),
        name: Some(name.into()),
        age: Some(30),
        sex: Some("female".into()),
        service_raw: Some("Centre expert".into()),
        last_visit_months: Some(24),
        occupation: Some("étudiante".into()),
        alcohol_raw: Some("RAS".into()),
        tca: Some("Boulimie sans vomissements".into()),
        bewe_raw: Some(bewe.into()),
        cao_c: Some(0),
        cao_a: None,
        cao_o: Some(1),
        quasi_vide: false,
        name_from_filename: false,
    }
}

#[test]
fn fiches_completent_sans_dupliquer() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let csv = d.path().join("t.csv");
    std::fs::write(&csv, "NOM prénom;Age;BEWE\nFICTIF Alpha;30;6–11 (7)\nFICTIF Beta;41;3\n").unwrap();
    let pv = import::preview(&s, &csv).unwrap();
    let cfg: Vec<SheetConfig> = pv.sheets.iter().map(|x| SheetConfig { name: x.name.clone(), kind: "individual".into(), header_row: x.header_row, mapping: x.suggested_mapping.clone(), service_from_sheet: false }).collect();
    let sum = s.import_stage(&csv, cfg).unwrap();
    s.import_commit(&sum.document_id, None).unwrap();
    assert_eq!(s.all_rows(true).unwrap().len(), 2);

    let rep = s.integrate_fiches(false, vec![
        fiche("a.pages", "FICTIF Alpha", "2025-01-02", "7"),
        fiche("a2.pages", "FICTIF Alpha", "2025-10-06", "9"),
        fiche("b.pages", "FICTIF Beta", "2025-02-03", "5"),
        fiche("c.pages", "FICTIF Gamma", "2025-03-04", ""),
        Fiche { file: "vide.pages".into(), ..Default::default() },
    ]).unwrap();
    assert_eq!(rep.enriched.len(), 1, "Alpha : même consultation complétée");
    assert_eq!(rep.followups.len(), 1, "Alpha : seconde fiche à une autre date = nouvelle consultation");
    assert_eq!(rep.conflicts.len(), 1, "Beta : BEWE divergent, rien modifié");
    assert_eq!(rep.new_patients, 1, "Gamma : nouveau dossier");
    assert_eq!(rep.skipped.len(), 1);
    let rows = s.all_rows(true).unwrap();
    assert_eq!(rows.len(), 4);
    let alpha: Vec<_> = rows.iter().filter(|r| r.display_name.as_deref() == Some("FICTIF Alpha")).collect();
    assert_eq!(alpha.len(), 2);
    assert_eq!(alpha[0].patient_id, alpha[1].patient_id);
    let first = alpha.iter().find(|r| r.visit_date.as_deref() == Some("2025-01-02")).unwrap();
    assert_eq!(first.bewe.value, Some(7), "valeur du tableau conservée");
    let e = s.load_encounter(&first.encounter_id, false).unwrap();
    assert_eq!(e.meta.revision, 2, "complément = nouvelle révision tracée");
    assert_eq!(e.values.iter().find(|v| v.field == "age_years").unwrap().value_num, Some(30.0));
    assert!(e.values.iter().any(|v| v.field == "fiche_pages_text"));
    assert_eq!(e.values.iter().find(|v| v.field == "ed_diagnosis").unwrap().value_text.as_deref(), Some("BN"));
    let beta = rows.iter().find(|r| r.display_name.as_deref() == Some("FICTIF Beta")).unwrap();
    assert!(beta.visit_date.is_none(), "conflit : aucune modification");
    // Relancer le même lot : refusé.
    assert!(s.integrate_fiches(false, vec![fiche("a.pages", "FICTIF Alpha", "2025-01-02", "7"), fiche("a2.pages", "FICTIF Alpha", "2025-10-06", "9"), fiche("b.pages", "FICTIF Beta", "2025-02-03", "5"), fiche("c.pages", "FICTIF Gamma", "2025-03-04", ""), Fiche { file: "vide.pages".into(), ..Default::default() }]).is_err());
}

#[test]
fn bewe_divergent_valeur_du_tableau_conservee() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let csv = d.path().join("t.csv");
    std::fs::write(&csv, "NOM prénom;BEWE\nFICTIF Beta;3\n").unwrap();
    let pv = import::preview(&s, &csv).unwrap();
    let cfg: Vec<SheetConfig> = pv.sheets.iter().map(|x| SheetConfig { name: x.name.clone(), kind: "individual".into(), header_row: x.header_row, mapping: x.suggested_mapping.clone(), service_from_sheet: false }).collect();
    let sum = s.import_stage(&csv, cfg).unwrap();
    s.import_commit(&sum.document_id, None).unwrap();
    let rep = s.integrate_fiches(true, vec![fiche("b.pages", "FICTIF Beta", "2025-02-03", "5")]).unwrap();
    assert_eq!((rep.enriched.len(), rep.conflicts.len()), (1, 0));
    let r = &s.all_rows(false).unwrap()[0];
    assert_eq!(r.bewe.value, Some(3), "valeur du tableau conservée");
    assert_eq!(r.visit_date.as_deref(), Some("2025-02-03"), "le reste de la fiche est repris");
    let e = s.load_encounter(&r.encounter_id, false).unwrap();
    assert!(e.flags.iter().any(|f| f.flag == "bewe_fiche_divergent" && f.status == "resolved"));
}
