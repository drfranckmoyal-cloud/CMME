//! Intégration des fiches de consultation Pages (« Évaluation dentaire CMME »), exportées en texte.
//! Rapprochement avec les dossiers déjà repris du tableau récapitulatif : une fiche qui décrit la même
//! consultation la complète (amendement tracé, rien n'est écrasé) ; une divergence est signalée, jamais
//! tranchée ; un patient inconnu crée un nouveau dossier. Texte intégral de la fiche conservé.

use crate::domain::bewe;
use crate::domain::legacy::{self, identity_key};
use crate::domain::values::{self, FieldInput};
use crate::error::{CoreError, Result};
use crate::store::encounters::{insert_encounter, load_full, write_identity, write_value, IdentityInput};
use crate::store::{audit_on, new_id, now, Store};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Fiche {
    pub file: String,
    pub raw: String,
    pub date: Option<String>,
    pub name: Option<String>,
    pub age: Option<i64>,
    pub sex: Option<String>,
    pub service_raw: Option<String>,
    pub last_visit_months: Option<i64>,
    pub occupation: Option<String>,
    pub alcohol_raw: Option<String>,
    pub tca: Option<String>,
    pub bewe_raw: Option<String>,
    pub cao_c: Option<i64>,
    pub cao_a: Option<i64>,
    pub cao_o: Option<i64>,
    #[serde(default)]
    pub quasi_vide: bool,
    #[serde(default)]
    pub name_from_filename: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FicheReport {
    pub fiches: usize,
    pub new_patients: usize,
    pub enriched: Vec<(String, String)>,
    pub followups: Vec<(String, String)>,
    pub conflicts: Vec<(String, String, String)>,
    pub skipped: Vec<(String, String)>,
    /// Fiches rattachées à un dossier dont le nom s'écrit à une ou deux lettres près (même service).
    pub near_matches: Vec<(String, String)>,
}

fn lev(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + (a[i - 1] != b[j - 1]) as usize);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Nom de fichier générique (modèle dupliqué) : aucune identité exploitable.
fn generic_name(f: &Fiche) -> bool {
    let k = f.name.as_deref().map(identity_key).unwrap_or_default();
    f.name_from_filename && ["template", "modele", "sans titre", "copie", "document"].iter().any(|g| k.starts_with(g))
}

fn diagnosis(tca: &str) -> Option<&'static str> {
    let t = legacy::simplify(tca);
    let hits: Vec<&'static str> = [("anorexie", "AN"), ("boulimi", "BN"), ("hyperphagi", "BED"), ("arfid", "ARFID"), ("merycisme", "rumination")]
        .iter()
        .filter(|(k, _)| t.contains(k))
        .map(|(_, c)| *c)
        .collect();
    (hits.len() == 1).then(|| hits[0])
}

/// Valeurs structurées tirées de la fiche (uniquement ce qui est explicitement écrit).
fn fiche_values(f: &Fiche) -> Vec<FieldInput> {
    let mut v = vec![];
    let mut put = |field: &str, value: Value, precision: Option<&str>| {
        v.push(FieldInput { field: field.into(), value: Some(value), precision: precision.map(String::from), source_type: Some("legacy_import".into()), ..Default::default() });
    };
    if let Some(d) = &f.date {
        put("visit_date", json!(d), None);
    }
    if let Some(a) = f.age {
        put("age_years", json!(a), None);
    }
    if let Some(s) = &f.sex {
        put("sex_recorded", json!(s), None);
    }
    if let Some(s) = f.service_raw.as_deref().filter(|s| !s.trim().is_empty()) {
        put("service_code", json!(legacy::service(s).unwrap_or("autre")), None);
    }
    if let Some(m) = f.last_visit_months {
        put("last_dental_visit_months", json!(m), Some("estimated"));
    }
    if let Some(o) = f.occupation.as_deref().filter(|o| !o.trim().is_empty()) {
        put("occupation_text", json!(o.trim()), None);
    }
    if f.alcohol_raw.as_deref().map(|a| legacy::simplify(a) == "ras").unwrap_or(false) {
        put("alcohol_status", json!("none_reported"), None);
    }
    if let Some(t) = f.tca.as_deref().filter(|t| !t.trim().is_empty()) {
        put("ed_comment", json!(t), None);
        if let Some(d) = diagnosis(t) {
            put("ed_diagnosis", json!(d), None);
        }
    }
    for (field, n) in [("dmft_d", f.cao_c), ("dmft_m", f.cao_a), ("dmft_f", f.cao_o)] {
        if let Some(n) = n {
            put(field, json!(n), None);
        }
    }
    put("fiche_pages_text", json!(f.raw), None);
    v
}

fn fiche_bewe(f: &Fiche) -> Option<i64> {
    let raw = f.bewe_raw.as_deref()?.trim();
    if raw.is_empty() {
        return None;
    }
    let p = bewe::parse_legacy(raw);
    if p.flag.is_none() {
        p.proposed_total.map(|t| t as i64)
    } else {
        None
    }
}

fn existing_fields(conn: &Connection, eid: &str) -> Result<Vec<String>> {
    let mut st = conn.prepare("SELECT field FROM field_value WHERE encounter_id = ?1")?;
    let v = st.query_map([eid], |r| r.get(0))?.collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(v)
}

fn write_inputs(conn: &Connection, eid: &str, author: &str, inputs: &[FieldInput], only_missing: bool, record: &str) -> Result<usize> {
    let have = existing_fields(conn, eid)?;
    let mut n = 0;
    for i in inputs {
        if only_missing && have.contains(&i.field) {
            continue;
        }
        if let Some(sv) = values::validate(i)? {
            write_value(conn, eid, author, &sv, Some(record))?;
            n += 1;
        }
    }
    let date: Option<String> = conn.query_row("SELECT value_text FROM field_value WHERE encounter_id = ?1 AND field = 'visit_date'", [eid], |r| r.get(0)).optional()?.flatten();
    let service: Option<String> = conn.query_row("SELECT value_text FROM field_value WHERE encounter_id = ?1 AND field = 'service_code'", [eid], |r| r.get(0)).optional()?.flatten();
    conn.execute("UPDATE encounter SET visit_date = ?1, visit_date_precision = CASE WHEN ?1 IS NULL THEN NULL ELSE 'day' END, service_code = ?2 WHERE id = ?3", params![date, service, eid])?;
    Ok(n)
}

fn snapshot(conn: &Connection, eid: &str, author: &str, reason: &str) -> Result<()> {
    let t = now();
    conn.execute("UPDATE encounter SET status = 'validated', revision = revision + 1, version = version + 1, validated_at = ?1, updated_at = ?1 WHERE id = ?2", params![t, eid])?;
    let rev: i64 = conn.query_row("SELECT revision FROM encounter WHERE id = ?1", [eid], |r| r.get(0))?;
    let snap = load_full(conn, eid, false)?;
    conn.execute("INSERT INTO encounter_revision(encounter_id, revision, snapshot, reason, created_at, author) VALUES (?1,?2,?3,?4,?5,?6)", params![eid, rev, serde_json::to_string(&snap)?, reason, t, author])?;
    Ok(())
}

impl Store {
    /// `keep_existing_bewe` : en cas de BEWE divergent, garder la valeur du dossier (décision du praticien)
    /// et reprendre le reste de la fiche ; sinon, conflit signalé sans modification.
    pub fn integrate_fiches(&mut self, keep_existing_bewe: bool, mut fiches: Vec<Fiche>) -> Result<FicheReport> {
        let payload = serde_json::to_vec(&fiches)?;
        let sha = hex::encode(Sha256::digest(&payload));
        if self.conn.query_row("SELECT count(*) FROM source_document WHERE sha256 = ?1", [&sha], |r| r.get::<_, i64>(0))? > 0 {
            return Err(CoreError::Refused("ce lot de fiches a déjà été intégré : aucune duplication".into()));
        }
        // Ordre chronologique : la fiche la plus ancienne se rapproche en premier de la ligne du tableau.
        fiches.sort_by(|a, b| a.date.cmp(&b.date).then(a.file.cmp(&b.file)));
        let author = self.author.clone();
        let tx = self.conn.transaction()?;
        let doc = new_id();
        tx.execute(
            "INSERT INTO source_document(id, sha256, filename, format, size_bytes, imported_at, parser_version, content, config) VALUES (?1,?2,?3,'pages-txt',?4,?5,'cmme-fiches-1',?6,NULL)",
            params![doc, sha, "Fiches Pages 2024-2025 et 2025-2026", payload.len() as i64, now(), payload],
        )?;
        let mut rep = FicheReport { fiches: fiches.len(), ..Default::default() };
        for (i, f) in fiches.iter().enumerate() {
            let rid = new_id();
            let cells = serde_json::to_string(&vec![("fichier".to_string(), f.file.clone()), ("texte".to_string(), f.raw.clone())])?;
            let key = f.name.as_deref().map(identity_key).filter(|k| !k.is_empty());
            tx.execute(
                "INSERT INTO source_record(id, document_id, sheet, row_number, cells, identity_key, status, flags) VALUES (?1,?2,'Pages',?3,?4,?5,'pending','[]')",
                params![rid, doc, i as i64 + 1, cells, key],
            )?;
            let key = if generic_name(f) { None } else { key };
            let Some(key) = key else {
                tx.execute("UPDATE source_record SET status = 'excluded', exclusion_reason = 'fiche sans nom' WHERE id = ?1", [&rid])?;
                rep.skipped.push((f.file.clone(), "fiche sans nom (rubrique vide, fichier « modèle » non renommé) : patient non identifiable".into()));
                continue;
            };
            let inputs = fiche_values(f);
            let fb = fiche_bewe(f);
            let mut st = tx.prepare("SELECT p.id, p.code FROM patient p JOIN patient_identity i ON i.patient_id = p.id WHERE i.identity_key = ?1 AND p.merged_into IS NULL")?;
            let mut cands: Vec<(String, String)> = st.query_map([&key], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<rusqlite::Result<_>>()?;
            drop(st);
            if cands.is_empty() {
                // Orthographe voisine (une ou deux lettres) et même service : même patient, rattachement signalé.
                let service = f.service_raw.as_deref().and_then(legacy::service);
                let mut st = tx.prepare(
                    "SELECT p.id, p.code, i.identity_key, (SELECT group_concat(DISTINCT e.service_code) FROM encounter e WHERE e.patient_id = p.id)
                     FROM patient p JOIN patient_identity i ON i.patient_id = p.id WHERE p.merged_into IS NULL AND i.identity_key IS NOT NULL",
                )?;
                let near: Vec<(String, String)> = st
                    .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, Option<String>>(3)?)))?
                    .filter_map(|x| x.ok())
                    .filter(|(_, _, k, svc)| {
                        let d = lev(k, &key);
                        d >= 1 && d <= 2 && key.len() >= 8 && service.map(|s| svc.as_deref().unwrap_or("").split(',').any(|x| x == s)).unwrap_or(false)
                    })
                    .map(|(id, code, _, _)| (id, code))
                    .collect();
                drop(st);
                if near.len() == 1 {
                    rep.near_matches.push((f.file.clone(), near[0].1.clone()));
                    cands = near;
                }
            }
            if cands.len() > 1 {
                tx.execute("UPDATE source_record SET status = 'excluded', exclusion_reason = 'plusieurs dossiers portent ce nom' WHERE id = ?1", [&rid])?;
                rep.conflicts.push((f.file.clone(), cands.iter().map(|c| c.1.clone()).collect::<Vec<_>>().join(", "), "plusieurs dossiers portent ce nom : à rattacher à la main".into()));
                continue;
            }
            if let Some((pid, code)) = cands.first() {
                let mut st = tx.prepare("SELECT id, visit_date, bewe_total_historical, bewe_total_derived FROM encounter WHERE patient_id = ?1")?;
                let encs: Vec<(String, Option<String>, Option<i64>, Option<i64>)> = st.query_map([pid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?.collect::<rusqlite::Result<_>>()?;
                drop(st);
                let same = encs.iter().find(|e| e.1.is_some() && e.1 == f.date).or_else(|| encs.iter().find(|e| e.1.is_none()));
                if let Some((eid, _, hist, derived)) = same {
                    let known = derived.or(*hist);
                    let mut divergent = None;
                    if let (Some(a), Some(b)) = (known, fb) {
                        if a != b && keep_existing_bewe {
                            divergent = Some((a, b));
                        } else if a != b {
                            tx.execute("UPDATE source_record SET status = 'excluded', exclusion_reason = 'BEWE divergent' WHERE id = ?1", [&rid])?;
                            rep.conflicts.push((f.file.clone(), code.clone(), format!("BEWE {b} dans la fiche, {a} dans le dossier : rien n'a été modifié")));
                            continue;
                        }
                    }
                    let reason = format!("Complément depuis la fiche Pages « {} » (même consultation)", f.file);
                    write_inputs(&tx, eid, &author, &inputs, true, &rid)?;
                    if hist.is_none() && derived.is_none() {
                        if let Some(b) = fb {
                            tx.execute("UPDATE encounter SET bewe_total_historical = ?1, bewe_legacy_raw = coalesce(bewe_legacy_raw, ?2) WHERE id = ?3", params![b, f.bewe_raw, eid])?;
                        }
                    }
                    if let Some((a, b)) = divergent {
                        tx.execute(
                            "INSERT INTO encounter_flag(encounter_id, flag, detail, status, resolution, created_at) VALUES (?1,'bewe_fiche_divergent',?2,'resolved',?3,?4)",
                            params![eid, format!("BEWE {b} dans la fiche « {} », {a} dans le tableau", f.file), "Valeur du tableau conservée (décision du praticien)", now()],
                        )?;
                    }
                    snapshot(&tx, eid, &author, &reason)?;
                    audit_on(&tx, &author, "fiche_pages_complement", "encounter", Some(eid), None, None, Some(&rid), Some(&reason))?;
                    tx.execute("UPDATE source_record SET status = 'validated', encounter_id = ?1, link_patient_id = ?2, link_decision = 'same_patient' WHERE id = ?3", params![eid, pid, rid])?;
                    rep.enriched.push((f.file.clone(), code.clone()));
                    continue;
                }
                // Toutes les consultations connues ont une autre date : nouvelle consultation du même dossier.
                let eid = insert_encounter(&tx, pid, "legacy_retrospective", &author)?;
                tx.execute("UPDATE encounter SET source_record_id = ?1 WHERE id = ?2", params![rid, eid])?;
                write_inputs(&tx, &eid, &author, &inputs, false, &rid)?;
                if let Some(b) = fb {
                    tx.execute("UPDATE encounter SET bewe_total_historical = ?1, bewe_legacy_raw = ?2 WHERE id = ?3", params![b, f.bewe_raw, eid])?;
                }
                snapshot(&tx, &eid, &author, &format!("Reprise de la fiche Pages « {} »", f.file))?;
                tx.execute("UPDATE source_record SET status = 'validated', encounter_id = ?1, link_patient_id = ?2, link_decision = 'same_patient' WHERE id = ?3", params![eid, pid, rid])?;
                rep.followups.push((f.file.clone(), code.clone()));
                continue;
            }
            if f.quasi_vide {
                tx.execute("UPDATE source_record SET status = 'excluded', exclusion_reason = 'fiche non remplie' WHERE id = ?1", [&rid])?;
                rep.skipped.push((f.file.clone(), "fiche non remplie, patient absent de la base : non créé".into()));
                continue;
            }
            // Patient inconnu : nouveau dossier.
            let pid = new_id();
            let n: i64 = tx.query_row("SELECT count(*) FROM patient WHERE code LIKE 'H-%'", [], |r| r.get(0))?;
            let mut k = n + 1;
            let code = loop {
                let c = format!("H-{k:04}");
                if tx.query_row("SELECT count(*) FROM patient WHERE code = ?1", [&c], |r| r.get::<_, i64>(0))? == 0 {
                    break c;
                }
                k += 1;
            };
            tx.execute("INSERT INTO patient(id, code, created_at) VALUES (?1,?2,?3)", params![pid, code, now()])?;
            write_identity(&tx, &pid, &IdentityInput { last_name: f.name.clone(), first_name: None, hospital_id: None })?;
            let eid = insert_encounter(&tx, &pid, "legacy_retrospective", &author)?;
            tx.execute("UPDATE encounter SET source_record_id = ?1 WHERE id = ?2", params![rid, eid])?;
            write_inputs(&tx, &eid, &author, &inputs, false, &rid)?;
            if let Some(b) = fb {
                tx.execute("UPDATE encounter SET bewe_total_historical = ?1, bewe_legacy_raw = ?2 WHERE id = ?3", params![b, f.bewe_raw, eid])?;
            } else if f.bewe_raw.as_deref().map(|b| !b.trim().is_empty()).unwrap_or(false) {
                tx.execute("UPDATE encounter SET bewe_legacy_raw = ?1 WHERE id = ?2", params![f.bewe_raw, eid])?;
                tx.execute("INSERT INTO encounter_flag(encounter_id, flag, detail, status, created_at) VALUES (?1,'unparseable','BEWE de la fiche non interprétable','open',?2)", params![eid, now()])?;
            }
            snapshot(&tx, &eid, &author, &format!("Reprise de la fiche Pages « {} »", f.file))?;
            tx.execute("UPDATE source_record SET status = 'validated', encounter_id = ?1, link_patient_id = ?2, link_decision = 'new' WHERE id = ?3", params![eid, pid, rid])?;
            rep.new_patients += 1;
        }
        audit_on(&tx, &author, "integrate_fiches", "source_document", Some(&doc), None, None, Some(&serde_json::to_string(&json!({"fiches": rep.fiches, "nouveaux": rep.new_patients, "completes": rep.enriched.len(), "suivis": rep.followups.len(), "conflits": rep.conflicts.len(), "ecartees": rep.skipped.len()}))?), None)?;
        tx.commit()?;
        Ok(rep)
    }
}
