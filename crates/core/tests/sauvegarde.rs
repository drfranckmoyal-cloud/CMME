//! Sauvegarde / restauration chiffrées, mot de passe, clé de trousseau (coffre en mémoire pour le test).

mod common;
use cmme_core::backup::{self, inspect_backup};
use cmme_core::domain::values::FieldInput;
use cmme_core::keystore::{generate_key, KeyStore, MemoryKeyStore};
use cmme_core::{CoreError, Store};
use serde_json::json;

const PHRASE: &str = "cheval-agrafe-lune-42";

fn fi(field: &str, value: serde_json::Value) -> FieldInput {
    FieldInput { field: field.into(), value: Some(value), ..Default::default() }
}

#[test]
fn sauvegarder_modifier_restaurer_sur_un_poste_vierge() {
    let d = common::tmpdir();
    let (mut s, _) = common::store_in(&d);
    let e = s.create_dossier(Some("SAV-001".into()), None, false).unwrap();
    s.save_fields(&e.meta.id, e.meta.version, vec![fi("age_years", json!(33)), fi("consultation_observations", json!("Observation fictive"))]).unwrap();
    let dest = d.path().join("sauvegardes");
    std::fs::create_dir(&dest).unwrap();
    let info = s.create_backup(&dest, PHRASE, "test").unwrap();
    let bytes = std::fs::read(&info.path).unwrap();
    assert!(!bytes.starts_with(b"SQLite format 3"), "sauvegarde chiffrée");
    assert!(!bytes.windows(19).any(|w| w == b"Observation fictive"));
    assert_eq!(info.meta.counts["encounters"], 1);

    // Modification après la sauvegarde.
    let cur = s.load_encounter(&e.meta.id, false).unwrap();
    s.save_fields(&e.meta.id, cur.meta.version, vec![fi("age_years", json!(99))]).unwrap();
    s.create_dossier(Some("SAV-002".into()), None, false).unwrap();

    // « Nouveau Mac » : trousseau vide, nouvelle clé, aucune base.
    let ks = MemoryKeyStore::default();
    assert!(ks.get("clinique").unwrap().is_none());
    let new_key = generate_key().unwrap();
    ks.set("clinique", &new_key).unwrap();
    let d2 = common::tmpdir();
    let target = d2.path().join("cmme.db");
    let (tmp, meta) = backup::prepare_restore(std::path::Path::new(&info.path), PHRASE, &target, &new_key).unwrap();
    assert_eq!(meta.counts["patients"], 1);
    backup::commit_restore(&tmp, &target).unwrap();
    let r = Store::open(&target, &ks.get("clinique").unwrap().unwrap(), false, "t").unwrap();
    let rows = r.all_rows(false).unwrap();
    assert_eq!(rows.len(), 1, "état exact de la sauvegarde");
    let full = r.load_encounter(&e.meta.id, false).unwrap();
    assert_eq!(full.values.iter().find(|v| v.field == "age_years").unwrap().value_num, Some(33.0));
    assert!(!r.audit_for(&e.meta.id).unwrap().is_empty(), "journal d'audit restauré");
    // La clé de l'ancien poste n'ouvre pas la base restaurée.
    drop(r);
    assert!(matches!(Store::open(&target, &common::KEY, false, "t"), Err(CoreError::BadSecret)));
}

#[test]
fn mauvaise_phrase_et_fichier_tronque_sans_ecrasement() {
    let d = common::tmpdir();
    let (mut s, db_path) = common::store_in(&d);
    s.create_dossier(Some("KEEP-1".into()), None, false).unwrap();
    let dest = d.path().join("b");
    std::fs::create_dir(&dest).unwrap();
    let info = s.create_backup(&dest, PHRASE, "test").unwrap();
    let p = std::path::Path::new(&info.path);
    assert!(matches!(inspect_backup(p, "mauvaise-phrase-123"), Err(CoreError::BadSecret)));
    assert!(backup::prepare_restore(p, "mauvaise-phrase-123", &db_path, &common::KEY).is_err());
    // Fichier tronqué.
    let bytes = std::fs::read(p).unwrap();
    let cut = dest.join("tronque.cmmebak");
    std::fs::write(&cut, &bytes[..bytes.len() / 2]).unwrap();
    assert!(backup::prepare_restore(&cut, PHRASE, &db_path, &common::KEY).is_err());
    // Octet altéré au milieu : refus (HMAC par page).
    let mut alt = bytes.clone();
    let mid = alt.len() / 2;
    alt[mid] ^= 0xFF;
    let altp = dest.join("altere.cmmebak");
    std::fs::write(&altp, &alt).unwrap();
    assert!(inspect_backup(&altp, PHRASE).is_err());
    // L'état courant est intact, aucun fichier temporaire laissé.
    assert_eq!(s.all_rows(false).unwrap().len(), 1);
    let leftovers: Vec<_> = std::fs::read_dir(d.path()).unwrap().filter_map(|e| e.ok()).filter(|e| e.file_name().to_string_lossy().ends_with(".tmp")).collect();
    assert!(leftovers.is_empty());
}

#[test]
fn phrase_trop_courte_et_destination_impossible() {
    let d = common::tmpdir();
    let (s, _) = common::store_in(&d);
    assert!(s.create_backup(d.path(), "court", "t").is_err());
    assert!(s.create_backup(&d.path().join("absent"), PHRASE, "t").is_err());
    // Dossier en lecture seule : erreur explicite, pas de faux succès ni de fichier final.
    use std::os::unix::fs::PermissionsExt;
    let ro = d.path().join("ro");
    std::fs::create_dir(&ro).unwrap();
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o555)).unwrap();
    assert!(s.create_backup(&ro, PHRASE, "t").is_err());
    assert_eq!(std::fs::read_dir(&ro).unwrap().count(), 0);
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn mot_de_passe_application() {
    let d = common::tmpdir();
    let (s, _) = common::store_in(&d);
    assert!(!s.has_app_password().unwrap());
    assert!(s.set_app_password(None, "court").is_err());
    s.set_app_password(None, "motdepasse-fictif").unwrap();
    assert!(s.verify_app_password("motdepasse-fictif").unwrap());
    assert!(!s.verify_app_password("autre").unwrap());
    assert!(matches!(s.set_app_password(Some("faux"), "nouveau-mdp-123"), Err(CoreError::BadSecret)));
    s.set_app_password(Some("motdepasse-fictif"), "nouveau-mdp-123").unwrap();
    assert!(s.verify_app_password("nouveau-mdp-123").unwrap());
    let h = s.setting("app_password").unwrap().unwrap();
    assert!(h.starts_with("$argon2id$") && !h.contains("nouveau"));
}
