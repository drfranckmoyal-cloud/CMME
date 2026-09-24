//! Espace étude : projet, sélection de la visite index, export pseudonymisé figé (liste blanche).

use crate::domain::bewe::SEXTANTS;
use crate::domain::catalog::{self, Kind, FORM_VERSION, PREVENTION_ACTIONS};
use crate::domain::values::StoredValue;
use crate::error::{CoreError, Result};
use crate::stats::bewe_stats;
use crate::store::encounters::{load_bewe, load_exposures, load_flags, load_prevention, load_values};
use crate::store::listing::bewe_analysis;
use crate::store::{audit_on, new_id, now, Store, SCHEMA_VERSION};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

pub const EXPORT_FORMAT: &str = "CMME-EXPORT-1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub include_legacy: bool,
    pub include_prospective: bool,
    pub exclude_open_anomalies: bool,
    pub plan_version: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInput {
    pub name: String,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub include_legacy: bool,
    pub include_prospective: bool,
    pub exclude_open_anomalies: bool,
    pub plan_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedPatient {
    pub patient_id: String,
    pub patient_code: String,
    pub index_encounter: String,
    pub eligible_encounters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub patients_total: usize,
    pub encounters_total: usize,
    pub excluded_encounters: BTreeMap<String, usize>,
    pub excluded_patients: Vec<(String, String, String)>,
    pub included: Vec<SelectedPatient>,
    pub bewe_analysable_index: usize,
}

/// Liste blanche des colonnes exportables, par fichier. Tout le reste est interdit.
pub fn whitelist() -> BTreeMap<&'static str, Vec<String>> {
    let mut m = BTreeMap::new();
    m.insert("patients.csv", ["study_id", "n_eligible_visits", "index_visit_study_id"].iter().map(|s| s.to_string()).collect());
    let mut visits: Vec<String> = ["study_id", "visit_study_id", "is_index", "collection_mode", "form_version", "status", "revision", "visit_year", "visit_date", "visit_date_precision", "open_anomalies", "accepted_anomalies"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    for f in export_fields() {
        visits.push(f.code.to_string());
        if f.allow_range {
            visits.push(format!("{}__max", f.code));
        }
        if f.kind == Kind::Number {
            visits.push(format!("{}__precision", f.code));
        }
        visits.push(format!("{}__missing", f.code));
        visits.push(format!("{}__source", f.code));
    }
    m.insert("visits.csv", visits);
    let mut b: Vec<String> = vec!["visit_study_id".into(), "is_index".into()];
    for (k, _, _) in SEXTANTS {
        b.push(format!("s_{k}"));
        b.push(format!("s_{k}__missing"));
        b.push(format!("s_{k}__unassessable_reason"));
    }
    for c in ["sextants_filled", "total_derived", "total_historical", "historical_band_min", "historical_band_max", "analysis_value", "analysis_origin", "flags"] {
        b.push(c.into());
    }
    m.insert("bewe.csv", b);
    m.insert("exposures.csv", ["visit_study_id", "is_index", "group", "group_status", "category", "temporality", "freq_min", "freq_max", "freq_unit"].iter().map(|s| s.to_string()).collect());
    m.insert(
        "care_actions.csv",
        ["visit_study_id", "is_index", "action", "decision", "confirmed", "proposed_by", "already_used", "done_in_consultation", "given_today", "refused", "route", "tray_minutes", "protocol_version"].iter().map(|s| s.to_string()).collect(),
    );
    m
}

/// Champs du catalogue autorisés à l'export : jamais de texte libre.
pub fn export_fields() -> Vec<&'static catalog::Field> {
    catalog::catalog().fields.iter().filter(|f| f.export && f.kind != Kind::Text).collect()
}

fn csv_cell(v: &Value) -> String {
    let s = match v {
        Value::Null => return String::new(),
        Value::Number(n) => return n.to_string(),
        Value::Bool(b) => return if *b { "1".into() } else { "0".into() },
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    // Neutralisation de l'injection de formule dans les tableurs (texte uniquement).
    if s.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{s}")
    } else {
        s
    }
}

fn to_csv(header: &[String], rows: &[BTreeMap<String, Value>]) -> Result<String> {
    let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_writer(vec![]);
    w.write_record(header).map_err(|e| CoreError::Io(e.to_string()))?;
    for r in rows {
        let rec: Vec<String> = header.iter().map(|h| r.get(h).map(csv_cell).unwrap_or_default()).collect();
        w.write_record(&rec).map_err(|e| CoreError::Io(e.to_string()))?;
    }
    let bytes = w.into_inner().map_err(|e| CoreError::Io(e.to_string()))?;
    Ok(String::from_utf8(bytes).unwrap_or_default())
}

fn random_study_id(prefix: &str) -> Result<String> {
    const ALPHA: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut b = [0u8; 8];
    getrandom::fill(&mut b).map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(format!("{prefix}-{}", b.iter().map(|x| ALPHA[(*x as usize) % ALPHA.len()] as char).collect::<String>()))
}

impl Store {
    pub fn create_project(&mut self, input: ProjectInput) -> Result<Project> {
        if input.name.trim().len() < 3 {
            return Err(CoreError::validation("nom", "nom du projet trop court"));
        }
        for d in [&input.period_start, &input.period_end].into_iter().flatten() {
            if !d.is_empty() && chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").is_err() {
                return Err(CoreError::validation("période", "date AAAA-MM-JJ attendue"));
            }
        }
        if !input.include_legacy && !input.include_prospective {
            return Err(CoreError::validation("recueil", "au moins un mode de recueil"));
        }
        let id = new_id();
        let blank = |x: &Option<String>| x.clone().filter(|s| !s.trim().is_empty());
        self.conn.execute(
            "INSERT INTO research_project(id, name, period_start, period_end, include_legacy, include_prospective, exclude_open_anomalies, plan_version, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![id, input.name.trim(), blank(&input.period_start), blank(&input.period_end), input.include_legacy as i64, input.include_prospective as i64, input.exclude_open_anomalies as i64, blank(&input.plan_version), now()],
        )?;
        self.audit("create_project", "research_project", Some(&id), None, None, Some(input.name.trim()), None)?;
        self.project(&id)
    }

    pub fn project(&self, id: &str) -> Result<Project> {
        self.conn
            .query_row("SELECT * FROM research_project WHERE id = ?1", [id], |r| {
                Ok(Project {
                    id: r.get("id")?,
                    name: r.get("name")?,
                    period_start: r.get("period_start")?,
                    period_end: r.get("period_end")?,
                    include_legacy: r.get::<_, i64>("include_legacy")? == 1,
                    include_prospective: r.get::<_, i64>("include_prospective")? == 1,
                    exclude_open_anomalies: r.get::<_, i64>("exclude_open_anomalies")? == 1,
                    plan_version: r.get("plan_version")?,
                    created_at: r.get("created_at")?,
                })
            })
            .optional()?
            .ok_or_else(|| CoreError::NotFound("projet".into()))
    }

    pub fn projects(&self) -> Result<Vec<Project>> {
        let mut st = self.conn.prepare("SELECT id FROM research_project ORDER BY created_at DESC")?;
        let ids: Vec<String> = st.query_map([], |r| r.get(0))?.collect::<rusqlite::Result<_>>()?;
        ids.iter().map(|i| self.project(i)).collect()
    }

    /// Retirer (ou rendre) l'éligibilité d'un dossier pour un projet, sans toucher aux soins.
    pub fn set_eligibility(&mut self, project_id: &str, patient_id: &str, excluded_reason: Option<String>) -> Result<()> {
        self.project(project_id)?;
        match excluded_reason.filter(|r| !r.trim().is_empty()) {
            Some(r) => {
                self.conn.execute(
                    "INSERT INTO research_eligibility(project_id, patient_id, status, reason, at) VALUES (?1,?2,'excluded',?3,?4) ON CONFLICT(project_id, patient_id) DO UPDATE SET reason = excluded.reason, at = excluded.at",
                    params![project_id, patient_id, r.trim(), now()],
                )?;
            }
            None => {
                self.conn.execute("DELETE FROM research_eligibility WHERE project_id = ?1 AND patient_id = ?2", params![project_id, patient_id])?;
            }
        }
        self.audit("eligibility", "research_project", Some(project_id), Some(patient_id), None, None, None)?;
        Ok(())
    }

    /// Sélection déterministe : consultations validées dans le périmètre ; visite index = première
    /// consultation éligible datée ; ordre impossible à établir = dossier ambigu, exclu et compté.
    pub fn selection(&self, project_id: &str) -> Result<Selection> {
        let p = self.project(project_id)?;
        let rows = self.all_rows(false)?;
        let mut excluded_e: BTreeMap<String, usize> = BTreeMap::new();
        let elig_excl: HashMap<String, String> = {
            let mut st = self.conn.prepare("SELECT patient_id, reason FROM research_eligibility WHERE project_id = ?1")?;
            let v = st.query_map([project_id], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<rusqlite::Result<Vec<(String, String)>>>()?;
            v.into_iter().collect()
        };
        let open_flags: HashMap<String, i64> = {
            let mut st = self.conn.prepare("SELECT encounter_id, count(*) FROM encounter_flag WHERE status = 'open' GROUP BY encounter_id")?;
            let v = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<rusqlite::Result<Vec<(String, i64)>>>()?;
            v.into_iter().collect()
        };
        let mut by_patient: BTreeMap<String, (String, Vec<&crate::store::listing::EncounterRow>)> = BTreeMap::new();
        let mut all_patients: BTreeMap<String, String> = BTreeMap::new();
        for r in &rows {
            all_patients.insert(r.patient_id.clone(), r.patient_code.clone());
            let reason = if r.status != "validated" {
                Some("non_validee")
            } else if r.collection_mode == "legacy_retrospective" && !p.include_legacy {
                Some("historique_hors_projet")
            } else if r.collection_mode == "structured_prospective" && !p.include_prospective {
                Some("prospectif_hors_projet")
            } else if p.exclude_open_anomalies && open_flags.get(&r.encounter_id).copied().unwrap_or(0) > 0 {
                Some("anomalie_ouverte")
            } else if p.period_start.is_some() || p.period_end.is_some() {
                match &r.visit_date {
                    None => Some("date_absente_periode_non_verifiable"),
                    Some(d) => {
                        let lo = p.period_start.as_deref().map(|s| d.as_str() < s).unwrap_or(false);
                        let hi = p.period_end.as_deref().map(|e| d.as_str() > e).unwrap_or(false);
                        let partial = r.visit_date_precision.as_deref() != Some("day") && (p.period_start.as_deref().map(|s| s.starts_with(d.as_str())).unwrap_or(false) || p.period_end.as_deref().map(|e| e.starts_with(d.as_str())).unwrap_or(false));
                        if partial {
                            Some("date_imprecise_periode_non_verifiable")
                        } else if lo || hi {
                            Some("hors_periode")
                        } else {
                            None
                        }
                    }
                }
            } else {
                None
            };
            match reason {
                Some(x) => *excluded_e.entry(x.into()).or_insert(0) += 1,
                None => by_patient.entry(r.patient_id.clone()).or_insert((r.patient_code.clone(), vec![])).1.push(r),
            }
        }
        let mut included = vec![];
        let mut excluded_p = vec![];
        for (pid, (code, encs)) in by_patient {
            if let Some(reason) = elig_excl.get(&pid) {
                excluded_p.push((pid, code, format!("éligibilité retirée : {reason}")));
                continue;
            }
            let index = if encs.len() == 1 {
                Some(encs[0])
            } else {
                let all_dated = encs.iter().all(|e| e.visit_date.is_some() && e.visit_date_precision.as_deref() == Some("day"));
                if all_dated {
                    let mut sorted = encs.clone();
                    sorted.sort_by(|a, b| a.visit_date.cmp(&b.visit_date));
                    (sorted[0].visit_date != sorted[1].visit_date).then_some(sorted[0])
                } else {
                    None
                }
            };
            match index {
                Some(i) => included.push(SelectedPatient { patient_id: pid, patient_code: code, index_encounter: i.encounter_id.clone(), eligible_encounters: encs.iter().map(|e| e.encounter_id.clone()).collect() }),
                None => excluded_p.push((pid, code, "visite index indéterminable (dates absentes ou identiques)".into())),
            }
        }
        let bewe_ok = included.iter().filter(|s| rows.iter().find(|r| r.encounter_id == s.index_encounter).and_then(|r| r.bewe.value).is_some()).count();
        Ok(Selection { patients_total: all_patients.len(), encounters_total: rows.len(), excluded_encounters: excluded_e, excluded_patients: excluded_p, included, bewe_analysable_index: bewe_ok })
    }

    fn study_id(&self, project_id: &str, kind: &str, local: &str) -> Result<String> {
        if let Some(s) = self.conn.query_row("SELECT study_id FROM study_id_map WHERE project_id = ?1 AND kind = ?2 AND local_id = ?3", params![project_id, kind, local], |r| r.get(0)).optional()? {
            return Ok(s);
        }
        loop {
            let sid = random_study_id(if kind == "patient" { "S" } else { "V" })?;
            let n = self.conn.execute("INSERT OR IGNORE INTO study_id_map(project_id, kind, local_id, study_id) VALUES (?1,?2,?3,?4)", params![project_id, kind, local, sid])?;
            if n == 1 {
                return Ok(sid);
            }
        }
    }

    /// Fige une extraction : fichiers calculés une fois, stockés dans la base chiffrée, écrits dans
    /// le dossier choisi. Une correction ultérieure n'altère jamais un instantané existant.
    pub fn freeze_export(&mut self, project_id: &str, exact_dates: bool, app_version: &str) -> Result<ExportSnapshot> {
        let p = self.project(project_id)?;
        let sel = self.selection(project_id)?;
        if sel.included.is_empty() {
            return Err(CoreError::Refused("aucun dossier inclus : rien à exporter".into()));
        }
        let wl = whitelist();
        let fields = export_fields();
        let mut patients = vec![];
        let mut visits = vec![];
        let mut bewe_rows = vec![];
        let mut expo_rows = vec![];
        let mut care_rows = vec![];
        let mut index_values = vec![];
        let mut index_origins = vec![];
        let mut availability: BTreeMap<String, (BTreeMap<String, usize>, BTreeMap<String, usize>)> = BTreeMap::new();
        for sp in &sel.included {
            let sid = self.study_id(project_id, "patient", &sp.patient_id)?;
            let mut idx_vsid = String::new();
            for eid in &sp.eligible_encounters {
                let is_index = *eid == sp.index_encounter;
                let vsid = self.study_id(project_id, "visit", eid)?;
                if is_index {
                    idx_vsid = vsid.clone();
                }
                let meta = crate::store::encounters::load_meta(&self.conn, eid)?;
                let vals: Vec<StoredValue> = load_values(&self.conn, eid)?;
                let flags = load_flags(&self.conn, eid)?;
                let legacy = meta.collection_mode == "legacy_retrospective";
                let absent = if legacy { "not_recorded" } else { "not_asked" };
                let mut v: BTreeMap<String, Value> = BTreeMap::new();
                v.insert("study_id".into(), json!(sid));
                v.insert("visit_study_id".into(), json!(vsid));
                v.insert("is_index".into(), json!(is_index as i64));
                v.insert("collection_mode".into(), json!(meta.collection_mode));
                v.insert("form_version".into(), json!(meta.form_version));
                v.insert("status".into(), json!(meta.status));
                v.insert("revision".into(), json!(meta.revision));
                v.insert("visit_year".into(), meta.visit_date.as_ref().map(|d| json!(&d[..4])).unwrap_or(Value::Null));
                if exact_dates {
                    v.insert("visit_date".into(), json!(meta.visit_date));
                    v.insert("visit_date_precision".into(), json!(meta.visit_date_precision));
                }
                v.insert("open_anomalies".into(), json!(flags.iter().filter(|f| f.status == "open").map(|f| f.flag.clone()).collect::<Vec<_>>().join("|")));
                v.insert("accepted_anomalies".into(), json!(flags.iter().filter(|f| f.status == "accepted").map(|f| f.flag.clone()).collect::<Vec<_>>().join("|")));
                for f in &fields {
                    let sv = vals.iter().find(|x| x.field == f.code);
                    let key_state;
                    match sv {
                        Some(sv) if sv.has_value() => {
                            let val = match f.kind {
                                Kind::Number => sv.value_num.map(|x| json!(x)).unwrap_or(Value::Null),
                                Kind::Multi => json!(serde_json::from_str::<Vec<String>>(sv.value_text.as_deref().unwrap_or("[]")).unwrap_or_default().join("|")),
                                Kind::Date => {
                                    if exact_dates {
                                        json!(sv.value_text)
                                    } else {
                                        json!(sv.value_text.as_deref().map(|d| &d[..4]))
                                    }
                                }
                                _ => json!(sv.value_text),
                            };
                            v.insert(f.code.into(), val);
                            if f.allow_range {
                                v.insert(format!("{}__max", f.code), sv.value_num_max.map(|x| json!(x)).unwrap_or(Value::Null));
                            }
                            if f.kind == Kind::Number {
                                v.insert(format!("{}__precision", f.code), json!(sv.precision));
                            }
                            v.insert(format!("{}__source", f.code), json!(sv.source_type));
                            key_state = "available".to_string();
                        }
                        Some(sv) => {
                            v.insert(format!("{}__missing", f.code), json!(sv.missing_reason));
                            v.insert(format!("{}__source", f.code), json!(sv.source_type));
                            key_state = sv.missing_reason.clone().unwrap_or_default();
                        }
                        None => {
                            // Champ de l'ancien recueil sur une consultation structurée : non applicable.
                            // Champ du nouveau formulaire sur une ligne historique : non consigné.
                            let r = if f.legacy && !legacy { "not_applicable" } else { absent };
                            v.insert(format!("{}__missing", f.code), json!(r));
                            key_state = r.to_string();
                        }
                    }
                    if is_index {
                        let e = availability.entry(f.code.to_string()).or_default();
                        let side = if legacy { &mut e.0 } else { &mut e.1 };
                        *side.entry(key_state).or_insert(0) += 1;
                    }
                }
                visits.push(v);
                // BEWE
                let sx = load_bewe(&self.conn, eid)?;
                let an = bewe_analysis(meta.bewe_total_derived, meta.bewe_total_historical, sx.len() as i64);
                let mut b: BTreeMap<String, Value> = BTreeMap::new();
                b.insert("visit_study_id".into(), json!(vsid));
                b.insert("is_index".into(), json!(is_index as i64));
                for (k, _, _) in SEXTANTS {
                    let row = sx.iter().find(|r| r.sextant == k);
                    b.insert(format!("s_{k}"), row.and_then(|r| r.score).map(|x| json!(x)).unwrap_or(Value::Null));
                    b.insert(format!("s_{k}__missing"), match row {
                        Some(r) => json!(r.missing_reason),
                        None => json!(if legacy { "not_recorded" } else { "not_asked" }),
                    });
                    b.insert(format!("s_{k}__unassessable_reason"), json!(row.and_then(|r| r.unassessable_reason.clone())));
                }
                b.insert("sextants_filled".into(), json!(sx.len()));
                b.insert("total_derived".into(), json!(meta.bewe_total_derived));
                b.insert("total_historical".into(), json!(meta.bewe_total_historical));
                b.insert("historical_band_min".into(), json!(meta.bewe_historical_band.map(|x| x.0)));
                b.insert("historical_band_max".into(), json!(meta.bewe_historical_band.map(|x| x.1)));
                b.insert("analysis_value".into(), json!(an.value));
                b.insert("analysis_origin".into(), json!(an.origin));
                b.insert("flags".into(), json!(flags.iter().map(|f| format!("{}:{}", f.flag, f.status)).collect::<Vec<_>>().join("|")));
                if is_index {
                    index_values.push(an.value);
                    index_origins.push(an.origin.clone());
                }
                bewe_rows.push(b);
                // Expositions
                let ex = load_exposures(&self.conn, eid)?;
                for (grp, status_field) in [("drink", "acidic_drinks_status"), ("food", "acidic_foods_status")] {
                    let st = vals.iter().find(|x| x.field == status_field);
                    let status = st.and_then(|s| s.value_text.clone().or(s.missing_reason.clone())).unwrap_or_else(|| absent.to_string());
                    let items: Vec<_> = ex.iter().filter(|e| e.grp == grp).collect();
                    if items.is_empty() {
                        let mut r = BTreeMap::new();
                        r.insert("visit_study_id".into(), json!(vsid));
                        r.insert("is_index".into(), json!(is_index as i64));
                        r.insert("group".into(), json!(grp));
                        r.insert("group_status".into(), json!(status));
                        expo_rows.push(r);
                    }
                    for e in items {
                        let mut r = BTreeMap::new();
                        r.insert("visit_study_id".into(), json!(vsid));
                        r.insert("is_index".into(), json!(is_index as i64));
                        r.insert("group".into(), json!(grp));
                        r.insert("group_status".into(), json!(status));
                        r.insert("category".into(), json!(e.category));
                        r.insert("temporality".into(), json!(e.temporality));
                        r.insert("freq_min".into(), json!(e.freq_min));
                        r.insert("freq_max".into(), json!(e.freq_max));
                        r.insert("freq_unit".into(), json!(e.freq_unit));
                        expo_rows.push(r);
                    }
                }
                // Actions de prévention : HBD, protocole, composantes (statuts distincts).
                let base = |action: &str| {
                    let mut r: BTreeMap<String, Value> = BTreeMap::new();
                    r.insert("visit_study_id".into(), json!(vsid));
                    r.insert("is_index".into(), json!(is_index as i64));
                    r.insert("action".into(), json!(action));
                    r
                };
                if !legacy {
                    for f in ["hbd_teaching", "prevention_protocol"] {
                        let mut r = base(f);
                        let s = vals.iter().find(|x| x.field == f);
                        r.insert("decision".into(), json!(s.and_then(|s| s.value_text.clone()).unwrap_or_else(|| "not_asked".into())));
                        care_rows.push(r);
                    }
                }
                for a in load_prevention(&self.conn, eid)? {
                    let mut r = base(&a.action_type);
                    r.insert("decision".into(), json!(a.decision));
                    r.insert("confirmed".into(), json!(a.confirmed));
                    r.insert("proposed_by".into(), json!(a.proposed_by));
                    r.insert("already_used".into(), json!(a.st_already_used));
                    r.insert("done_in_consultation".into(), json!(a.st_done_in_consultation));
                    r.insert("given_today".into(), json!(a.st_given_today));
                    r.insert("refused".into(), json!(a.st_refused));
                    r.insert("route".into(), json!(a.route));
                    r.insert("tray_minutes".into(), json!(a.tray_minutes));
                    r.insert("protocol_version".into(), json!(crate::domain::catalog::PROTOCOL_VERSION));
                    care_rows.push(r);
                }
            }
            let mut pr = BTreeMap::new();
            pr.insert("study_id".into(), json!(sid));
            pr.insert("n_eligible_visits".into(), json!(sp.eligible_encounters.len()));
            pr.insert("index_visit_study_id".into(), json!(idx_vsid));
            patients.push(pr);
        }
        let mut files: BTreeMap<String, String> = BTreeMap::new();
        for (name, rows) in [("patients.csv", &patients), ("visits.csv", &visits), ("bewe.csv", &bewe_rows), ("exposures.csv", &expo_rows), ("care_actions.csv", &care_rows)] {
            let mut header = wl[name].clone();
            if name == "visits.csv" && !exact_dates {
                header.retain(|h| h != "visit_date" && h != "visit_date_precision");
            }
            // Contrôle de la liste blanche : aucune clé produite hors liste.
            for r in rows.iter() {
                for k in r.keys() {
                    if !wl[name].contains(k) {
                        return Err(CoreError::Refused(format!("colonne non autorisée à l'export : {k}")));
                    }
                }
            }
            files.insert(name.into(), to_csv(&header, rows)?);
        }
        let bs = bewe_stats(&index_values, &index_origins);
        files.insert("dictionary.json".into(), serde_json::to_string_pretty(&dictionary())?);
        files.insert("dictionary.md".into(), dictionary_md());
        let quality = json!({
            "selection": {
                "patients_in_base": sel.patients_total,
                "visits_in_base": sel.encounters_total,
                "excluded_visits_by_reason": sel.excluded_encounters,
                "excluded_patients": sel.excluded_patients.iter().map(|(_, _, r)| r.clone()).fold(BTreeMap::<String, usize>::new(), |mut m, r| { *m.entry(r).or_insert(0) += 1; m }),
                "included_patients": sel.included.len(),
                "bewe_analysable_index": sel.bewe_analysable_index,
            },
            "availability_index_visits": availability.iter().map(|(k, (l, p))| (k.clone(), json!({"legacy_retrospective": l, "structured_prospective": p}))).collect::<BTreeMap<_, _>>(),
            "bewe_index": bs,
        });
        files.insert("quality_report.json".into(), serde_json::to_string_pretty(&quality)?);
        files.insert("quality_report.md".into(), quality_md(&p, &quality));
        let hashes: BTreeMap<String, String> = files.iter().map(|(k, v)| (k.clone(), hex::encode(Sha256::digest(v.as_bytes())))).collect();
        let counts: BTreeMap<String, usize> = [("patients.csv", patients.len()), ("visits.csv", visits.len()), ("bewe.csv", bewe_rows.len()), ("exposures.csv", expo_rows.len()), ("care_actions.csv", care_rows.len())].iter().map(|(a, b)| (a.to_string(), *b)).collect();
        let snap_id = new_id();
        let created = now();
        let manifest = json!({
            "format": EXPORT_FORMAT,
            "snapshot_id": snap_id,
            "project": {"name": p.name, "period_start": p.period_start, "period_end": p.period_end, "include_legacy": p.include_legacy, "include_prospective": p.include_prospective, "exclude_open_anomalies": p.exclude_open_anomalies, "plan_version": p.plan_version},
            "created_at": created,
            "versions": {"app": app_version, "schema": SCHEMA_VERSION, "form": FORM_VERSION, "dictionary": FORM_VERSION, "protocol": crate::domain::catalog::PROTOCOL_VERSION},
            "selection_rules": [
                "Consultations validées uniquement",
                "Visite index : première consultation éligible datée au jour près ; ordre indéterminable = dossier exclu et compté",
                "BEWE analysable : total dérivé des six sextants, sinon total historique validé ; conflit = non analysable",
                "Aucune imputation ; raisons de manque dans les colonnes __missing",
            ],
            "exact_dates": exact_dates,
            "rows": counts,
            "sha256": hashes,
            "summary": {"index_bewe_n": bs.n_analysable, "index_bewe_median": bs.distribution.median, "index_bewe_ge9": bs.ge9.k},
            "privacy": "Pseudonymisé, pas anonymisé : combinaison âge/service/diagnostic potentiellement identifiante. Aucun nom, aucune date de naissance, aucun texte libre.",
        });
        let files_json = serde_json::to_string(&files)?;
        let author = self.author.clone();
        let tx = self.conn.transaction()?;
        tx.execute("INSERT INTO export_snapshot(id, project_id, created_at, manifest, files) VALUES (?1,?2,?3,?4,?5)", params![snap_id, project_id, created, manifest.to_string(), files_json])?;
        audit_on(&tx, &author, "freeze_export", "research_project", Some(project_id), None, None, Some(&snap_id), None)?;
        tx.commit()?;
        Ok(ExportSnapshot { id: snap_id, project_id: project_id.into(), created_at: created, manifest })
    }

    pub fn snapshots(&self, project_id: &str) -> Result<Vec<ExportSnapshot>> {
        let mut st = self.conn.prepare("SELECT id, project_id, created_at, manifest FROM export_snapshot WHERE project_id = ?1 ORDER BY created_at DESC")?;
        let rows = st
            .query_map([project_id], |r| {
                let m: String = r.get(3)?;
                Ok(ExportSnapshot { id: r.get(0)?, project_id: r.get(1)?, created_at: r.get(2)?, manifest: serde_json::from_str(&m).unwrap_or(Value::Null) })
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows)
    }

    pub fn snapshot_files(&self, snapshot_id: &str) -> Result<BTreeMap<String, String>> {
        let f: String = self.conn.query_row("SELECT files FROM export_snapshot WHERE id = ?1", [snapshot_id], |r| r.get(0)).optional()?.ok_or_else(|| CoreError::NotFound("instantané".into()))?;
        Ok(serde_json::from_str(&f)?)
    }

    /// Écrit un instantané (toujours les mêmes octets) dans un nouveau sous-dossier de la destination.
    pub fn write_snapshot(&self, snapshot_id: &str, dest_dir: &Path) -> Result<String> {
        if !dest_dir.is_dir() {
            return Err(CoreError::Io("dossier de destination introuvable".into()));
        }
        let snaps = self.conn.query_row("SELECT manifest, created_at FROM export_snapshot WHERE id = ?1", [snapshot_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        let files = self.snapshot_files(snapshot_id)?;
        let stamp = snaps.1.replace([':', '-'], "").chars().take(15).collect::<String>().replace('T', "-");
        let dir = dest_dir.join(format!("CMME-export-{stamp}-{}", &snapshot_id[..8]));
        if dir.exists() {
            return Err(CoreError::Refused("cet instantané a déjà été écrit dans ce dossier".into()));
        }
        let tmp = dest_dir.join(format!(".cmme-export-{}.tmp", new_id()));
        std::fs::create_dir(&tmp)?;
        let r = (|| -> Result<()> {
            for (name, content) in &files {
                std::fs::write(tmp.join(name), content)?;
            }
            let manifest: Value = serde_json::from_str(&snaps.0)?;
            std::fs::write(tmp.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;
            Ok(())
        })();
        if let Err(e) = r {
            let _ = std::fs::remove_dir_all(&tmp);
            return Err(e);
        }
        std::fs::rename(&tmp, &dir)?;
        self.audit("write_export", "export_snapshot", Some(snapshot_id), None, None, None, None)?;
        Ok(dir.to_string_lossy().into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSnapshot {
    pub id: String,
    pub project_id: String,
    pub created_at: String,
    pub manifest: Value,
}

pub fn dictionary() -> Value {
    let fields: Vec<Value> = export_fields()
        .iter()
        .map(|f| {
            json!({
                "code": f.code, "label_fr": f.label, "label_en": f.label_en, "kind": f.kind, "section": f.section,
                "legacy_only": f.legacy, "unit": f.unit, "min": f.min, "max": f.max, "integer": f.integer, "range_allowed": f.allow_range,
                "values": f.options.iter().map(|o| json!({"code": o.code, "label_fr": o.label})).collect::<Vec<_>>(),
                "multi_separator": if f.kind == Kind::Multi { Some("|") } else { None },
            })
        })
        .collect();
    json!({
        "version": FORM_VERSION,
        "missing_reasons": crate::domain::missing::MissingReason::ALL.iter().map(|m| json!({"code": m.as_str(), "label_fr": m.label_fr()})).collect::<Vec<_>>(),
        "fields": fields,
        "bewe": {
            "sextants": SEXTANTS.iter().map(|(k, l, t)| json!({"code": k, "label_fr": l, "teeth": t})).collect::<Vec<_>>(),
            "scores": [0, 1, 2, 3],
            "total_derived": "Somme des six sextants, seulement si les six sont scorés ; sinon vide",
            "categories": ["0-2", "3-8", "9-13", "14-18"],
            "analysis_value": "total_derived si disponible, sinon total_historical validé ; conflit = vide",
        },
        "exposure_categories": {"drink": catalog::DRINKS.iter().map(|(c, l)| json!({"code": c, "label_fr": l})).collect::<Vec<_>>(), "food": catalog::FOODS.iter().map(|(c, l)| json!({"code": c, "label_fr": l})).collect::<Vec<_>>()},
        "prevention_actions": PREVENTION_ACTIONS.iter().map(|(c, l)| json!({"code": c, "label_fr": l})).collect::<Vec<_>>(),
        "csv": {"separator": ",", "encoding": "UTF-8", "formula_neutralization": "Un texte commençant par = + - @ tabulation ou retour chariot est préfixé d'une apostrophe ; les nombres restent typés."},
    })
}

fn dictionary_md() -> String {
    let mut s = String::from("# Dictionnaire des variables exportées\n\nVersion du formulaire : ");
    s.push_str(FORM_VERSION);
    s.push_str("\n\n| Code | Libellé | Type | Unité | Valeurs |\n|---|---|---|---|---|\n");
    for f in export_fields() {
        let vals = f.options.iter().map(|o| format!("`{}` {}", o.code, o.label)).collect::<Vec<_>>().join(" ; ");
        s.push_str(&format!("| `{}` | {} | {:?} | {} | {} |\n", f.code, f.label, f.kind, f.unit.unwrap_or(""), vals));
    }
    s.push_str("\nRaisons de manque (colonnes `__missing`) : ");
    s.push_str(&crate::domain::missing::MissingReason::ALL.iter().map(|m| format!("`{}` {}", m.as_str(), m.label_fr())).collect::<Vec<_>>().join(" ; "));
    s.push('\n');
    s
}

fn quality_md(p: &Project, q: &Value) -> String {
    let sel = &q["selection"];
    let b = &q["bewe_index"];
    format!(
        "# Rapport de qualité — {}\n\n## Sélection\n\n- Dossiers dans la base : {}\n- Consultations dans la base : {}\n- Consultations exclues par motif : {}\n- Dossiers exclus : {}\n- **Dossiers inclus : {}**\n- BEWE analysable à la visite index : {}\n\n## BEWE à la visite index\n\n- N analysable : {}\n- Médiane [Q1–Q3] : {} [{}–{}]\n- Origines : {}\n\nLes proportions sont toujours rapportées n/N sur les seules valeurs disponibles. Aucune imputation.\n",
        p.name,
        sel["patients_in_base"],
        sel["visits_in_base"],
        sel["excluded_visits_by_reason"],
        sel["excluded_patients"],
        sel["included_patients"],
        sel["bewe_analysable_index"],
        b["n_analysable"],
        b["distribution"]["median"],
        b["distribution"]["q1"],
        b["distribution"]["q3"],
        b["origins"],
    )
}
