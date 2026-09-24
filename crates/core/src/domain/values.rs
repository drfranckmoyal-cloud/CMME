//! Validation native d'une saisie de champ, à partir du catalogue.

use super::catalog::{self, Field, Kind};
use super::missing::{MissingReason, CERTAINTIES, SOURCE_TYPES};
use crate::error::{CoreError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Saisie venant de l'interface ou de l'import.
/// `value` et `missing_reason` tous deux absents = effacer (retour à « non renseigné »).
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FieldInput {
    pub field: String,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub missing_reason: Option<String>,
    #[serde(default)]
    pub precision: Option<String>,
    #[serde(default)]
    pub source_type: Option<String>,
    #[serde(default)]
    pub certainty: Option<String>,
}

/// Valeur prête à être stockée (une ligne de `field_value`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StoredValue {
    pub field: String,
    pub value_text: Option<String>,
    pub value_num: Option<f64>,
    pub value_num_max: Option<f64>,
    pub precision: Option<String>,
    pub date_precision: Option<String>,
    pub missing_reason: Option<String>,
    pub source_type: Option<String>,
    pub certainty: Option<String>,
}

impl StoredValue {
    pub fn has_value(&self) -> bool {
        self.value_text.is_some() || self.value_num.is_some()
    }
}

pub fn parse_number(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => {
            let t = s.trim().replace(',', ".").replace(' ', "");
            if t.is_empty() {
                return None;
            }
            t.parse::<f64>().ok().filter(|x| x.is_finite())
        }
        _ => None,
    }
}

fn check_bounds(f: &Field, x: f64) -> Result<()> {
    if let Some(min) = f.min {
        if x < min {
            return Err(CoreError::validation(f.code, format!("inférieure au minimum {min}")));
        }
    }
    if let Some(max) = f.max {
        if x > max {
            return Err(CoreError::validation(f.code, format!("supérieure au maximum {max}")));
        }
    }
    if f.integer && x.fract() != 0.0 {
        return Err(CoreError::validation(f.code, "un nombre entier est attendu"));
    }
    Ok(())
}

/// Date clinique : `AAAA-MM-JJ`, `AAAA-MM` ou `AAAA` (précision conservée, aucun jour inventé).
pub fn parse_iso_date(s: &str) -> Option<(String, &'static str)> {
    let t = s.trim();
    let parts: Vec<&str> = t.split('-').collect();
    let num = |p: &str, len: usize| p.len() == len && p.chars().all(|c| c.is_ascii_digit());
    match parts.as_slice() {
        [y] if num(y, 4) => Some((t.to_string(), "year")),
        [y, m] if num(y, 4) && num(m, 2) => {
            let mm: u32 = m.parse().ok()?;
            (1..=12).contains(&mm).then(|| (t.to_string(), "month"))
        }
        [y, m, d] if num(y, 4) && num(m, 2) && num(d, 2) => {
            chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d").ok()?;
            Some((t.to_string(), "day"))
        }
        _ => None,
    }
}

pub fn validate(input: &FieldInput) -> Result<Option<StoredValue>> {
    let f = catalog::field(&input.field).ok_or_else(|| CoreError::validation(&input.field, "champ inconnu"))?;
    let mut out = StoredValue { field: f.code.to_string(), ..Default::default() };

    if let Some(st) = &input.source_type {
        if !SOURCE_TYPES.contains(&st.as_str()) {
            return Err(CoreError::validation(f.code, "source inconnue"));
        }
        out.source_type = Some(st.clone());
    }
    if let Some(c) = &input.certainty {
        if !CERTAINTIES.contains(&c.as_str()) {
            return Err(CoreError::validation(f.code, "certitude inconnue"));
        }
        out.certainty = Some(c.clone());
    }

    let value = input.value.as_ref().filter(|v| !v.is_null());
    match (value, &input.missing_reason) {
        (None, None) => return Ok(None),
        (Some(_), Some(_)) => return Err(CoreError::validation(f.code, "une valeur et une raison de manque ne peuvent coexister")),
        (None, Some(m)) => {
            let mr = MissingReason::parse(m).ok_or_else(|| CoreError::validation(f.code, "raison de manque inconnue"))?;
            out.missing_reason = Some(mr.as_str().to_string());
            return Ok(Some(out));
        }
        (Some(v), None) => match f.kind {
            Kind::Choice => {
                let s = v.as_str().ok_or_else(|| CoreError::validation(f.code, "code attendu"))?;
                if !f.options.iter().any(|o| o.code == s) {
                    return Err(CoreError::validation(f.code, "option inconnue"));
                }
                out.value_text = Some(s.to_string());
            }
            Kind::Multi => {
                let arr = v.as_array().ok_or_else(|| CoreError::validation(f.code, "liste attendue"))?;
                let mut codes: Vec<&str> = Vec::new();
                for x in arr {
                    let s = x.as_str().ok_or_else(|| CoreError::validation(f.code, "code attendu"))?;
                    if !f.options.iter().any(|o| o.code == s) {
                        return Err(CoreError::validation(f.code, "option inconnue"));
                    }
                    if !codes.contains(&s) {
                        codes.push(s);
                    }
                }
                if codes.is_empty() {
                    return Ok(None);
                }
                if codes.len() > 1 && codes.iter().any(|c| f.exclusive.contains(c)) {
                    return Err(CoreError::validation(f.code, "« aucun » ne peut pas coexister avec une autre réponse"));
                }
                // Ordre du catalogue, pour un stockage stable.
                let ordered: Vec<&str> = f.options.iter().map(|o| o.code).filter(|c| codes.contains(c)).collect();
                out.value_text = Some(serde_json::to_string(&ordered)?);
            }
            Kind::Number => {
                let (a, b) = match v {
                    Value::Object(o) => {
                        let a = o.get("min").and_then(parse_number).ok_or_else(|| CoreError::validation(f.code, "nombre attendu"))?;
                        let b = match o.get("max").filter(|x| !x.is_null()) {
                            Some(x) => Some(parse_number(x).ok_or_else(|| CoreError::validation(f.code, "nombre attendu"))?),
                            None => None,
                        };
                        (a, b)
                    }
                    other => (parse_number(other).ok_or_else(|| CoreError::validation(f.code, "nombre attendu"))?, None),
                };
                check_bounds(f, a)?;
                let b = match b {
                    Some(b) if b != a => {
                        if !f.allow_range {
                            return Err(CoreError::validation(f.code, "une plage n'est pas admise pour ce champ"));
                        }
                        check_bounds(f, b)?;
                        if b < a {
                            return Err(CoreError::validation(f.code, "borne haute inférieure à la borne basse"));
                        }
                        Some(b)
                    }
                    _ => None,
                };
                out.value_num = Some(a);
                out.value_num_max = b;
                out.precision = Some(match (b, input.precision.as_deref()) {
                    (Some(_), _) => "range".to_string(),
                    (None, Some("estimated")) => "estimated".to_string(),
                    _ => "exact".to_string(),
                });
            }
            Kind::Date => {
                let s = v.as_str().ok_or_else(|| CoreError::validation(f.code, "date attendue"))?;
                let (iso, prec) = parse_iso_date(s).ok_or_else(|| CoreError::validation(f.code, "date invalide (AAAA-MM-JJ, AAAA-MM ou AAAA)"))?;
                out.value_text = Some(iso);
                out.date_precision = Some(prec.to_string());
            }
            Kind::Text => {
                let s = v.as_str().ok_or_else(|| CoreError::validation(f.code, "texte attendu"))?;
                if s.trim().is_empty() {
                    return Ok(None);
                }
                // Texte conservé à l'identique : ni troncature ni nettoyage.
                out.value_text = Some(s.to_string());
            }
        },
    }
    Ok(Some(out))
}
