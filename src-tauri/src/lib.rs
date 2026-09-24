//! Coquille Tauri : commandes typées exposées à l'interface. Aucune requête SQL côté interface ;
//! chaque réponse positive correspond à une écriture réellement confirmée par le cœur.

use cmme_core::backup;
use cmme_core::domain::bewe::{SEXTANTS, UNASSESSABLE_REASONS};
use cmme_core::domain::catalog::{self, DRINKS, FOODS, PREVENTION_ACTIONS};
use cmme_core::domain::missing::MissingReason;
use cmme_core::domain::values::FieldInput;
use cmme_core::error::ErrorPayload;
use cmme_core::import::{self, SheetConfig};
use cmme_core::keystore::{generate_key, KeyStore, KeychainStore};
use cmme_core::research::ProjectInput;
use cmme_core::store::encounters::{ExposurePatch, IdentityInput, PreventionPatch, SextantInput};
use cmme_core::store::listing::Filter;
use cmme_core::{CoreError, Store};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{Manager, State};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

type R<T> = std::result::Result<T, ErrorPayload>;

fn err(e: CoreError) -> ErrorPayload {
    ErrorPayload::from(&e)
}

/// Clés de base : trousseau macOS en version publiée. En développement uniquement (données fictives),
/// un fichier local évite les demandes d'autorisation du trousseau à chaque recompilation.
fn keystore(data_dir: &Path) -> Box<dyn KeyStore> {
    if cfg!(debug_assertions) {
        Box::new(DevFileKeyStore { dir: data_dir.join("dev-keys") })
    } else {
        Box::new(KeychainStore { service: "fr.cmme.recueil".into() })
    }
}

struct DevFileKeyStore {
    dir: PathBuf,
}
impl KeyStore for DevFileKeyStore {
    fn get(&self, account: &str) -> cmme_core::Result<Option<[u8; 32]>> {
        match std::fs::read_to_string(self.dir.join(account)) {
            Ok(h) => {
                let b = hex_decode(h.trim()).ok_or_else(|| CoreError::Keychain("clé de développement illisible".into()))?;
                Ok(Some(b))
            }
            Err(_) => Ok(None),
        }
    }
    fn set(&self, account: &str, key: &[u8; 32]) -> cmme_core::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        std::fs::write(self.dir.join(account), key.iter().map(|b| format!("{b:02x}")).collect::<String>())?;
        Ok(())
    }
    fn delete(&self, account: &str) -> cmme_core::Result<()> {
        let _ = std::fs::remove_file(self.dir.join(account));
        Ok(())
    }
}
fn hex_decode(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).ok()?;
    }
    Some(out)
}

pub struct AppState {
    store: Mutex<Option<Store>>,
    profile: Mutex<Option<String>>,
    data_dir: PathBuf,
    keys: Box<dyn KeyStore>,
}

impl AppState {
    fn db_path(&self, profile: &str) -> PathBuf {
        self.data_dir.join(profile).join("cmme.db")
    }
    fn account(profile: &str) -> String {
        format!("db-{profile}")
    }
    fn with<T>(&self, f: impl FnOnce(&mut Store) -> cmme_core::Result<T>) -> R<T> {
        let mut g = self.store.lock().map_err(|_| err(CoreError::Storage("verrou interne".into())))?;
        let s = g.as_mut().ok_or_else(|| ErrorPayload { kind: "locked_app", message: "Application verrouillée : saisissez le mot de passe.".into() })?;
        f(s).map_err(err)
    }
    fn is_demo(&self) -> bool {
        self.profile.lock().map(|p| p.as_deref() == Some("demo")).unwrap_or(false)
    }
}

fn check_profile(p: &str) -> R<()> {
    if p == "clinique" || p == "demo" {
        Ok(())
    } else {
        Err(err(CoreError::validation("profil", "profil inconnu")))
    }
}

// ------------------------------------------------------------------ Accès

#[derive(Serialize)]
struct Status {
    app_version: &'static str,
    unlocked: bool,
    profile: Option<String>,
    clinique_exists: bool,
    demo_exists: bool,
    practitioner: Option<String>,
    cipher_version: Option<String>,
    last_backup_at: Option<String>,
    data_dir: String,
    keychain: &'static str,
    macos_version: Option<String>,
}

#[tauri::command]
fn app_status(state: State<AppState>) -> R<Status> {
    let g = state.store.lock().unwrap();
    let macos = std::process::Command::new("sw_vers").arg("-productVersion").output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    Ok(Status {
        app_version: APP_VERSION,
        unlocked: g.is_some(),
        profile: state.profile.lock().unwrap().clone(),
        clinique_exists: state.db_path("clinique").exists(),
        demo_exists: state.db_path("demo").exists(),
        practitioner: g.as_ref().and_then(|s| s.setting("practitioner_name").ok().flatten()),
        cipher_version: g.as_ref().and_then(|s| s.cipher_version().ok()),
        last_backup_at: g.as_ref().and_then(|s| s.setting("last_backup_at").ok().flatten()),
        data_dir: state.data_dir.to_string_lossy().into(),
        keychain: if cfg!(debug_assertions) { "fichier de développement (données fictives uniquement)" } else { "trousseau macOS" },
        macos_version: macos,
    })
}

#[tauri::command]
fn setup_clinique(state: State<AppState>, practitioner: String, password: String) -> R<()> {
    let path = state.db_path("clinique");
    if path.exists() {
        return Err(err(CoreError::Refused("un profil clinique existe déjà".into())));
    }
    if practitioner.trim().len() < 2 {
        return Err(err(CoreError::validation("praticien", "nom du praticien requis")));
    }
    if password.chars().count() < cmme_core::auth::MIN_LEN {
        return Err(err(CoreError::validation("mot de passe", "8 caractères au moins")));
    }
    let key = generate_key().map_err(err)?;
    state.keys.set(&AppState::account("clinique"), &key).map_err(err)?;
    let s = Store::open(&path, &key, true, practitioner.trim()).map_err(err)?;
    s.set_setting("practitioner_name", practitioner.trim()).map_err(err)?;
    s.set_app_password(None, &password).map_err(err)?;
    *state.store.lock().unwrap() = Some(s);
    *state.profile.lock().unwrap() = Some("clinique".into());
    Ok(())
}

#[tauri::command]
fn unlock(state: State<AppState>, profile: String, password: Option<String>) -> R<()> {
    check_profile(&profile)?;
    let path = state.db_path(&profile);
    let account = AppState::account(&profile);
    let store = if profile == "demo" {
        let key = match state.keys.get(&account).map_err(err)? {
            Some(k) => k,
            None => {
                if path.exists() {
                    // Base de démonstration sans clé : fictive, on la recrée.
                    let _ = std::fs::remove_file(&path);
                }
                let k = generate_key().map_err(err)?;
                state.keys.set(&account, &k).map_err(err)?;
                k
            }
        };
        let mut s = Store::open(&path, &key, !path.exists(), "Démonstration").map_err(err)?;
        cmme_core::demo::seed_demo(&mut s).map_err(err)?;
        s
    } else {
        let key = state.keys.get(&account).map_err(err)?.ok_or_else(|| err(CoreError::Keychain("clé absente : restaurez une sauvegarde avec votre phrase de récupération".into())))?;
        let author = "Praticien";
        let mut s = Store::open(&path, &key, false, author).map_err(err)?;
        let ok = s.verify_app_password(password.as_deref().unwrap_or("")).map_err(err)?;
        if !ok {
            std::thread::sleep(std::time::Duration::from_millis(800));
            return Err(err(CoreError::BadSecret));
        }
        if let Some(n) = s.setting("practitioner_name").map_err(err)? {
            s.author = n;
        }
        s.audit("unlock", "app", None, None, None, None, None).map_err(err)?;
        s
    };
    *state.store.lock().unwrap() = Some(store);
    *state.profile.lock().unwrap() = Some(profile);
    Ok(())
}

#[tauri::command]
fn lock(state: State<AppState>) -> R<()> {
    *state.store.lock().unwrap() = None;
    Ok(())
}

/// Réessayer après une erreur disque : rouvre la base avec la même clé (connexion neuve).
#[tauri::command]
fn reconnect(state: State<AppState>) -> R<()> {
    let profile = state.profile.lock().unwrap().clone().ok_or_else(|| err(CoreError::Refused("aucun profil ouvert".into())))?;
    let mut g = state.store.lock().unwrap();
    let author = g.as_ref().map(|s| s.author.clone()).ok_or_else(|| err(CoreError::Refused("application verrouillée".into())))?;
    let key = state.keys.get(&AppState::account(&profile)).map_err(err)?.ok_or_else(|| err(CoreError::Keychain("clé absente".into())))?;
    let s = Store::open(&state.db_path(&profile), &key, false, &author).map_err(err)?;
    *g = Some(s);
    Ok(())
}

#[tauri::command]
fn change_password(state: State<AppState>, current: String, new_password: String) -> R<()> {
    state.with(|s| s.set_app_password(Some(&current), &new_password))
}

#[tauri::command]
fn verify_password(state: State<AppState>, password: String) -> R<bool> {
    if state.is_demo() {
        return Ok(true);
    }
    state.with(|s| s.verify_app_password(&password))
}

// ------------------------------------------------------------------ Référentiels

#[tauri::command]
fn get_catalog() -> Value {
    json!({
        "catalog": catalog::catalog(),
        "sextants": SEXTANTS.iter().map(|(k, l, t)| json!({"key": k, "label": l, "teeth": t})).collect::<Vec<_>>(),
        "unassessable_reasons": UNASSESSABLE_REASONS.iter().map(|(k, l)| json!({"code": k, "label": l})).collect::<Vec<_>>(),
        "drinks": DRINKS.iter().map(|(k, l)| json!({"code": k, "label": l})).collect::<Vec<_>>(),
        "foods": FOODS.iter().map(|(k, l)| json!({"code": k, "label": l})).collect::<Vec<_>>(),
        "prevention_actions": PREVENTION_ACTIONS.iter().map(|(k, l)| json!({"code": k, "label": l})).collect::<Vec<_>>(),
        "missing_reasons": MissingReason::ALL.iter().map(|m| json!({"code": m.as_str(), "label": m.label_fr()})).collect::<Vec<_>>(),
        "import_targets": import::TARGETS.iter().map(|(k, l)| json!({"code": k, "label": l})).collect::<Vec<_>>(),
    })
}

// ------------------------------------------------------------------ Consultation

#[tauri::command]
fn create_dossier(state: State<AppState>, code: Option<String>, identity: Option<IdentityInput>) -> R<Value> {
    let demo = state.is_demo();
    state.with(|s| Ok(serde_json::to_value(s.create_dossier(code, identity, demo)?)?))
}

#[tauri::command]
fn new_encounter(state: State<AppState>, patient_id: String) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.new_encounter_for_patient(&patient_id)?)?))
}

#[tauri::command]
fn load_encounter(state: State<AppState>, id: String) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.load_encounter(&id, true)?)?))
}

#[tauri::command]
fn update_identity(state: State<AppState>, patient_id: String, identity: IdentityInput) -> R<()> {
    state.with(|s| s.update_identity(&patient_id, identity))
}

#[tauri::command]
fn save_fields(state: State<AppState>, id: String, version: i64, inputs: Vec<FieldInput>) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.save_fields(&id, version, inputs)?)?))
}

#[tauri::command]
fn set_sextant(state: State<AppState>, id: String, version: i64, input: SextantInput) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.set_sextant(&id, version, input)?)?))
}

#[tauri::command]
fn set_exposure_group(state: State<AppState>, id: String, version: i64, group: String, none_reported: bool, categories: Vec<String>) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.set_exposure_group(&id, version, &group, none_reported, categories)?)?))
}

#[tauri::command]
fn update_exposure(state: State<AppState>, id: String, version: i64, group: String, category: String, patch: ExposurePatch) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.update_exposure(&id, version, &group, &category, patch)?)?))
}

#[tauri::command]
fn apply_protocol(state: State<AppState>, id: String, version: i64, protocol: Option<String>) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.apply_protocol(&id, version, protocol)?)?))
}

#[tauri::command]
fn update_prevention_action(state: State<AppState>, id: String, version: i64, action: String, patch: PreventionPatch) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.update_prevention_action(&id, version, &action, patch)?)?))
}

#[tauri::command]
fn confirm_prevention(state: State<AppState>, id: String, version: i64) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.confirm_prevention(&id, version)?)?))
}

#[tauri::command]
fn recap(state: State<AppState>, id: String) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.recap(&id)?)?))
}

#[tauri::command]
fn validate_encounter(state: State<AppState>, id: String, version: i64) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.validate_encounter(&id, version)?)?))
}

#[tauri::command]
fn start_amendment(state: State<AppState>, id: String, version: i64, reason: String) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.start_amendment(&id, version, &reason)?)?))
}

#[tauri::command]
fn history(state: State<AppState>, id: String) -> R<Value> {
    state.with(|s| Ok(json!({"revisions": s.revisions(&id)?, "audit": s.audit_for(&id)?})))
}

#[tauri::command]
fn discard_empty_draft(state: State<AppState>, id: String) -> R<()> {
    state.with(|s| s.discard_empty_draft(&id))
}

// ------------------------------------------------------------------ Tableau, statistiques

#[tauri::command]
fn list_encounters(state: State<AppState>, filter: Filter, with_identity: bool) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.list_encounters(&filter, with_identity)?)?))
}

#[tauri::command]
fn patient_encounters(state: State<AppState>, patient_id: String) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.patient_encounters(&patient_id)?)?))
}

#[tauri::command]
fn stats(state: State<AppState>, filter: Filter) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.stats(&filter)?)?))
}

#[tauri::command]
fn home_summary(state: State<AppState>) -> R<Value> {
    let clinical = !state.is_demo();
    state.with(|s| {
        let rows = s.all_rows(clinical)?;
        let docs = s.import_documents()?;
        Ok(json!({
            "drafts": rows.iter().filter(|r| r.status != "validated").count(),
            "visits": rows.len(),
            "patients": rows.iter().map(|r| r.patient_id.clone()).collect::<std::collections::HashSet<_>>().len(),
            "anomalies": rows.iter().filter(|r| r.open_flags > 0).count(),
            "import_pending": docs.iter().map(|d| d.counts.get("pending").copied().unwrap_or(0)).sum::<i64>(),
            "last_backup_at": s.setting("last_backup_at")?,
            "recent": rows.iter().take(6).collect::<Vec<_>>(),
        }))
    })
}

// ------------------------------------------------------------------ Import

#[tauri::command]
fn import_preview(state: State<AppState>, path: String) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(import::preview(s, Path::new(&path))?)?))
}

#[tauri::command]
fn import_stage(state: State<AppState>, path: String, config: Vec<SheetConfig>) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.import_stage(Path::new(&path), config)?)?))
}

#[tauri::command]
fn import_documents(state: State<AppState>) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.import_documents()?)?))
}

#[tauri::command]
fn import_records(state: State<AppState>, document_id: String, status: Option<String>) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.import_records(&document_id, status.as_deref())?)?))
}

#[tauri::command]
fn import_decide(state: State<AppState>, record_id: String, target: String, decision: String, value: Option<Value>, justification: Option<String>) -> R<()> {
    state.with(|s| s.import_decide(&record_id, &target, &decision, value, justification))
}

#[tauri::command]
fn import_link(state: State<AppState>, record_id: String, decision: String, patient_id: Option<String>) -> R<()> {
    state.with(|s| s.import_link(&record_id, &decision, patient_id))
}

#[tauri::command]
fn import_exclude(state: State<AppState>, record_id: String, reason: String) -> R<()> {
    state.with(|s| s.import_exclude(&record_id, &reason))
}

#[tauri::command]
fn import_reinclude(state: State<AppState>, record_id: String) -> R<()> {
    state.with(|s| s.import_reinclude(&record_id))
}

#[tauri::command]
fn import_commit(state: State<AppState>, document_id: String, record_ids: Option<Vec<String>>) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.import_commit(&document_id, record_ids)?)?))
}

#[tauri::command]
fn merge_patients(state: State<AppState>, source: String, target: String, reason: String) -> R<String> {
    state.with(|s| s.merge_patients(&source, &target, &reason))
}

#[tauri::command]
fn undo_merge(state: State<AppState>, merge_id: String) -> R<()> {
    state.with(|s| s.undo_merge(&merge_id))
}

#[tauri::command]
fn merges(state: State<AppState>) -> R<Value> {
    state.with(|s| Ok(json!({"merges": s.merges()?, "candidates": s.merge_candidates()?})))
}

// ------------------------------------------------------------------ Étude et export

#[tauri::command]
fn projects(state: State<AppState>) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.projects()?)?))
}

#[tauri::command]
fn create_project(state: State<AppState>, input: ProjectInput) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.create_project(input)?)?))
}

#[tauri::command]
fn selection(state: State<AppState>, project_id: String) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.selection(&project_id)?)?))
}

#[tauri::command]
fn set_eligibility(state: State<AppState>, project_id: String, patient_id: String, excluded_reason: Option<String>) -> R<()> {
    state.with(|s| s.set_eligibility(&project_id, &patient_id, excluded_reason))
}

#[tauri::command]
fn freeze_export(state: State<AppState>, project_id: String, exact_dates: bool) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.freeze_export(&project_id, exact_dates, APP_VERSION)?)?))
}

#[tauri::command]
fn snapshots(state: State<AppState>, project_id: String) -> R<Value> {
    state.with(|s| Ok(serde_json::to_value(s.snapshots(&project_id)?)?))
}

fn refuse_icloud(dir: &Path) -> R<()> {
    let home = std::env::var("HOME").unwrap_or_default();
    let d = dir.to_string_lossy();
    for synced in ["Desktop", "Documents", "Library/Mobile Documents"] {
        if d.starts_with(&format!("{home}/{synced}")) {
            return Err(err(CoreError::Refused(format!("dossier synchronisé avec iCloud (~/{synced}) : choisissez un dossier local ou une clé USB autorisée"))));
        }
    }
    Ok(())
}

#[tauri::command]
fn write_snapshot(state: State<AppState>, snapshot_id: String, dir: String) -> R<String> {
    refuse_icloud(Path::new(&dir))?;
    state.with(|s| s.write_snapshot(&snapshot_id, Path::new(&dir)))
}

// ------------------------------------------------------------------ Sauvegarde

#[tauri::command]
fn backup_create(state: State<AppState>, dir: String, passphrase: String) -> R<Value> {
    refuse_icloud(Path::new(&dir))?;
    state.with(|s| Ok(serde_json::to_value(s.create_backup(Path::new(&dir), &passphrase, APP_VERSION)?)?))
}

#[tauri::command]
fn backup_inspect(path: String, passphrase: String) -> R<Value> {
    Ok(serde_json::to_value(backup::inspect_backup(Path::new(&path), &passphrase).map_err(err)?).unwrap())
}

/// Restauration depuis l'application ouverte : réauthentification, copie de sécurité, remplacement atomique.
#[tauri::command]
fn backup_restore(state: State<AppState>, path: String, passphrase: String, password: String) -> R<Value> {
    let profile = state.profile.lock().unwrap().clone().unwrap_or_default();
    if profile != "clinique" {
        return Err(err(CoreError::Refused("restauration disponible dans le profil clinique".into())));
    }
    let ok = state.with(|s| s.verify_app_password(&password))?;
    if !ok {
        return Err(err(CoreError::BadSecret));
    }
    let key = state.keys.get(&AppState::account("clinique")).map_err(err)?.ok_or_else(|| err(CoreError::Keychain("clé absente".into())))?;
    let target = state.db_path("clinique");
    let (tmp, meta) = backup::prepare_restore(Path::new(&path), &passphrase, &target, &key).map_err(err)?;
    let mut g = state.store.lock().unwrap();
    let safety = match g.as_ref() {
        Some(s) => backup::safety_copy(s, &state.data_dir.join("clinique").join("avant-restauration")).map_err(err)?,
        None => return Err(err(CoreError::Refused("application verrouillée".into()))),
    };
    *g = None;
    backup::commit_restore(&tmp, &target).map_err(err)?;
    drop(g);
    // Vérification d'ouverture ; l'application reste verrouillée (le mot de passe peut avoir changé).
    Store::open(&target, &key, false, "Praticien").map_err(err)?;
    Ok(json!({"meta": meta, "safety_copy": safety.to_string_lossy()}))
}

/// Premier lancement sur un nouveau Mac : nouvelle clé dans le trousseau, restauration depuis la phrase.
#[tauri::command]
fn restore_first_run(state: State<AppState>, path: String, passphrase: String) -> R<Value> {
    let target = state.db_path("clinique");
    if target.exists() {
        return Err(err(CoreError::Refused("un profil clinique existe déjà sur ce Mac : restaurez depuis Sauvegarde".into())));
    }
    let key = generate_key().map_err(err)?;
    let (tmp, meta) = backup::prepare_restore(Path::new(&path), &passphrase, &target, &key).map_err(err)?;
    state.keys.set(&AppState::account("clinique"), &key).map_err(err)?;
    backup::commit_restore(&tmp, &target).map_err(err)?;
    Ok(serde_json::to_value(meta).unwrap())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir().expect("dossier de données");
            std::fs::create_dir_all(&dir)?;
            app.manage(AppState { store: Mutex::new(None), profile: Mutex::new(None), keys: keystore(&dir), data_dir: dir });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_status, setup_clinique, unlock, lock, reconnect, change_password, verify_password, get_catalog,
            create_dossier, new_encounter, load_encounter, update_identity, save_fields, set_sextant, set_exposure_group, update_exposure,
            apply_protocol, update_prevention_action, confirm_prevention, recap, validate_encounter, start_amendment, history, discard_empty_draft,
            list_encounters, patient_encounters, stats, home_summary,
            import_preview, import_stage, import_documents, import_records, import_decide, import_link, import_exclude, import_reinclude, import_commit,
            merge_patients, undo_merge, merges,
            projects, create_project, selection, set_eligibility, freeze_export, snapshots, write_snapshot,
            backup_create, backup_inspect, backup_restore, restore_first_run
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de l'application");
}
