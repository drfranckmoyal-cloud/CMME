//! Statistiques descriptives du tableau filtré : effectifs patients/visites séparés, N analysable
//! par variable, aucune imputation.

use crate::domain::bewe::{self, CATEGORIES};
use crate::domain::catalog;
use crate::domain::stats::{distribution, proportion, Distribution, Proportion};
use crate::error::Result;
use crate::store::listing::{EncounterRow, Filter};
use crate::store::Store;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeweStats {
    pub n_visits: usize,
    pub n_analysable: usize,
    pub origins: BTreeMap<String, usize>,
    pub distribution: Distribution,
    pub categories: Vec<(String, usize)>,
    pub gt0: Proportion,
    pub ge9: Proportion,
    pub ge14: Proportion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completeness {
    pub field: String,
    pub label: String,
    pub n: usize,
    pub available: usize,
    pub missing: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub filters: Vec<String>,
    pub n_visits: usize,
    pub n_patients: usize,
    pub n_legacy: usize,
    pub n_prospective: usize,
    pub bewe: BeweStats,
    pub age_exact: Distribution,
    pub age_band_only: usize,
    pub age_missing: usize,
    pub services: BTreeMap<String, usize>,
    pub vomiting_prospective: BTreeMap<String, usize>,
    pub vomiting_legacy: BTreeMap<String, usize>,
    pub protocol_prospective: BTreeMap<String, usize>,
    pub prevention_legacy: BTreeMap<String, usize>,
    pub hbd_done: Proportion,
    pub completeness: Vec<Completeness>,
}

pub fn bewe_stats(values: &[Option<i64>], origins: &[String]) -> BeweStats {
    let vals: Vec<f64> = values.iter().flatten().map(|v| *v as f64).collect();
    let n = vals.len();
    let mut cats = vec![];
    for c in CATEGORIES {
        cats.push((c.to_string(), values.iter().flatten().filter(|v| bewe::category(**v as u8) == Some(c)).count()));
    }
    let mut o = BTreeMap::new();
    for x in origins {
        *o.entry(x.clone()).or_insert(0) += 1;
    }
    BeweStats {
        n_visits: values.len(),
        n_analysable: n,
        origins: o,
        distribution: distribution(&vals),
        categories: cats,
        gt0: proportion(vals.iter().filter(|v| **v > 0.0).count(), n),
        ge9: proportion(vals.iter().filter(|v| **v >= 9.0).count(), n),
        ge14: proportion(vals.iter().filter(|v| **v >= 14.0).count(), n),
    }
}

impl Store {
    pub fn stats(&self, filter: &Filter) -> Result<Stats> {
        let rows: Vec<EncounterRow> = self.list_encounters(filter, false)?;
        let ids: HashSet<&str> = rows.iter().map(|r| r.encounter_id.as_str()).collect();
        let patients: HashSet<&str> = rows.iter().map(|r| r.patient_id.as_str()).collect();
        let bw = bewe_stats(&rows.iter().map(|r| r.bewe.value).collect::<Vec<_>>(), &rows.iter().map(|r| r.bewe.origin.clone()).collect::<Vec<_>>());
        let ages: Vec<f64> = rows.iter().filter_map(|r| r.age_years).collect();
        let count = |f: &dyn Fn(&EncounterRow) -> Option<String>| {
            let mut m = BTreeMap::new();
            for r in &rows {
                if let Some(k) = f(r) {
                    *m.entry(k).or_insert(0) += 1;
                }
            }
            m
        };
        let prosp: Vec<&EncounterRow> = rows.iter().filter(|r| r.collection_mode == "structured_prospective").collect();
        // Complétude des champs essentiels.
        let mut per: HashMap<(String, String), (bool, Option<String>)> = HashMap::new();
        {
            let mut st = self.conn.prepare("SELECT encounter_id, field, missing_reason FROM field_value")?;
            for r in st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?)))? {
                let (e, f, m) = r?;
                if ids.contains(e.as_str()) {
                    per.insert((e, f), (m.is_none(), m));
                }
            }
        }
        let mut completeness = vec![];
        for f in catalog::catalog().fields.iter().filter(|f| f.essential) {
            let mut c = Completeness { field: f.code.into(), label: f.label.into(), n: rows.len(), available: 0, missing: BTreeMap::new() };
            for r in &rows {
                match per.get(&(r.encounter_id.clone(), f.code.to_string())) {
                    Some((true, _)) => c.available += 1,
                    Some((false, Some(m))) => *c.missing.entry(m.clone()).or_insert(0) += 1,
                    _ => {
                        let reason = if r.collection_mode == "legacy_retrospective" { "not_recorded" } else { "not_asked" };
                        *c.missing.entry(reason.into()).or_insert(0) += 1;
                    }
                }
            }
            completeness.push(c);
        }
        Ok(Stats {
            filters: filter.describe(),
            n_visits: rows.len(),
            n_patients: patients.len(),
            n_legacy: rows.len() - prosp.len(),
            n_prospective: prosp.len(),
            bewe: bw,
            age_exact: distribution(&ages),
            age_band_only: rows.iter().filter(|r| r.age_years.is_none() && r.age_band.is_some()).count(),
            age_missing: rows.iter().filter(|r| r.age_years.is_none() && r.age_band.is_none()).count(),
            services: count(&|r| Some(r.service_code.clone().unwrap_or("non_renseigne".into()))),
            vomiting_prospective: count(&|r| (r.collection_mode == "structured_prospective").then(|| if r.vomiting_origin.as_deref() == Some("prospective") { r.vomiting.clone().unwrap_or_default() } else { "not_asked".into() })),
            vomiting_legacy: count(&|r| (r.collection_mode == "legacy_retrospective").then(|| r.vomiting.clone().unwrap_or("not_recorded".into()))),
            protocol_prospective: count(&|r| (r.collection_mode == "structured_prospective").then(|| if r.protocol_origin.as_deref() == Some("prospective") { r.protocol.clone().unwrap_or_default() } else { "not_asked".into() })),
            prevention_legacy: count(&|r| (r.collection_mode == "legacy_retrospective").then(|| r.protocol.clone().unwrap_or("not_recorded".into()))),
            hbd_done: proportion(prosp.iter().filter(|r| r.hbd.as_deref() == Some("done")).count(), prosp.len()),
            completeness,
        })
    }
}
