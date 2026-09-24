//! Parcours de consultation : persistance fidèle, manques, BEWE, prévention, validation, amendement.

mod common;
use cmme_core::domain::values::FieldInput;
use cmme_core::store::encounters::{ExposurePatch, PreventionPatch, SextantInput};
use cmme_core::{CoreError, Store};
use serde_json::json;

fn fi(field: &str, value: serde_json::Value) -> FieldInput {
    FieldInput { field: field.into(), value: Some(value), ..Default::default() }
}
fn missing(field: &str, reason: &str) -> FieldInput {
    FieldInput { field: field.into(), missing_reason: Some(reason.into()), ..Default::default() }
}

#[test]
fn dossier_vierge_sans_reponse_inventee() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let e = s.create_dossier(None, None, false).unwrap();
    assert_eq!(e.values.len(), 1, "seule la date du jour est proposée");
    assert_eq!(e.values[0].field, "visit_date");
    assert_eq!(e.values[0].value_text.as_deref(), Some(chrono::Local::now().format("%Y-%m-%d").to_string().as_str()));
    assert!(e.bewe.is_empty() && e.exposures.is_empty() && e.prevention.is_empty());
    assert_eq!(e.meta.status, "draft");
    assert_eq!(e.meta.collection_mode, "structured_prospective");
    assert!(e.meta.bewe_total_derived.is_none());
}

#[test]
fn saisie_partielle_fermee_puis_rouverte_a_l_identique() {
    let d = common::tmpdir();
    let (mut s, path) = common::store_in(&d);
    let e = s.create_dossier(Some("TEST-001".into()), None, false).unwrap();
    let id = e.meta.id.clone();
    let mut v = e.meta.version;
    v = s.save_fields(&id, v, vec![
        fi("age_years", json!(26)),
        fi("brushing_daily", json!({"min": "2", "max": "3"})),
        fi("usual_dentist", json!("no")),
        missing("sex_recorded", "declined"),
        fi("hypersensitivity_intensity", json!(0)),
        fi("toothpaste_features", json!(["whitening", "charcoal"])),
        fi("consultation_observations", json!("  Texte libre\nsur deux lignes — avec accents éàü  ")),
        fi("visit_date", json!("2026-09")),
    ]).unwrap().version;
    v = s.set_sextant(&id, v, SextantInput { sextant: "lower_left".into(), score: Some(json!(0)), missing_reason: None, unassessable_reason: None }).unwrap().version;
    v = s.set_exposure_group(&id, v, "drink", false, vec!["soda_sugar".into(), "fruit_juice".into()]).unwrap().version;
    s.update_exposure(&id, v, "drink", "soda_sugar", ExposurePatch { freq_min: Some(json!("2")), freq_max: Some(json!("3")), freq_unit: Some("day".into()), temporality: Some("current".into()), quantity_text: Some("1 canette".into()) }).unwrap();
    drop(s);

    let s = Store::open(&path, &common::KEY, false, "Praticien test").unwrap();
    let r = s.load_encounter(&id, true).unwrap();
    let get = |f: &str| r.values.iter().find(|x| x.field == f).cloned().unwrap();
    assert_eq!(get("age_years").value_num, Some(26.0));
    let b = get("brushing_daily");
    assert_eq!((b.value_num, b.value_num_max, b.precision.as_deref()), (Some(2.0), Some(3.0), Some("range")), "plage conservée, pas 2,5");
    assert_eq!(get("usual_dentist").value_text.as_deref(), Some("no"));
    assert_eq!(get("sex_recorded").missing_reason.as_deref(), Some("declined"));
    assert_eq!(get("hypersensitivity_intensity").value_num, Some(0.0), "un zéro explicite reste zéro");
    assert_eq!(get("toothpaste_features").value_text.as_deref(), Some("[\"charcoal\",\"whitening\"]"));
    assert_eq!(get("consultation_observations").value_text.as_deref(), Some("  Texte libre\nsur deux lignes — avec accents éàü  "), "texte intact");
    assert_eq!(get("visit_date").date_precision.as_deref(), Some("month"), "aucun jour inventé");
    assert!(r.values.iter().all(|x| x.field != "dry_mouth_reported"), "jamais renseigné = absent");
    assert_eq!(r.bewe.len(), 1);
    assert_eq!(r.bewe[0].score, Some(0));
    assert!(r.meta.bewe_total_derived.is_none(), "1/6 sextant : pas de total");
    let soda = r.exposures.iter().find(|x| x.category == "soda_sugar").unwrap();
    assert_eq!((soda.freq_min, soda.freq_max, soda.freq_unit.as_deref()), (Some(2.0), Some(3.0), Some("day")));
    assert_eq!(r.exposures.len(), 2);
}

#[test]
fn a_puis_b_puis_a_sans_melange() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let a = s.create_dossier(Some("A".into()), None, false).unwrap();
    let b = s.create_dossier(Some("B".into()), None, false).unwrap();
    s.save_fields(&a.meta.id, a.meta.version, vec![fi("age_years", json!(30)), fi("toothpaste_features", json!(["charcoal"]))]).unwrap();
    s.save_fields(&b.meta.id, b.meta.version, vec![fi("age_years", json!(41)), fi("exam_note", json!("note B"))]).unwrap();
    let a2 = s.load_encounter(&a.meta.id, false).unwrap();
    assert_eq!(a2.values.len(), 3);
    assert!(a2.values.iter().all(|v| v.field != "exam_note"));
    assert_eq!(a2.values.iter().find(|v| v.field == "age_years").unwrap().value_num, Some(30.0));
}

#[test]
fn validations_refusees_sans_rien_ecrire() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let e = s.create_dossier(None, None, false).unwrap();
    let id = e.meta.id;
    let v = e.meta.version;
    for bad in [
        vec![fi("age_years", json!(26.5))],
        vec![fi("age_years", json!(-1))],
        vec![fi("service_code", json!("inconnu"))],
        vec![FieldInput { field: "age_years".into(), value: Some(json!(3)), missing_reason: Some("unknown".into()), ..Default::default() }],
        vec![fi("access_barrier", json!(["none", "cost"]))],
        vec![fi("age_years", json!(20)), fi("brushing_daily", json!({"min": 3, "max": 2}))],
        vec![fi("visit_date", json!("2026-02-30"))],
        vec![fi("vomiting_legacy_code", json!("yes"))],
    ] {
        assert!(matches!(s.save_fields(&id, v, bad), Err(CoreError::Validation { .. })));
    }
    for bad_score in [json!(4), json!(-1), json!(1.5), json!("2x")] {
        assert!(s.set_sextant(&id, v, SextantInput { sextant: "upper_left".into(), score: Some(bad_score), missing_reason: None, unassessable_reason: None }).is_err());
    }
    let r = s.load_encounter(&id, false).unwrap();
    assert!(r.values.len() == 1 && r.bewe.is_empty(), "aucune écriture partielle");
    assert_eq!(r.meta.version, v);
}

#[test]
fn conflit_de_version_detecte() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let e = s.create_dossier(None, None, false).unwrap();
    s.save_fields(&e.meta.id, e.meta.version, vec![fi("age_years", json!(20))]).unwrap();
    let r = s.save_fields(&e.meta.id, e.meta.version, vec![fi("age_years", json!(21))]);
    assert!(matches!(r, Err(CoreError::Conflict { .. })));
}

#[test]
fn bewe_total_seulement_avec_six_scores_et_non_evaluable_conserve() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let e = s.create_dossier(None, None, false).unwrap();
    let id = e.meta.id;
    let mut v = e.meta.version;
    let set = |s: &mut Store, v: i64, k: &str, sc: Option<i64>, m: Option<&str>, r: Option<&str>| {
        s.set_sextant(&id, v, SextantInput { sextant: k.into(), score: sc.map(|x| json!(x)), missing_reason: m.map(String::from), unassessable_reason: r.map(String::from) }).unwrap()
    };
    for (k, sc) in [("upper_right", 2), ("upper_anterior", 2), ("upper_left", 2), ("lower_left", 1), ("lower_anterior", 1)] {
        let r = set(&mut s, v, k, Some(sc), None, None);
        assert_eq!(r.bewe_total_derived, None, "jamais de somme partielle");
        v = r.version;
    }
    let r = set(&mut s, v, "lower_right", None, Some("not_assessable"), Some("edentulous"));
    assert_eq!(r.bewe_total_derived, None, "non évaluable : pas de total, pas de zéro implicite");
    v = r.version;
    let r = set(&mut s, v, "lower_right", Some(1), None, None);
    assert_eq!(r.bewe_total_derived, Some(9));
    let full = s.load_encounter(&id, false).unwrap();
    assert_eq!(full.bewe.len(), 6);
    // Motif de non-évaluation incohérent refusé.
    assert!(s.set_sextant(&id, r.version, SextantInput { sextant: "upper_left".into(), score: Some(json!(1)), missing_reason: None, unassessable_reason: Some("edentulous".into()) }).is_err());
}

#[test]
fn alimentation_aucune_exclusive_et_decocher_sans_perte() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let e = s.create_dossier(None, None, false).unwrap();
    let id = e.meta.id;
    let mut v = e.meta.version;
    assert!(s.set_exposure_group(&id, v, "food", true, vec!["citrus".into()]).is_err());
    v = s.set_exposure_group(&id, v, "food", false, vec!["citrus".into(), "vinegar".into(), "pickles".into()]).unwrap().version;
    v = s.update_exposure(&id, v, "food", "citrus", ExposurePatch { freq_min: Some(json!(3)), freq_unit: Some("week".into()), ..Default::default() }).unwrap().version;
    v = s.set_exposure_group(&id, v, "food", false, vec!["citrus".into(), "pickles".into()]).unwrap().version;
    let r = s.load_encounter(&id, false).unwrap();
    assert_eq!(r.exposures.iter().map(|x| x.category.as_str()).collect::<Vec<_>>(), vec!["citrus", "pickles"]);
    assert_eq!(r.exposures[0].freq_min, Some(3.0), "détail des autres choix conservé");
    assert_eq!(r.values.iter().find(|x| x.field == "acidic_foods_status").unwrap().value_text.as_deref(), Some("reported"));
    v = s.set_exposure_group(&id, v, "food", false, vec![]).unwrap().version;
    let r = s.load_encounter(&id, false).unwrap();
    assert!(r.values.iter().all(|x| x.field != "acidic_foods_status"), "menu vide = non renseigné");
    s.set_exposure_group(&id, v, "food", true, vec![]).unwrap();
    let r = s.load_encounter(&id, false).unwrap();
    assert_eq!(r.values.iter().find(|x| x.field == "acidic_foods_status").unwrap().value_text.as_deref(), Some("none_reported"));
}

#[test]
fn prevention_hbd_independant_et_protocoles() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let e = s.create_dossier(None, None, false).unwrap();
    let id = e.meta.id;
    let mut v = e.meta.version;
    // HBD seul avec « Aucun protocole proposé » : enregistrable.
    v = s.save_fields(&id, v, vec![fi("hbd_teaching", json!("done"))]).unwrap().version;
    v = s.apply_protocol(&id, v, Some("none".into())).unwrap().save.version;
    let r = s.load_encounter(&id, false).unwrap();
    assert!(r.prevention.is_empty());
    // Modéré puis Avancé : HBD conservé, gouttière vomissements jamais cochée.
    let c = s.apply_protocol(&id, v, Some("moderate".into())).unwrap();
    assert_eq!(c.proposed, vec!["anti_erosion_toothpaste", "anti_erosion_rinse"]);
    v = c.save.version;
    let c = s.apply_protocol(&id, v, Some("advanced".into())).unwrap();
    assert_eq!(c.proposed, vec!["tooth_mousse"]);
    v = c.save.version;
    let r = s.load_encounter(&id, false).unwrap();
    assert_eq!(r.values.iter().find(|x| x.field == "hbd_teaching").unwrap().value_text.as_deref(), Some("done"));
    assert!(r.prevention.iter().all(|p| p.action_type != "vomiting_semirigid_tray"));
    assert!(r.prevention.iter().all(|p| !p.confirmed), "propositions non confirmées");
    let tm = r.prevention.iter().find(|p| p.action_type == "tooth_mousse").unwrap();
    assert_eq!(tm.route, None, "mode d'application à renseigner");
    assert_eq!(tm.tray_minutes, None);
    // Voie directe : pas de durée en gouttière.
    assert!(s.update_prevention_action(&id, v, "tooth_mousse", PreventionPatch { route: Some("direct".into()), tray_minutes: Some(json!(10)), ..Default::default() }).is_ok());
    let r = s.load_encounter(&id, false).unwrap();
    v = r.meta.version;
    assert_eq!(r.prevention.iter().find(|p| p.action_type == "tooth_mousse").unwrap().tray_minutes, None);
    v = s.update_prevention_action(&id, v, "tooth_mousse", PreventionPatch { route: Some("tray".into()), tray_minutes: Some(json!("10")), ..Default::default() }).unwrap().version;
    let r = s.load_encounter(&id, false).unwrap();
    assert_eq!(r.prevention.iter().find(|p| p.action_type == "tooth_mousse").unwrap().tray_minutes, Some(10));
    // Confirmation, puis retour à Modéré : Tooth Mousse confirmé conservé et signalé.
    v = s.confirm_prevention(&id, v).unwrap().version;
    let c = s.apply_protocol(&id, v, Some("moderate".into())).unwrap();
    assert_eq!(c.kept_confirmed_outside, vec!["tooth_mousse"]);
    assert!(c.removed_unconfirmed.is_empty());
    v = c.save.version;
    // Gouttière semi-rigide : option indépendante, explicitement choisie.
    s.update_prevention_action(&id, v, "vomiting_semirigid_tray", PreventionPatch { decision: Some("not_applicable".into()), ..Default::default() }).unwrap();
    let r = s.load_encounter(&id, false).unwrap();
    assert_eq!(r.prevention.iter().find(|p| p.action_type == "vomiting_semirigid_tray").unwrap().decision.as_deref(), Some("not_applicable"));
    // Le protocole ne se modifie pas par l'enregistrement générique.
    assert!(s.save_fields(&id, r.meta.version, vec![fi("prevention_protocol", json!("advanced"))]).is_err());
}

#[test]
fn propositions_non_confirmees_retirees_au_changement() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let e = s.create_dossier(None, None, false).unwrap();
    let id = e.meta.id;
    let v = s.apply_protocol(&id, e.meta.version, Some("advanced".into())).unwrap().save.version;
    let c = s.apply_protocol(&id, v, Some("moderate".into())).unwrap();
    assert_eq!(c.removed_unconfirmed, vec!["tooth_mousse"]);
    let r = s.recap(&id).unwrap();
    assert_eq!(r.unconfirmed_prevention.len(), 2);
}

#[test]
fn terminer_valide_verrouille_et_amendement_trace() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let e = s.create_dossier(None, None, false).unwrap();
    let id = e.meta.id;
    let v = s.save_fields(&id, e.meta.version, vec![fi("age_years", json!(22)), fi("consultation_observations", json!("Obs initiale"))]).unwrap().version;
    let recap = s.recap(&id).unwrap();
    assert!(recap.missing_essentials.iter().any(|m| m.contains("BEWE")), "manques signalés, non bloquants");
    let val = s.validate_encounter(&id, v).unwrap();
    assert_eq!(val.meta.status, "validated");
    assert_eq!(val.meta.revision, 1);
    // Verrouillée.
    assert!(matches!(s.save_fields(&id, val.meta.version, vec![fi("age_years", json!(23))]), Err(CoreError::Locked)));
    // Motif obligatoire.
    assert!(s.start_amendment(&id, val.meta.version, "").is_err());
    let am = s.start_amendment(&id, val.meta.version, "Erreur de saisie de l'âge").unwrap();
    let v = s.save_fields(&id, am.meta.version, vec![fi("age_years", json!(23))]).unwrap().version;
    let fin = s.validate_encounter(&id, v).unwrap();
    assert_eq!(fin.meta.revision, 2);
    let revs = s.revisions(&id).unwrap();
    assert_eq!(revs.len(), 2);
    let old_age = revs[0].snapshot["values"].as_array().unwrap().iter().find(|x| x["field"] == "age_years").unwrap()["value_num"].as_f64();
    assert_eq!(old_age, Some(22.0), "ancien état consultable");
    assert_eq!(revs[1].reason.as_deref(), Some("Erreur de saisie de l'âge"));
    let audit = s.audit_for(&id).unwrap();
    let change = audit.iter().find(|a| a.field.as_deref() == Some("age_years") && a.reason.is_some()).unwrap();
    assert!(change.old_value.as_ref().unwrap().contains("22") && change.new_value.as_ref().unwrap().contains("23"));
    assert_eq!(change.author.as_deref(), Some("Praticien test"));
}

#[test]
fn ancien_vomissement_arrete_et_absence_actuelle_coexistent() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let e = s.create_dossier(None, None, false).unwrap();
    s.save_fields(&e.meta.id, e.meta.version, vec![fi("vomiting_lifetime", json!("past_only")), fi("vomiting_stop_months", json!(24)), fi("compensatory_behaviors", json!(["vomiting"]))]).unwrap();
    let r = s.load_encounter(&e.meta.id, false).unwrap();
    assert_eq!(r.values.len(), 4);
}

#[test]
fn erreur_disque_sans_faux_succes() {
    use std::os::unix::fs::PermissionsExt;
    let d = common::tmpdir();
    let (mut s, path) = common::store_in(&d);
    let e = s.create_dossier(None, None, false).unwrap();
    drop(s);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();
    let mut s = Store::open(&path, &common::KEY, false, "t").unwrap();
    let r = s.save_fields(&e.meta.id, e.meta.version, vec![fi("age_years", json!(30))]);
    assert!(matches!(r, Err(CoreError::Storage(_))), "l'échec d'écriture remonte comme erreur : {r:?}");
    drop(s);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    let mut s = Store::open(&path, &common::KEY, false, "t").unwrap();
    assert!(s.load_encounter(&e.meta.id, false).unwrap().values.iter().all(|v| v.field != "age_years"), "rien d'écrit pendant l'échec");
    let r = s.save_fields(&e.meta.id, e.meta.version, vec![fi("age_years", json!(30))]);
    assert!(r.is_ok(), "réessayer (connexion rouverte) fonctionne une fois le disque disponible");
}

#[test]
fn base_illisible_sans_la_bonne_cle() {
    let d = common::tmpdir();
    let (mut s, path) = common::store_in(&d);
    let e = s.create_dossier(Some("SECRET-CODE-XYZ".into()), None, false).unwrap();
    s.save_fields(&e.meta.id, e.meta.version, vec![fi("consultation_observations", json!("OBSERVATION-FICTIVE-TRES-RECONNAISSABLE"))]).unwrap();
    drop(s);
    let bytes = std::fs::read(&path).unwrap();
    assert!(!bytes.starts_with(b"SQLite format 3"));
    for needle in [&b"SECRET-CODE-XYZ"[..], b"OBSERVATION-FICTIVE"] {
        assert!(!bytes.windows(needle.len()).any(|w| w == needle));
    }
    assert!(matches!(Store::open(&path, &[9u8; 32], false, "t"), Err(CoreError::BadSecret)));
    // Aucun fichier annexe en clair laissé à côté de la base.
    let others: Vec<_> = std::fs::read_dir(d.path()).unwrap().filter_map(|e| e.ok()).map(|e| e.file_name()).collect();
    assert_eq!(others.len(), 1, "fichiers présents : {others:?}");
}
