//! Dossiers et consultations : création, rechargement fidèle, sauvegarde contrôlée, validation, amendements.

use super::{audit_on, new_id, now, Store};
use crate::domain::bewe::{self, SEXTANTS, UNASSESSABLE_REASONS};
use crate::domain::catalog::{self, exposure_categories, protocol_components, FORM_VERSION, PREVENTION_ACTIONS, PROTOCOL_VERSION};
use crate::domain::values::{self, FieldInput, StoredValue};
use crate::error::{CoreError, Result};
use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdentityInput {
    pub last_name: Option<String>,
    pub first_name: Option<String>,
    pub hospital_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterMeta {
    pub id: String,
    pub patient_id: String,
    pub patient_code: String,
    pub collection_mode: String,
    pub form_version: String,
    pub status: String,
    pub revision: i64,
    pub version: i64,
    pub visit_date: Option<String>,
    pub visit_date_precision: Option<String>,
    pub service_code: Option<String>,
    pub service_label_source: Option<String>,
    pub examiner: Option<String>,
    pub bewe_total_derived: Option<i64>,
    pub bewe_total_historical: Option<i64>,
    pub bewe_historical_band: Option<(i64, i64)>,
    pub bewe_legacy_raw: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub validated_at: Option<String>,
    pub is_demo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SextantRow {
    pub sextant: String,
    pub score: Option<i64>,
    pub missing_reason: Option<String>,
    pub unassessable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ExposureRow {
    pub grp: String,
    pub category: String,
    pub temporality: Option<String>,
    pub freq_min: Option<f64>,
    pub freq_max: Option<f64>,
    pub freq_unit: Option<String>,
    pub quantity_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PreventionRow {
    pub action_type: String,
    pub decision: Option<String>,
    pub proposed_by: Option<String>,
    pub confirmed: bool,
    pub st_already_used: bool,
    pub st_done_in_consultation: bool,
    pub st_given_today: bool,
    pub st_refused: bool,
    pub product_text: Option<String>,
    pub route: Option<String>,
    pub frequency_text: Option<String>,
    pub tray_minutes: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagRow {
    pub id: i64,
    pub flag: String,
    pub label: String,
    pub detail: Option<String>,
    pub status: String,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterFull {
    pub meta: EncounterMeta,
    pub identity: Option<IdentityInput>,
    pub values: Vec<StoredValue>,
    pub bewe: Vec<SextantRow>,
    pub exposures: Vec<ExposureRow>,
    pub prevention: Vec<PreventionRow>,
    pub flags: Vec<FlagRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResult {
    pub version: i64,
    pub saved_at: String,
    pub bewe_total_derived: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SextantInput {
    pub sextant: String,
    #[serde(default)]
    pub score: Option<Value>,
    #[serde(default)]
    pub missing_reason: Option<String>,
    #[serde(default)]
    pub unassessable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExposurePatch {
    pub temporality: Option<String>,
    pub freq_min: Option<Value>,
    pub freq_max: Option<Value>,
    pub freq_unit: Option<String>,
    pub quantity_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreventionPatch {
    pub decision: Option<String>,
    pub st_already_used: Option<bool>,
    pub st_done_in_consultation: Option<bool>,
    pub st_given_today: Option<bool>,
    pub st_refused: Option<bool>,
    pub product_text: Option<String>,
    pub route: Option<String>,
    pub frequency_text: Option<String>,
    pub tray_minutes: Option<Value>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolChange {
    pub save: SaveResult,
    pub proposed: Vec<String>,
    pub removed_unconfirmed: Vec<String>,
    pub kept_confirmed_outside: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recap {
    pub bewe_filled: usize,
    pub bewe_total: Option<i64>,
    pub bewe_category: Option<String>,
    pub missing_essentials: Vec<String>,
    pub unconfirmed_prevention: Vec<String>,
    pub hbd: Option<String>,
    pub protocol: Option<String>,
    pub open_flags: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionRow {
    pub revision: i64,
    pub reason: Option<String>,
    pub created_at: String,
    pub author: Option<String>,
    pub snapshot: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRow {
    pub at: String,
    pub kind: String,
    pub field: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub reason: Option<String>,
    pub author: Option<String>,
}

fn opt_text(s: Option<String>) -> Option<String> {
    s.filter(|t| !t.trim().is_empty())
}

fn stored_from_row(r: &Row) -> rusqlite::Result<StoredValue> {
    Ok(StoredValue {
        field: r.get("field")?,
        value_text: r.get("value_text")?,
        value_num: r.get("value_num")?,
        value_num_max: r.get("value_num_max")?,
        precision: r.get("precision")?,
        date_precision: r.get("date_precision")?,
        missing_reason: r.get("missing_reason")?,
        source_type: r.get("source_type")?,
        certainty: r.get("certainty")?,
    })
}

pub(crate) fn load_values(conn: &Connection, encounter_id: &str) -> Result<Vec<StoredValue>> {
    let mut st = conn.prepare("SELECT * FROM field_value WHERE encounter_id = ?1 ORDER BY field")?;
    let rows = st.query_map([encounter_id], stored_from_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub(crate) fn load_bewe(conn: &Connection, encounter_id: &str) -> Result<Vec<SextantRow>> {
    let mut st = conn.prepare("SELECT sextant, score, missing_reason, unassessable_reason FROM bewe_sextant WHERE encounter_id = ?1")?;
    let mut rows = st
        .query_map([encounter_id], |r| {
            Ok(SextantRow { sextant: r.get(0)?, score: r.get(1)?, missing_reason: r.get(2)?, unassessable_reason: r.get(3)? })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let order = |k: &str| SEXTANTS.iter().position(|(key, _, _)| *key == k).unwrap_or(9);
    rows.sort_by_key(|r| order(&r.sextant));
    Ok(rows)
}

pub(crate) fn load_exposures(conn: &Connection, encounter_id: &str) -> Result<Vec<ExposureRow>> {
    let mut st = conn.prepare("SELECT grp, category, temporality, freq_min, freq_max, freq_unit, quantity_text FROM exposure WHERE encounter_id = ?1")?;
    let mut rows = st
        .query_map([encounter_id], |r| {
            Ok(ExposureRow {
                grp: r.get(0)?,
                category: r.get(1)?,
                temporality: r.get(2)?,
                freq_min: r.get(3)?,
                freq_max: r.get(4)?,
                freq_unit: r.get(5)?,
                quantity_text: r.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let order = |g: &str, c: &str| exposure_categories(g).and_then(|l| l.iter().position(|(k, _)| *k == c)).unwrap_or(99);
    rows.sort_by_key(|r| (r.grp.clone(), order(&r.grp, &r.category)));
    Ok(rows)
}

pub(crate) fn load_prevention(conn: &Connection, encounter_id: &str) -> Result<Vec<PreventionRow>> {
    let mut st = conn.prepare("SELECT * FROM prevention_action WHERE encounter_id = ?1")?;
    let mut rows = st
        .query_map([encounter_id], |r| {
            Ok(PreventionRow {
                action_type: r.get("action_type")?,
                decision: r.get("decision")?,
                proposed_by: r.get("proposed_by")?,
                confirmed: r.get::<_, i64>("confirmed")? == 1,
                st_already_used: r.get::<_, i64>("st_already_used")? == 1,
                st_done_in_consultation: r.get::<_, i64>("st_done_in_consultation")? == 1,
                st_given_today: r.get::<_, i64>("st_given_today")? == 1,
                st_refused: r.get::<_, i64>("st_refused")? == 1,
                product_text: r.get("product_text")?,
                route: r.get("route")?,
                frequency_text: r.get("frequency_text")?,
                tray_minutes: r.get("tray_minutes")?,
                note: r.get("note")?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let order = |k: &str| PREVENTION_ACTIONS.iter().position(|(a, _)| *a == k).unwrap_or(9);
    rows.sort_by_key(|r| order(&r.action_type));
    Ok(rows)
}

pub(crate) fn load_flags(conn: &Connection, encounter_id: &str) -> Result<Vec<FlagRow>> {
    let mut st = conn.prepare("SELECT id, flag, detail, status, resolution FROM encounter_flag WHERE encounter_id = ?1 ORDER BY id")?;
    let rows = st
        .query_map([encounter_id], |r| {
            let flag: String = r.get(1)?;
            Ok(FlagRow { id: r.get(0)?, label: crate::domain::legacy::flag_label(&flag).to_string(), flag, detail: r.get(2)?, status: r.get(3)?, resolution: r.get(4)? })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub(crate) fn load_meta(conn: &Connection, encounter_id: &str) -> Result<EncounterMeta> {
    conn.query_row(
        "SELECT e.*, p.code AS patient_code, p.is_demo FROM encounter e JOIN patient p ON p.id = e.patient_id WHERE e.id = ?1",
        [encounter_id],
        |r| {
            let bmin: Option<i64> = r.get("bewe_historical_band_min")?;
            let bmax: Option<i64> = r.get("bewe_historical_band_max")?;
            Ok(EncounterMeta {
                id: r.get("id")?,
                patient_id: r.get("patient_id")?,
                patient_code: r.get("patient_code")?,
                collection_mode: r.get("collection_mode")?,
                form_version: r.get("form_version")?,
                status: r.get("status")?,
                revision: r.get("revision")?,
                version: r.get("version")?,
                visit_date: r.get("visit_date")?,
                visit_date_precision: r.get("visit_date_precision")?,
                service_code: r.get("service_code")?,
                service_label_source: r.get("service_label_source")?,
                examiner: r.get("examiner")?,
                bewe_total_derived: r.get("bewe_total_derived")?,
                bewe_total_historical: r.get("bewe_total_historical")?,
                bewe_historical_band: bmin.zip(bmax),
                bewe_legacy_raw: r.get("bewe_legacy_raw")?,
                created_at: r.get("created_at")?,
                updated_at: r.get("updated_at")?,
                validated_at: r.get("validated_at")?,
                is_demo: r.get::<_, i64>("is_demo")? == 1,
            })
        },
    )
    .optional()?
    .ok_or_else(|| CoreError::NotFound("consultation".into()))
}

pub(crate) fn load_identity(conn: &Connection, patient_id: &str) -> Result<Option<IdentityInput>> {
    Ok(conn
        .query_row("SELECT last_name, first_name, hospital_id FROM patient_identity WHERE patient_id = ?1", [patient_id], |r| {
            Ok(IdentityInput { last_name: r.get(0)?, first_name: r.get(1)?, hospital_id: r.get(2)? })
        })
        .optional()?)
}

pub(crate) fn load_full(conn: &Connection, encounter_id: &str, with_identity: bool) -> Result<EncounterFull> {
    let meta = load_meta(conn, encounter_id)?;
    let identity = if with_identity { load_identity(conn, &meta.patient_id)? } else { None };
    Ok(EncounterFull {
        values: load_values(conn, encounter_id)?,
        bewe: load_bewe(conn, encounter_id)?,
        exposures: load_exposures(conn, encounter_id)?,
        prevention: load_prevention(conn, encounter_id)?,
        flags: load_flags(conn, encounter_id)?,
        identity,
        meta,
    })
}

/// Total dérivé recalculé côté domaine, jamais saisi.
pub(crate) fn recompute_bewe(conn: &Connection, encounter_id: &str) -> Result<Option<i64>> {
    let rows = load_bewe(conn, encounter_id)?;
    let mut scores: [Option<u8>; 6] = [None; 6];
    for (i, (key, _, _)) in SEXTANTS.iter().enumerate() {
        scores[i] = rows.iter().find(|r| r.sextant == *key).and_then(|r| r.score).map(|s| s as u8);
    }
    let total = bewe::derived_total(&scores).map(|t| t as i64);
    conn.execute("UPDATE encounter SET bewe_total_derived = ?1 WHERE id = ?2", params![total, encounter_id])?;
    Ok(total)
}

fn value_json(v: &Option<StoredValue>) -> Option<String> {
    v.as_ref().map(|x| serde_json::to_string(x).unwrap_or_default())
}

pub(crate) fn write_value(tx: &Connection, encounter_id: &str, author: &str, sv: &StoredValue, source_ref: Option<&str>) -> Result<()> {
    tx.execute(
        "INSERT INTO field_value(encounter_id, field, value_text, value_num, value_num_max, precision, date_precision, missing_reason, source_type, source_ref, certainty, recorded_at, author)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
         ON CONFLICT(encounter_id, field) DO UPDATE SET value_text=excluded.value_text, value_num=excluded.value_num, value_num_max=excluded.value_num_max,
           precision=excluded.precision, date_precision=excluded.date_precision, missing_reason=excluded.missing_reason, source_type=excluded.source_type,
           source_ref=excluded.source_ref, certainty=excluded.certainty, recorded_at=excluded.recorded_at, author=excluded.author",
        params![
            encounter_id,
            sv.field,
            sv.value_text,
            sv.value_num,
            sv.value_num_max,
            sv.precision,
            sv.date_precision,
            sv.missing_reason,
            sv.source_type,
            source_ref,
            sv.certainty,
            now(),
            author
        ],
    )?;
    Ok(())
}

fn current_value(conn: &Connection, encounter_id: &str, field: &str) -> Result<Option<StoredValue>> {
    Ok(conn.query_row("SELECT * FROM field_value WHERE encounter_id = ?1 AND field = ?2", params![encounter_id, field], stored_from_row).optional()?)
}

fn sync_denormalized(conn: &Connection, encounter_id: &str) -> Result<()> {
    let date = current_value(conn, encounter_id, "visit_date")?;
    let service = current_value(conn, encounter_id, "service_code")?;
    conn.execute(
        "UPDATE encounter SET visit_date = ?1, visit_date_precision = ?2, service_code = ?3 WHERE id = ?4",
        params![
            date.as_ref().and_then(|d| d.value_text.clone()),
            date.as_ref().and_then(|d| d.date_precision.clone()),
            service.as_ref().and_then(|s| s.value_text.clone()),
            encounter_id
        ],
    )?;
    Ok(())
}

fn text_or_none(v: &Option<String>) -> Option<String> {
    v.as_ref().filter(|t| !t.trim().is_empty()).cloned()
}

impl Store {
    fn next_patient_code(&self, demo: bool) -> Result<String> {
        let prefix = if demo { "DEMO".to_string() } else { format!("C{}", super::today().format("%Y")) };
        let n: i64 = self.conn.query_row("SELECT count(*) FROM patient WHERE code LIKE ?1 || '-%'", [&prefix], |r| r.get(0))?;
        let mut i = n + 1;
        loop {
            let code = format!("{prefix}-{i:03}");
            let exists: bool = self.conn.query_row("SELECT count(*) FROM patient WHERE code = ?1", [&code], |r| r.get::<_, i64>(0))? > 0;
            if !exists {
                return Ok(code);
            }
            i += 1;
        }
    }

    /// Nouveau dossier + première consultation (brouillon vierge : aucune réponse clinique).
    pub fn create_dossier(&mut self, code: Option<String>, identity: Option<IdentityInput>, demo: bool) -> Result<EncounterFull> {
        let code = match text_or_none(&code) {
            Some(c) => {
                let c = c.trim().to_string();
                if c.len() > 40 {
                    return Err(CoreError::validation("code", "40 caractères au plus"));
                }
                c
            }
            None => self.next_patient_code(demo)?,
        };
        let author = self.author.clone();
        let tx = self.conn.transaction()?;
        let exists: i64 = tx.query_row("SELECT count(*) FROM patient WHERE code = ?1", [&code], |r| r.get(0))?;
        if exists > 0 {
            return Err(CoreError::validation("code", "ce code de dossier existe déjà"));
        }
        let pid = new_id();
        tx.execute("INSERT INTO patient(id, code, created_at, is_demo) VALUES (?1,?2,?3,?4)", params![pid, code, now(), demo as i64])?;
        if let Some(idn) = identity {
            write_identity(&tx, &pid, &idn)?;
        }
        let eid = insert_encounter(&tx, &pid, "structured_prospective", &author)?;
        propose_today(&tx, &eid, &author)?;
        audit_on(&tx, &author, "create", "patient", Some(&pid), None, None, Some(&code), None)?;
        tx.commit()?;
        self.load_encounter(&eid, true)
    }

    pub fn new_encounter_for_patient(&mut self, patient_id: &str) -> Result<EncounterFull> {
        let author = self.author.clone();
        let tx = self.conn.transaction()?;
        let ok: i64 = tx.query_row("SELECT count(*) FROM patient WHERE id = ?1 AND merged_into IS NULL", [patient_id], |r| r.get(0))?;
        if ok == 0 {
            return Err(CoreError::NotFound("dossier".into()));
        }
        let eid = insert_encounter(&tx, patient_id, "structured_prospective", &author)?;
        propose_today(&tx, &eid, &author)?;
        tx.commit()?;
        self.load_encounter(&eid, true)
    }

    pub fn update_identity(&mut self, patient_id: &str, identity: IdentityInput) -> Result<()> {
        let tx = self.conn.transaction()?;
        write_identity(&tx, patient_id, &identity)?;
        audit_on(&tx, &self.author, "update_identity", "patient", Some(patient_id), None, None, None, None)?;
        tx.commit()?;
        Ok(())
    }

    pub fn load_encounter(&self, id: &str, with_identity: bool) -> Result<EncounterFull> {
        load_full(&self.conn, id, with_identity)
    }

    /// Enveloppe commune à toute modification : statut modifiable, version attendue, horodatage.
    fn edit<T>(&mut self, encounter_id: &str, expected_version: i64, f: impl FnOnce(&Transaction, &str, &str) -> Result<T>) -> Result<(T, SaveResult)> {
        let author = self.author.clone();
        let tx = self.conn.transaction()?;
        let (status, version): (String, i64) = tx
            .query_row("SELECT status, version FROM encounter WHERE id = ?1", [encounter_id], |r| Ok((r.get(0)?, r.get(1)?)))
            .optional()?
            .ok_or_else(|| CoreError::NotFound("consultation".into()))?;
        if status == "validated" {
            return Err(CoreError::Locked);
        }
        if version != expected_version {
            return Err(CoreError::Conflict { expected: expected_version, actual: version });
        }
        let reason: Option<String> = if status == "amending" {
            tx.query_row("SELECT reason FROM audit_event WHERE entity_id = ?1 AND kind = 'start_amendment' ORDER BY id DESC LIMIT 1", [encounter_id], |r| r.get(0)).optional()?.flatten()
        } else {
            None
        };
        let out = f(&tx, &author, reason.as_deref().unwrap_or(""))?;
        let saved_at = now();
        tx.execute("UPDATE encounter SET version = version + 1, updated_at = ?1 WHERE id = ?2", params![saved_at, encounter_id])?;
        let (version, derived): (i64, Option<i64>) = tx.query_row("SELECT version, bewe_total_derived FROM encounter WHERE id = ?1", [encounter_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
        tx.commit()?;
        Ok((out, SaveResult { version, saved_at, bewe_total_derived: derived }))
    }

    pub fn save_fields(&mut self, encounter_id: &str, expected_version: i64, inputs: Vec<FieldInput>) -> Result<SaveResult> {
        // Validation complète avant toute écriture : un lot invalide n'écrit rien.
        let mut validated = Vec::with_capacity(inputs.len());
        for input in &inputs {
            let f = catalog::field(&input.field).ok_or_else(|| CoreError::validation(&input.field, "champ inconnu"))?;
            if f.legacy && input.source_type.as_deref() != Some("legacy_import") && input.source_type.as_deref() != Some("clinician_adjudication") {
                // Les champs historiques ne se saisissent pas dans le formulaire prospectif.
                let mode: String = self.conn.query_row("SELECT collection_mode FROM encounter WHERE id = ?1", [encounter_id], |r| r.get(0))?;
                if mode != "legacy_retrospective" {
                    return Err(CoreError::validation(f.code, "champ réservé aux données historiques"));
                }
            }
            if f.custom && matches!(f.code, "acidic_drinks_status" | "acidic_foods_status" | "prevention_protocol") {
                return Err(CoreError::validation(f.code, "ce champ se modifie par son composant dédié"));
            }
            validated.push((input.field.clone(), values::validate(input)?));
        }
        let (_, res) = self.edit(encounter_id, expected_version, |tx, author, reason| {
            for (field, sv) in &validated {
                let old = current_value(tx, encounter_id, field)?;
                match sv {
                    Some(sv) => write_value(tx, encounter_id, author, sv, None)?,
                    None => {
                        tx.execute("DELETE FROM field_value WHERE encounter_id = ?1 AND field = ?2", params![encounter_id, field])?;
                    }
                }
                if value_json(&old) != value_json(sv) {
                    audit_on(tx, author, "field", "encounter", Some(encounter_id), Some(field), value_json(&old).as_deref(), value_json(sv).as_deref(), (!reason.is_empty()).then_some(reason))?;
                }
            }
            sync_denormalized(tx, encounter_id)?;
            Ok(())
        })?;
        Ok(res)
    }

    pub fn set_sextant(&mut self, encounter_id: &str, expected_version: i64, input: SextantInput) -> Result<SaveResult> {
        if !bewe::is_sextant_key(&input.sextant) {
            return Err(CoreError::validation("bewe", "sextant inconnu"));
        }
        let score = match input.score.as_ref().filter(|v| !v.is_null()) {
            Some(v) => Some(bewe::validate_score(v).ok_or_else(|| CoreError::validation("bewe", "score admis : 0, 1, 2 ou 3"))?),
            None => None,
        };
        let missing = input.missing_reason.clone().filter(|m| !m.is_empty());
        if score.is_some() && missing.is_some() {
            return Err(CoreError::validation("bewe", "un score et une raison de manque ne peuvent coexister"));
        }
        if let Some(m) = &missing {
            if m != "not_assessable" && m != "unknown" {
                return Err(CoreError::validation("bewe", "raison de manque non admise"));
            }
        }
        let ureason = input.unassessable_reason.clone().filter(|r| !r.is_empty());
        if let Some(r) = &ureason {
            if missing.as_deref() != Some("not_assessable") || !UNASSESSABLE_REASONS.iter().any(|(k, _)| k == r) {
                return Err(CoreError::validation("bewe", "motif de non-évaluation invalide"));
            }
        }
        let (_, res) = self.edit(encounter_id, expected_version, |tx, author, reason| {
            let old: Option<String> = tx
                .query_row("SELECT coalesce(score, missing_reason) FROM bewe_sextant WHERE encounter_id = ?1 AND sextant = ?2", params![encounter_id, input.sextant], |r| {
                    r.get::<_, rusqlite::types::Value>(0).map(|v| match v {
                        rusqlite::types::Value::Integer(i) => i.to_string(),
                        rusqlite::types::Value::Text(t) => t,
                        _ => String::new(),
                    })
                })
                .optional()?;
            if score.is_none() && missing.is_none() {
                tx.execute("DELETE FROM bewe_sextant WHERE encounter_id = ?1 AND sextant = ?2", params![encounter_id, input.sextant])?;
            } else {
                tx.execute(
                    "INSERT INTO bewe_sextant(encounter_id, sextant, score, missing_reason, unassessable_reason, recorded_at) VALUES (?1,?2,?3,?4,?5,?6)
                     ON CONFLICT(encounter_id, sextant) DO UPDATE SET score=excluded.score, missing_reason=excluded.missing_reason, unassessable_reason=excluded.unassessable_reason, recorded_at=excluded.recorded_at",
                    params![encounter_id, input.sextant, score.map(|s| s as i64), missing, ureason, now()],
                )?;
            }
            let new = score.map(|s| s.to_string()).or(missing.clone());
            audit_on(tx, author, "bewe", "encounter", Some(encounter_id), Some(&input.sextant), old.as_deref(), new.as_deref(), (!reason.is_empty()).then_some(reason))?;
            recompute_bewe(tx, encounter_id)?;
            Ok(())
        })?;
        Ok(res)
    }

    /// Sélection multiple boissons/aliments. « Aucune rapportée » est exclusive ; vide = non renseigné.
    pub fn set_exposure_group(&mut self, encounter_id: &str, expected_version: i64, group: &str, none_reported: bool, categories: Vec<String>) -> Result<SaveResult> {
        let cats = exposure_categories(group).ok_or_else(|| CoreError::validation("exposure", "groupe inconnu"))?;
        for c in &categories {
            if !cats.iter().any(|(k, _)| k == c) {
                return Err(CoreError::validation("exposure", "catégorie inconnue"));
            }
        }
        if none_reported && !categories.is_empty() {
            return Err(CoreError::validation("exposure", "« aucune rapportée » exclut les catégories positives"));
        }
        let status_field = if group == "drink" { "acidic_drinks_status" } else { "acidic_foods_status" };
        let (_, res) = self.edit(encounter_id, expected_version, |tx, author, reason| {
            let before: Vec<String> = load_exposures(tx, encounter_id)?.into_iter().filter(|e| e.grp == group).map(|e| e.category).collect();
            for c in &before {
                if !categories.contains(c) {
                    tx.execute("DELETE FROM exposure WHERE encounter_id = ?1 AND grp = ?2 AND category = ?3", params![encounter_id, group, c])?;
                }
            }
            for c in &categories {
                tx.execute("INSERT OR IGNORE INTO exposure(encounter_id, grp, category, recorded_at) VALUES (?1,?2,?3,?4)", params![encounter_id, group, c, now()])?;
            }
            let status = if none_reported {
                Some("none_reported")
            } else if !categories.is_empty() {
                Some("reported")
            } else {
                None
            };
            let old = current_value(tx, encounter_id, status_field)?;
            match status {
                Some(s) => write_value(tx, encounter_id, author, &StoredValue { field: status_field.into(), value_text: Some(s.into()), ..Default::default() }, None)?,
                None => {
                    tx.execute("DELETE FROM field_value WHERE encounter_id = ?1 AND field = ?2", params![encounter_id, status_field])?;
                }
            }
            audit_on(tx, author, "exposure_group", "encounter", Some(encounter_id), Some(group), Some(&format!("{:?}{:?}", old.and_then(|o| o.value_text), before)), Some(&format!("{status:?}{categories:?}")), (!reason.is_empty()).then_some(reason))?;
            Ok(())
        })?;
        Ok(res)
    }

    pub fn update_exposure(&mut self, encounter_id: &str, expected_version: i64, group: &str, category: &str, patch: ExposurePatch) -> Result<SaveResult> {
        let num = |v: &Option<Value>, name: &str| -> Result<Option<f64>> {
            match v.as_ref().filter(|x| !x.is_null() && x.as_str() != Some("")) {
                None => Ok(None),
                Some(x) => {
                    let n = values::parse_number(x).ok_or_else(|| CoreError::validation(name, "nombre attendu"))?;
                    if n < 0.0 {
                        return Err(CoreError::validation(name, "nombre positif attendu"));
                    }
                    Ok(Some(n))
                }
            }
        };
        let fmin = num(&patch.freq_min, "fréquence")?;
        let fmax = num(&patch.freq_max, "fréquence")?.filter(|m| Some(*m) != fmin);
        if fmax.is_some() && fmin.is_none() {
            return Err(CoreError::validation("fréquence", "borne basse manquante"));
        }
        if let (Some(a), Some(b)) = (fmin, fmax) {
            if b < a {
                return Err(CoreError::validation("fréquence", "borne haute inférieure à la borne basse"));
            }
        }
        let unit = text_or_none(&patch.freq_unit);
        if let Some(u) = &unit {
            if !["day", "week", "month"].contains(&u.as_str()) {
                return Err(CoreError::validation("fréquence", "unité inconnue"));
            }
        }
        let temp = text_or_none(&patch.temporality);
        if let Some(t) = &temp {
            if t != "current" && t != "past" {
                return Err(CoreError::validation("temporalité", "valeur inconnue"));
            }
        }
        let (_, res) = self.edit(encounter_id, expected_version, |tx, author, reason| {
            let n = tx.execute(
                "UPDATE exposure SET temporality=?1, freq_min=?2, freq_max=?3, freq_unit=?4, quantity_text=?5, recorded_at=?6 WHERE encounter_id=?7 AND grp=?8 AND category=?9",
                params![temp, fmin, fmax, unit, opt_text(patch.quantity_text.clone()), now(), encounter_id, group, category],
            )?;
            if n == 0 {
                return Err(CoreError::NotFound("exposition".into()));
            }
            audit_on(tx, author, "exposure_detail", "encounter", Some(encounter_id), Some(&format!("{group}.{category}")), None, Some(&serde_json::to_string(&patch).unwrap_or_default()), (!reason.is_empty()).then_some(reason))?;
            Ok(())
        })?;
        Ok(res)
    }

    /// Choix du protocole : prépare des propositions à confirmer ; ne détruit jamais une mesure confirmée.
    pub fn apply_protocol(&mut self, encounter_id: &str, expected_version: i64, protocol: Option<String>) -> Result<ProtocolChange> {
        let protocol = protocol.filter(|p| !p.is_empty());
        if let Some(p) = &protocol {
            if !["none", "moderate", "advanced", "custom"].contains(&p.as_str()) {
                return Err(CoreError::validation("prevention_protocol", "protocole inconnu"));
            }
        }
        let comps: Vec<&str> = protocol.as_deref().map(protocol_components).unwrap_or(&[]).to_vec();
        let (change, save) = self.edit(encounter_id, expected_version, |tx, author, reason| {
            let old = current_value(tx, encounter_id, "prevention_protocol")?;
            match &protocol {
                Some(p) => write_value(tx, encounter_id, author, &StoredValue { field: "prevention_protocol".into(), value_text: Some(p.clone()), ..Default::default() }, None)?,
                None => {
                    tx.execute("DELETE FROM field_value WHERE encounter_id = ?1 AND field = 'prevention_protocol'", [encounter_id])?;
                }
            }
            let rows = load_prevention(tx, encounter_id)?;
            let mut removed = vec![];
            let mut kept = vec![];
            for r in &rows {
                if r.action_type == "vomiting_semirigid_tray" || r.action_type == "other" {
                    continue;
                }
                let in_new = comps.contains(&r.action_type.as_str());
                if !in_new {
                    if r.confirmed {
                        kept.push(r.action_type.clone());
                    } else {
                        tx.execute("DELETE FROM prevention_action WHERE encounter_id = ?1 AND action_type = ?2", params![encounter_id, r.action_type])?;
                        removed.push(r.action_type.clone());
                    }
                }
            }
            let mut proposed = vec![];
            for c in &comps {
                if rows.iter().any(|r| r.action_type == *c) {
                    continue;
                }
                let freq = if *c == "tooth_mousse" { Some("quotidien") } else { None };
                tx.execute(
                    "INSERT INTO prevention_action(encounter_id, action_type, decision, proposed_by, confirmed, frequency_text, protocol_version, recorded_at) VALUES (?1,?2,'advised',?3,0,?4,?5,?6)",
                    params![encounter_id, c, protocol, freq, PROTOCOL_VERSION, now()],
                )?;
                proposed.push(c.to_string());
            }
            audit_on(tx, author, "protocol", "encounter", Some(encounter_id), Some("prevention_protocol"), old.and_then(|o| o.value_text).as_deref(), protocol.as_deref(), (!reason.is_empty()).then_some(reason))?;
            Ok((proposed, removed, kept))
        })?;
        Ok(ProtocolChange { save, proposed: change.0, removed_unconfirmed: change.1, kept_confirmed_outside: change.2 })
    }

    pub fn update_prevention_action(&mut self, encounter_id: &str, expected_version: i64, action_type: &str, patch: PreventionPatch) -> Result<SaveResult> {
        if !PREVENTION_ACTIONS.iter().any(|(k, _)| *k == action_type) {
            return Err(CoreError::validation("prevention", "action inconnue"));
        }
        if let Some(d) = &patch.decision {
            if !d.is_empty() && !["advised", "not_advised", "not_applicable"].contains(&d.as_str()) {
                return Err(CoreError::validation("prevention", "décision inconnue"));
            }
        }
        if let Some(r) = &patch.route {
            if !r.is_empty() && !["direct", "tray", "both", "unspecified"].contains(&r.as_str()) {
                return Err(CoreError::validation("prevention", "mode d'application inconnu"));
            }
        }
        let tray = match patch.tray_minutes.as_ref().filter(|v| !v.is_null() && v.as_str() != Some("")) {
            Some(v) => {
                let n = values::parse_number(v).ok_or_else(|| CoreError::validation("tray_minutes", "nombre attendu"))?;
                if n.fract() != 0.0 || !(1.0..=240.0).contains(&n) {
                    return Err(CoreError::validation("tray_minutes", "durée entre 1 et 240 minutes"));
                }
                Some(n as i64)
            }
            None => None,
        };
        let (_, res) = self.edit(encounter_id, expected_version, |tx, author, reason| {
            let exists: Option<PreventionRow> = load_prevention(tx, encounter_id)?.into_iter().find(|r| r.action_type == action_type);
            let mut row = exists.clone().unwrap_or(PreventionRow { action_type: action_type.to_string(), confirmed: true, ..Default::default() });
            if let Some(d) = &patch.decision {
                row.decision = opt_text(Some(d.clone()));
            }
            if let Some(b) = patch.st_already_used {
                row.st_already_used = b;
            }
            if let Some(b) = patch.st_done_in_consultation {
                row.st_done_in_consultation = b;
            }
            if let Some(b) = patch.st_given_today {
                row.st_given_today = b;
            }
            if let Some(b) = patch.st_refused {
                row.st_refused = b;
            }
            if patch.product_text.is_some() {
                row.product_text = opt_text(patch.product_text.clone());
            }
            if let Some(r) = &patch.route {
                row.route = opt_text(Some(r.clone()));
            }
            if patch.frequency_text.is_some() {
                row.frequency_text = opt_text(patch.frequency_text.clone());
            }
            if patch.tray_minutes.is_some() {
                row.tray_minutes = tray;
            }
            if patch.note.is_some() {
                row.note = opt_text(patch.note.clone());
            }
            // Voie directe : aucune durée de gouttière ne subsiste.
            if !matches!(row.route.as_deref(), Some("tray") | Some("both")) {
                row.tray_minutes = None;
            }
            if row.action_type != "tooth_mousse" {
                row.route = None;
                row.tray_minutes = None;
            }
            let empty = row.decision.is_none() && !row.st_already_used && !row.st_done_in_consultation && !row.st_given_today && !row.st_refused && row.product_text.is_none() && row.note.is_none();
            if empty && row.proposed_by.is_none() {
                tx.execute("DELETE FROM prevention_action WHERE encounter_id = ?1 AND action_type = ?2", params![encounter_id, action_type])?;
            } else {
                tx.execute(
                    "INSERT INTO prevention_action(encounter_id, action_type, decision, proposed_by, confirmed, st_already_used, st_done_in_consultation, st_given_today, st_refused, product_text, route, frequency_text, tray_minutes, note, protocol_version, recorded_at)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)
                     ON CONFLICT(encounter_id, action_type) DO UPDATE SET decision=excluded.decision, confirmed=excluded.confirmed, st_already_used=excluded.st_already_used,
                       st_done_in_consultation=excluded.st_done_in_consultation, st_given_today=excluded.st_given_today, st_refused=excluded.st_refused, product_text=excluded.product_text,
                       route=excluded.route, frequency_text=excluded.frequency_text, tray_minutes=excluded.tray_minutes, note=excluded.note, recorded_at=excluded.recorded_at",
                    params![
                        encounter_id,
                        action_type,
                        row.decision,
                        row.proposed_by,
                        row.confirmed as i64,
                        row.st_already_used as i64,
                        row.st_done_in_consultation as i64,
                        row.st_given_today as i64,
                        row.st_refused as i64,
                        row.product_text,
                        row.route,
                        row.frequency_text,
                        row.tray_minutes,
                        row.note,
                        PROTOCOL_VERSION,
                        now()
                    ],
                )?;
            }
            audit_on(tx, author, "prevention_action", "encounter", Some(encounter_id), Some(action_type), exists.map(|e| serde_json::to_string(&e).unwrap_or_default()).as_deref(), Some(&serde_json::to_string(&row).unwrap_or_default()), (!reason.is_empty()).then_some(reason))?;
            Ok(())
        })?;
        Ok(res)
    }

    /// « Confirmer les mesures conseillées » : seules les propositions confirmées comptent comme conseillées.
    pub fn confirm_prevention(&mut self, encounter_id: &str, expected_version: i64) -> Result<SaveResult> {
        let (_, res) = self.edit(encounter_id, expected_version, |tx, author, reason| {
            let n = tx.execute("UPDATE prevention_action SET confirmed = 1, recorded_at = ?1 WHERE encounter_id = ?2 AND confirmed = 0 AND decision IS NOT NULL", params![now(), encounter_id])?;
            audit_on(tx, author, "confirm_prevention", "encounter", Some(encounter_id), None, None, Some(&n.to_string()), (!reason.is_empty()).then_some(reason))?;
            Ok(())
        })?;
        Ok(res)
    }

    pub fn recap(&self, encounter_id: &str) -> Result<Recap> {
        let full = self.load_encounter(encounter_id, false)?;
        let has = |f: &str| full.values.iter().any(|v| v.field == f);
        let legacy = full.meta.collection_mode == "legacy_retrospective";
        let mut missing = vec![];
        for f in catalog::catalog().fields.iter().filter(|f| f.essential && !f.legacy) {
            if legacy {
                break;
            }
            if let Some(si) = &f.show_if {
                let dep = full.values.iter().find(|v| v.field == si.field).and_then(|v| v.value_text.clone());
                if !dep.map(|d| si.values.contains(&d.as_str())).unwrap_or(false) {
                    continue;
                }
            }
            if !has(f.code) {
                missing.push(f.label.to_string());
            }
        }
        let filled = full.bewe.len();
        if !legacy && filled < 6 {
            missing.push(format!("BEWE ({filled}/6 sextants)"));
        }
        if !legacy {
            if !has("acidic_drinks_status") {
                missing.push("Boissons acides".into());
            }
            if !has("acidic_foods_status") {
                missing.push("Aliments acides".into());
            }
        }
        let val = |f: &str| full.values.iter().find(|v| v.field == f).and_then(|v| v.value_text.clone());
        Ok(Recap {
            bewe_filled: filled,
            bewe_total: full.meta.bewe_total_derived,
            bewe_category: full.meta.bewe_total_derived.and_then(|t| bewe::category(t as u8)).map(String::from),
            missing_essentials: missing,
            unconfirmed_prevention: full.prevention.iter().filter(|p| !p.confirmed && p.decision.is_some()).map(|p| p.action_type.clone()).collect(),
            hbd: val("hbd_teaching"),
            protocol: val("prevention_protocol"),
            open_flags: full.flags.iter().filter(|f| f.status == "open").count(),
        })
    }

    /// « Terminer la saisie » : validation, verrouillage, instantané. Aucun document n'est produit.
    pub fn validate_encounter(&mut self, encounter_id: &str, expected_version: i64) -> Result<EncounterFull> {
        let author = self.author.clone();
        let tx = self.conn.transaction()?;
        let (status, version, revision): (String, i64, i64) = tx.query_row("SELECT status, version, revision FROM encounter WHERE id = ?1", [encounter_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
        if status == "validated" {
            return Err(CoreError::Refused("consultation déjà validée".into()));
        }
        if version != expected_version {
            return Err(CoreError::Conflict { expected: expected_version, actual: version });
        }
        let reason: Option<String> = if status == "amending" {
            tx.query_row("SELECT reason FROM audit_event WHERE entity_id = ?1 AND kind = 'start_amendment' ORDER BY id DESC LIMIT 1", [encounter_id], |r| r.get(0)).optional()?.flatten()
        } else {
            None
        };
        let at = now();
        tx.execute("UPDATE encounter SET status = 'validated', revision = revision + 1, version = version + 1, validated_at = ?1, updated_at = ?1 WHERE id = ?2", params![at, encounter_id])?;
        let snap = load_full(&tx, encounter_id, false)?;
        tx.execute(
            "INSERT INTO encounter_revision(encounter_id, revision, snapshot, reason, created_at, author) VALUES (?1,?2,?3,?4,?5,?6)",
            params![encounter_id, revision + 1, serde_json::to_string(&snap)?, reason, at, author],
        )?;
        audit_on(&tx, &author, "validate", "encounter", Some(encounter_id), None, None, Some(&(revision + 1).to_string()), reason.as_deref())?;
        tx.commit()?;
        self.load_encounter(encounter_id, true)
    }

    /// « Corriger » une consultation validée : motif obligatoire, l'ancienne révision reste consultable.
    pub fn start_amendment(&mut self, encounter_id: &str, expected_version: i64, reason: &str) -> Result<EncounterFull> {
        if reason.trim().len() < 3 {
            return Err(CoreError::validation("motif", "un motif de correction est obligatoire"));
        }
        let author = self.author.clone();
        let tx = self.conn.transaction()?;
        let (status, version): (String, i64) = tx.query_row("SELECT status, version FROM encounter WHERE id = ?1", [encounter_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
        if status != "validated" {
            return Err(CoreError::Refused("seule une consultation validée se corrige par amendement".into()));
        }
        if version != expected_version {
            return Err(CoreError::Conflict { expected: expected_version, actual: version });
        }
        tx.execute("UPDATE encounter SET status = 'amending', version = version + 1, updated_at = ?1 WHERE id = ?2", params![now(), encounter_id])?;
        audit_on(&tx, &author, "start_amendment", "encounter", Some(encounter_id), None, None, None, Some(reason.trim()))?;
        tx.commit()?;
        self.load_encounter(encounter_id, true)
    }

    pub fn revisions(&self, encounter_id: &str) -> Result<Vec<RevisionRow>> {
        let mut st = self.conn.prepare("SELECT revision, reason, created_at, author, snapshot FROM encounter_revision WHERE encounter_id = ?1 ORDER BY revision")?;
        let rows = st
            .query_map([encounter_id], |r| {
                let snap: String = r.get(4)?;
                Ok(RevisionRow { revision: r.get(0)?, reason: r.get(1)?, created_at: r.get(2)?, author: r.get(3)?, snapshot: serde_json::from_str(&snap).unwrap_or(Value::Null) })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn audit_for(&self, encounter_id: &str) -> Result<Vec<AuditRow>> {
        let mut st = self.conn.prepare("SELECT at, kind, field, old_value, new_value, reason, author FROM audit_event WHERE entity_id = ?1 ORDER BY id")?;
        let rows = st
            .query_map([encounter_id], |r| Ok(AuditRow { at: r.get(0)?, kind: r.get(1)?, field: r.get(2)?, old_value: r.get(3)?, new_value: r.get(4)?, reason: r.get(5)?, author: r.get(6)? }))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Supprime un brouillon jamais validé et sans aucune donnée (création par erreur).
    pub fn discard_empty_draft(&mut self, encounter_id: &str) -> Result<()> {
        let full = self.load_encounter(encounter_id, false)?;
        if full.meta.status != "draft" || full.values.iter().any(|v| v.field != "visit_date") || !full.bewe.is_empty() || !full.exposures.is_empty() || !full.prevention.is_empty() || full.meta.revision > 0 {
            return Err(CoreError::Refused("seul un brouillon entièrement vide peut être abandonné".into()));
        }
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM encounter WHERE id = ?1", [encounter_id])?;
        let left: i64 = tx.query_row("SELECT count(*) FROM encounter WHERE patient_id = ?1", [&full.meta.patient_id], |r| r.get(0))?;
        if left == 0 {
            tx.execute("DELETE FROM patient_identity WHERE patient_id = ?1", [&full.meta.patient_id])?;
            tx.execute("DELETE FROM patient WHERE id = ?1", [&full.meta.patient_id])?;
        }
        audit_on(&tx, &self.author, "discard_empty_draft", "encounter", Some(encounter_id), None, None, None, None)?;
        tx.commit()?;
        Ok(())
    }
}

pub(crate) fn write_identity(tx: &Connection, patient_id: &str, idn: &IdentityInput) -> Result<()> {
    let ln = text_or_none(&idn.last_name);
    let fnm = text_or_none(&idn.first_name);
    let full = [ln.clone(), fnm.clone()].into_iter().flatten().collect::<Vec<_>>().join(" ");
    let key = (!full.is_empty()).then(|| crate::domain::legacy::identity_key(&full));
    tx.execute(
        "INSERT INTO patient_identity(patient_id, last_name, first_name, full_name_raw, hospital_id, identity_key) VALUES (?1,?2,?3,?4,?5,?6)
         ON CONFLICT(patient_id) DO UPDATE SET last_name=excluded.last_name, first_name=excluded.first_name, full_name_raw=excluded.full_name_raw, hospital_id=excluded.hospital_id, identity_key=excluded.identity_key",
        params![patient_id, ln, fnm, (!full.is_empty()).then_some(full), text_or_none(&idn.hospital_id), key],
    )?;
    Ok(())
}

/// Date du jour proposée pour une nouvelle consultation (métadonnée de saisie, modifiable).
fn propose_today(tx: &Connection, encounter_id: &str, author: &str) -> Result<()> {
    let today = super::today().format("%Y-%m-%d").to_string();
    write_value(tx, encounter_id, author, &StoredValue { field: "visit_date".into(), value_text: Some(today), date_precision: Some("day".into()), ..Default::default() }, None)?;
    sync_denormalized(tx, encounter_id)
}

pub(crate) fn insert_encounter(tx: &Connection, patient_id: &str, mode: &str, author: &str) -> Result<String> {
    let eid = new_id();
    let t = now();
    tx.execute(
        "INSERT INTO encounter(id, patient_id, collection_mode, form_version, status, revision, version, examiner, created_at, updated_at) VALUES (?1,?2,?3,?4,'draft',0,1,?5,?6,?6)",
        params![eid, patient_id, mode, if mode == "legacy_retrospective" { "legacy-import" } else { FORM_VERSION }, author, t],
    )?;
    audit_on(tx, author, "create", "encounter", Some(&eid), None, None, Some(mode), None)?;
    Ok(eid)
}
