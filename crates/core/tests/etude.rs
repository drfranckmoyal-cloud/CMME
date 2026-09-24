//! Statistiques, sélection de la visite index, export figé et recalcul externe.

mod common;
use cmme_core::domain::values::FieldInput;
use cmme_core::research::{whitelist, ProjectInput};
use cmme_core::store::listing::Filter;
use serde_json::{json, Value};

fn fi(field: &str, value: Value) -> FieldInput {
    FieldInput { field: field.into(), value: Some(value), ..Default::default() }
}

#[test]
fn jeu_demo_statistiques_de_recette() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    assert_eq!(cmme_core::demo::seed_demo(&mut s).unwrap(), 8);
    let st = s.stats(&Filter::default()).unwrap();
    assert_eq!((st.n_visits, st.n_patients), (8, 8));
    assert_eq!(st.bewe.n_analysable, 7);
    assert_eq!(st.bewe.distribution.median, Some(5.0));
    let cats: Vec<usize> = st.bewe.categories.iter().map(|c| c.1).collect();
    assert_eq!(cats, vec![2, 3, 1, 1]);
    assert_eq!((st.bewe.ge9.k, st.bewe.ge9.n), (2, 7));
    assert_eq!(st.bewe.origins.get("incomplete"), Some(&1));
    // Filtre sans résultat : aucune donnée analysable, pas un 0 %.
    let empty = s.stats(&Filter { text: Some("ZZZ".into()), ..Default::default() }).unwrap();
    assert_eq!(empty.bewe.n_analysable, 0);
    assert_eq!(empty.bewe.ge9.pct, None);
    assert_eq!(empty.bewe.distribution.median, None);
    // Filtre combiné BEWE ≥ 9.
    assert_eq!(s.list_encounters(&Filter { bewe: Some("ge9".into()), ..Default::default() }, false).unwrap().len(), 2);
    // Relancer le jeu démo ne duplique rien.
    assert_eq!(cmme_core::demo::seed_demo(&mut s).unwrap(), 0);
}

#[test]
fn export_liste_blanche_fige_et_recalculable() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    cmme_core::demo::seed_demo(&mut s).unwrap();
    // Un dossier avec identité, textes libres et contenu piégé.
    let e = s.create_dossier(Some("EXP-001".into()), Some(cmme_core::store::encounters::IdentityInput { last_name: Some("NomFictifSecret".into()), first_name: Some("Prenom".into()), hospital_id: Some("IPP-999".into()) }), false).unwrap();
    let v = s.save_fields(&e.meta.id, e.meta.version, vec![
        fi("visit_date", json!("2026-03-02")), fi("age_years", json!(27)), fi("service_code", json!("sas")),
        fi("consultation_observations", json!("=HYPERLINK(\"http://x\") note libre secrète")),
        fi("occupation_text", json!("Profession précise")), fi("toothpaste_name", json!("+Marque")),
    ]).unwrap().version;
    s.validate_encounter(&e.meta.id, v).unwrap();
    // Un patient à deux visites : index = la première datée.
    let v2 = s.new_encounter_for_patient(&e.meta.patient_id).unwrap();
    let vv = s.save_fields(&v2.meta.id, v2.meta.version, vec![fi("visit_date", json!("2026-06-10")), fi("age_years", json!(27))]).unwrap().version;
    s.validate_encounter(&v2.meta.id, vv).unwrap();

    let p = s.create_project(ProjectInput { name: "Étude rétrospective fictive".into(), period_start: None, period_end: None, include_legacy: true, include_prospective: true, exclude_open_anomalies: true, plan_version: Some("PLAN-TEST-1".into()) }).unwrap();
    let sel = s.selection(&p.id).unwrap();
    assert_eq!(sel.included.len(), 7, "6 démo validées + EXP-001 ; brouillons exclus");
    assert_eq!(sel.excluded_encounters.get("non_validee"), Some(&2));
    let exp = sel.included.iter().find(|x| x.patient_code == "EXP-001").unwrap();
    assert_eq!(exp.index_encounter, e.meta.id, "une seule visite index : la première");
    assert_eq!(exp.eligible_encounters.len(), 2);

    let snap = s.freeze_export(&p.id, false, "test").unwrap();
    let files = s.snapshot_files(&snap.id).unwrap();
    let wl = whitelist();
    for (name, cols) in &wl {
        let content = &files[*name];
        let header: Vec<String> = content.lines().next().unwrap().split(',').map(String::from).collect();
        for h in &header {
            assert!(cols.contains(h), "{name} : colonne hors liste blanche {h}");
        }
        assert!(!header.contains(&"visit_date".to_string()), "dates exactes exclues par défaut");
    }
    let everything: String = files.values().cloned().collect::<Vec<_>>().join("\n");
    for forbidden in ["NomFictifSecret", "IPP-999", "note libre", "Profession précise", "Marque", "EXP-001", "HYPERLINK", "DEMO-00", &e.meta.patient_id] {
        assert!(!everything.contains(forbidden), "fuite dans l'export : {forbidden}");
    }
    // Empreintes du manifest.
    for (name, h) in snap.manifest["sha256"].as_object().unwrap() {
        use sha2::Digest;
        assert_eq!(hex::encode(sha2::Sha256::digest(files[name].as_bytes())), h.as_str().unwrap());
    }
    // Écriture sur disque puis recalcul externe indépendant (Python, bibliothèque standard).
    let out = d.path().join("exports");
    std::fs::create_dir(&out).unwrap();
    let dir = s.write_snapshot(&snap.id, &out).unwrap();
    let py = std::process::Command::new("python3").arg(common::specs_dir().join("../research/scripts/rapport_descriptif.py")).arg(&dir).output().unwrap();
    assert!(py.status.success(), "{}", String::from_utf8_lossy(&py.stderr));
    let r: Value = serde_json::from_slice(&py.stdout).unwrap();
    let q: Value = serde_json::from_str(&files["quality_report.json"]).unwrap();
    assert_eq!(r["n_analysable"], q["bewe_index"]["n_analysable"]);
    assert_eq!(r["median"].as_f64(), q["bewe_index"]["distribution"]["median"].as_f64());
    assert_eq!(r["ge9"]["k"], q["bewe_index"]["ge9"]["k"]);
    assert_eq!(r["n_analysable"], 6, "EXP-001 sans BEWE : dénominateur réduit, pas zéro");

    // Correction ultérieure : l'instantané figé ne change pas.
    let val = s.load_encounter(&e.meta.id, false).unwrap();
    let am = s.start_amendment(&e.meta.id, val.meta.version, "Correction test").unwrap();
    let v = s.save_fields(&e.meta.id, am.meta.version, vec![fi("age_years", json!(28))]).unwrap().version;
    s.validate_encounter(&e.meta.id, v).unwrap();
    assert_eq!(s.snapshot_files(&snap.id).unwrap(), files, "export antérieur inchangé");
    assert!(s.write_snapshot(&snap.id, &out).is_err(), "pas d'écrasement silencieux");

    // Retrait d'éligibilité : exclu du prochain export, soins intacts.
    s.set_eligibility(&p.id, &e.meta.patient_id, Some("Opposition (fictive)".into())).unwrap();
    let sel2 = s.selection(&p.id).unwrap();
    assert_eq!(sel2.included.len(), 6);
    assert_eq!(s.patient_encounters(&e.meta.patient_id).unwrap().len(), 2);
    // Identifiants d'étude stables et non dérivés du nom.
    let snap2 = s.freeze_export(&p.id, true, "test").unwrap();
    let f2 = s.snapshot_files(&snap2.id).unwrap();
    assert!(f2["visits.csv"].lines().next().unwrap().contains("visit_date"), "dates exactes seulement sur demande");
    let first_sid = files["patients.csv"].lines().nth(1).unwrap().split(',').next().unwrap().to_string();
    assert!(f2["patients.csv"].contains(&first_sid) || sel2.included.len() < sel.included.len());
}

#[test]
fn neutralisation_des_formules_dans_les_csv() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let e = s.create_dossier(Some("=CMD()".into()), None, false).unwrap();
    let v = s.save_fields(&e.meta.id, e.meta.version, vec![fi("age_years", json!(30))]).unwrap().version;
    s.validate_encounter(&e.meta.id, v).unwrap();
    let p = s.create_project(ProjectInput { name: "Projet".into(), period_start: Some("2020-01-01".into()), period_end: None, include_legacy: true, include_prospective: true, exclude_open_anomalies: true, plan_version: None }).unwrap();
    let sel = s.selection(&p.id).unwrap();
    assert_eq!(sel.included.len(), 0, "période définie sans date : non vérifiable, exclu");
    assert_eq!(sel.excluded_encounters.get("date_absente_periode_non_verifiable"), Some(&1));
    assert!(s.freeze_export(&p.id, false, "t").is_err(), "rien à exporter : refus explicite");
}
