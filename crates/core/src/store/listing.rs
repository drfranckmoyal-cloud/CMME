//! Tableau des consultations (une ligne par consultation) et filtres partagés avec les statistiques.

use super::Store;
use crate::domain::bewe;
use crate::domain::values::StoredValue;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeweAnalysis {
    /// Valeur retenue pour l'analyse, ou None (manquante, incomplète, en conflit).
    pub value: Option<i64>,
    /// `derived` (six sextants), `historical` (total ancien validé), `conflict`, `incomplete`, `missing`.
    pub origin: String,
    pub sextants_filled: i64,
}

/// Vue déterministe `bewe_analysis_value` (contrat 02) : dérivé complet, sinon historique ; conflit visible.
pub fn bewe_analysis(derived: Option<i64>, historical: Option<i64>, sextants_filled: i64) -> BeweAnalysis {
    match (derived, historical) {
        (Some(d), Some(h)) if d != h => BeweAnalysis { value: None, origin: "conflict".into(), sextants_filled },
        (Some(d), _) => BeweAnalysis { value: Some(d), origin: "derived".into(), sextants_filled },
        (None, Some(h)) => BeweAnalysis { value: Some(h), origin: "historical".into(), sextants_filled },
        (None, None) if sextants_filled > 0 => BeweAnalysis { value: None, origin: "incomplete".into(), sextants_filled },
        _ => BeweAnalysis { value: None, origin: "missing".into(), sextants_filled },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterRow {
    pub encounter_id: String,
    pub patient_id: String,
    pub patient_code: String,
    pub display_name: Option<String>,
    pub collection_mode: String,
    pub status: String,
    pub revision: i64,
    pub visit_date: Option<String>,
    pub visit_date_precision: Option<String>,
    pub service_code: Option<String>,
    pub age_years: Option<f64>,
    pub age_band: Option<(Option<f64>, Option<f64>)>,
    pub age_missing: Option<String>,
    pub vomiting: Option<String>,
    pub vomiting_origin: Option<String>,
    pub bewe: BeweAnalysis,
    pub bewe_category: Option<String>,
    pub protocol: Option<String>,
    pub protocol_origin: Option<String>,
    pub hbd: Option<String>,
    pub open_flags: i64,
    pub updated_at: String,
    pub is_demo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Filter {
    pub text: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub services: Option<Vec<String>>,
    pub collection_mode: Option<String>,
    pub status: Option<String>,
    /// `missing`, `ge9`, `available`
    pub bewe: Option<String>,
    pub anomalies_only: Option<bool>,
}

impl Filter {
    pub fn describe(&self) -> Vec<String> {
        let mut d = vec![];
        if let Some(t) = self.text.as_ref().filter(|t| !t.is_empty()) {
            d.push(format!("recherche « {t} »"));
        }
        if self.date_from.is_some() || self.date_to.is_some() {
            d.push(format!("période {} → {}", self.date_from.clone().unwrap_or("…".into()), self.date_to.clone().unwrap_or("…".into())));
        }
        if let Some(s) = self.services.as_ref().filter(|s| !s.is_empty()) {
            d.push(format!("services {}", s.join(", ")));
        }
        if let Some(m) = &self.collection_mode {
            d.push(if m == "legacy_retrospective" { "historique".into() } else { "prospectif".into() });
        }
        if let Some(s) = &self.status {
            d.push(format!("statut {s}"));
        }
        if let Some(b) = &self.bewe {
            d.push(format!("BEWE {b}"));
        }
        if self.anomalies_only == Some(true) {
            d.push("anomalies ouvertes".into());
        }
        d
    }

    pub fn matches(&self, r: &EncounterRow) -> bool {
        if let Some(t) = self.text.as_ref().map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty()) {
            let in_code = r.patient_code.to_lowercase().contains(&t);
            let in_name = r.display_name.as_ref().map(|n| crate::domain::legacy::identity_key(n).contains(&crate::domain::legacy::identity_key(&t))).unwrap_or(false);
            if !in_code && !in_name {
                return false;
            }
        }
        if let Some(from) = self.date_from.as_ref().filter(|d| !d.is_empty()) {
            match &r.visit_date {
                Some(d) if d.as_str() >= from.as_str() => {}
                _ => return false,
            }
        }
        if let Some(to) = self.date_to.as_ref().filter(|d| !d.is_empty()) {
            match &r.visit_date {
                // Une date partielle (AAAA-MM) est comparée à sa borne la plus basse.
                Some(d) if d.as_str() <= to.as_str() => {}
                _ => return false,
            }
        }
        if let Some(s) = self.services.as_ref().filter(|s| !s.is_empty()) {
            if !r.service_code.as_ref().map(|c| s.contains(c)).unwrap_or(false) {
                return false;
            }
        }
        if let Some(m) = self.collection_mode.as_ref().filter(|m| !m.is_empty()) {
            if &r.collection_mode != m {
                return false;
            }
        }
        if let Some(st) = self.status.as_ref().filter(|m| !m.is_empty()) {
            if &r.status != st {
                return false;
            }
        }
        match self.bewe.as_deref() {
            Some("missing") if r.bewe.value.is_some() => return false,
            Some("available") if r.bewe.value.is_none() => return false,
            Some("ge9") if r.bewe.value.map(|v| v < 9).unwrap_or(true) => return false,
            _ => {}
        }
        if self.anomalies_only == Some(true) && r.open_flags == 0 {
            return false;
        }
        true
    }
}

impl Store {
    pub fn all_rows(&self, with_identity: bool) -> Result<Vec<EncounterRow>> {
        let mut vals: HashMap<String, Vec<StoredValue>> = HashMap::new();
        {
            let mut st = self.conn.prepare(
                "SELECT encounter_id, field, value_text, value_num, value_num_max, missing_reason FROM field_value
                 WHERE field IN ('age_years','age_band_min','age_band_max','vomiting_lifetime','vomiting_legacy_code','prevention_protocol','prevention_legacy','hbd_teaching')",
            )?;
            let rows = st.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    StoredValue { field: r.get(1)?, value_text: r.get(2)?, value_num: r.get(3)?, value_num_max: r.get(4)?, missing_reason: r.get(5)?, ..Default::default() },
                ))
            })?;
            for r in rows {
                let (eid, v) = r?;
                vals.entry(eid).or_default().push(v);
            }
        }
        let mut sext: HashMap<String, i64> = HashMap::new();
        {
            let mut st = self.conn.prepare("SELECT encounter_id, count(*) FROM bewe_sextant GROUP BY encounter_id")?;
            for r in st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))? {
                let (e, n) = r?;
                sext.insert(e, n);
            }
        }
        let mut flags: HashMap<String, i64> = HashMap::new();
        {
            let mut st = self.conn.prepare("SELECT encounter_id, count(*) FROM encounter_flag WHERE status = 'open' GROUP BY encounter_id")?;
            for r in st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))? {
                let (e, n) = r?;
                flags.insert(e, n);
            }
        }
        let mut st = self.conn.prepare(
            "SELECT e.id, e.patient_id, p.code, e.collection_mode, e.status, e.revision, e.visit_date, e.visit_date_precision, e.service_code,
                    e.bewe_total_derived, e.bewe_total_historical, e.updated_at, p.is_demo, i.full_name_raw
             FROM encounter e JOIN patient p ON p.id = e.patient_id LEFT JOIN patient_identity i ON i.patient_id = p.id
             WHERE p.merged_into IS NULL",
        )?;
        let base = st
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, Option<String>>(7)?,
                    r.get::<_, Option<String>>(8)?,
                    r.get::<_, Option<i64>>(9)?,
                    r.get::<_, Option<i64>>(10)?,
                    r.get::<_, String>(11)?,
                    r.get::<_, i64>(12)? == 1,
                    r.get::<_, Option<String>>(13)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut out = Vec::with_capacity(base.len());
        for (eid, pid, code, mode, status, revision, date, prec, service, derived, hist, updated, demo, name) in base {
            let v = vals.get(&eid).cloned().unwrap_or_default();
            let get = |f: &str| v.iter().find(|x| x.field == f);
            let filled = *sext.get(&eid).unwrap_or(&0);
            let analysis = bewe_analysis(derived, hist, filled);
            let (vom, vom_origin) = match (get("vomiting_lifetime"), get("vomiting_legacy_code")) {
                (Some(x), _) => (x.value_text.clone().or(x.missing_reason.clone()), Some("prospective".to_string())),
                (None, Some(x)) => (x.value_text.clone().map(|c| format!("legacy_{c}")), Some("legacy".to_string())),
                _ => (None, None),
            };
            let (proto, proto_origin) = match (get("prevention_protocol"), get("prevention_legacy")) {
                (Some(x), _) => (x.value_text.clone(), Some("prospective".to_string())),
                (None, Some(x)) => (x.value_text.clone(), Some("legacy".to_string())),
                _ => (None, None),
            };
            let band_min = get("age_band_min").and_then(|x| x.value_num);
            let band_max = get("age_band_max").and_then(|x| x.value_num);
            out.push(EncounterRow {
                encounter_id: eid.clone(),
                patient_id: pid,
                patient_code: code,
                display_name: if with_identity { name } else { None },
                collection_mode: mode,
                status,
                revision,
                visit_date: date,
                visit_date_precision: prec,
                service_code: service,
                age_years: get("age_years").and_then(|x| x.value_num),
                age_band: (band_min.is_some() || band_max.is_some()).then_some((band_min, band_max)),
                age_missing: get("age_years").and_then(|x| x.missing_reason.clone()),
                vomiting: vom,
                vomiting_origin: vom_origin,
                bewe_category: analysis.value.and_then(|t| bewe::category(t as u8)).map(String::from),
                bewe: analysis,
                protocol: proto,
                protocol_origin: proto_origin,
                hbd: get("hbd_teaching").and_then(|x| x.value_text.clone()),
                open_flags: *flags.get(&eid).unwrap_or(&0),
                updated_at: updated,
                is_demo: demo,
            });
        }
        // Tri stable : date décroissante, puis code.
        out.sort_by(|a, b| b.visit_date.cmp(&a.visit_date).then(a.patient_code.cmp(&b.patient_code)).then(a.encounter_id.cmp(&b.encounter_id)));
        Ok(out)
    }

    pub fn list_encounters(&self, filter: &Filter, with_identity: bool) -> Result<Vec<EncounterRow>> {
        Ok(self.all_rows(with_identity)?.into_iter().filter(|r| filter.matches(r)).collect())
    }

    pub fn patient_encounters(&self, patient_id: &str) -> Result<Vec<EncounterRow>> {
        Ok(self.all_rows(true)?.into_iter().filter(|r| r.patient_id == patient_id).collect())
    }
}
