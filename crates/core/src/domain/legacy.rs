//! Règles déterministes de normalisation de l'ancien recueil. Chaque règle propose ; rien n'est validé seul.

use serde::Serialize;
use unicode_normalization::UnicodeNormalization;

/// Identité normalisée pour repérer des candidats au rapprochement (jamais pour fusionner seul).
pub fn identity_key(s: &str) -> String {
    let base: String = s.nfd().filter(|c| !unicode_normalization::char::is_combining_mark(*c)).collect::<String>().to_lowercase();
    base.split(|c: char| !c.is_alphanumeric()).filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ")
}

pub fn simplify(s: &str) -> String {
    identity_key(s)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Proposal {
    /// Champ du catalogue (ou pseudo-champ `bewe_total_historical`, `patient_code`…).
    pub target: String,
    pub value: serde_json::Value,
    pub rule: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct Normalized {
    pub proposals: Vec<Proposal>,
    pub flags: Vec<&'static str>,
}

impl Normalized {
    fn prop(&mut self, target: &str, value: serde_json::Value, rule: &'static str) {
        self.proposals.push(Proposal { target: target.to_string(), value, rule });
    }
}

fn dashes(s: &str) -> String {
    s.chars().map(|c| if matches!(c, '–' | '—' | '‐' | '‑' | '−') { '-' } else { c }).collect()
}

/// Âge : nombre exact, classe « A-B », « <A », « >A », éventuellement « A-B (N) ».
pub fn age(raw: &str) -> Normalized {
    let mut n = Normalized::default();
    let s = dashes(raw.trim());
    if s.is_empty() {
        n.flags.push("age_missing");
        return n;
    }
    let int = |t: &str| t.trim().parse::<u32>().ok().filter(|x| *x <= 110);
    if let Some(a) = int(&s) {
        n.prop("age_years", serde_json::json!(a), "age_exact");
        return n;
    }
    n.prop("age_band_legacy", serde_json::json!(raw.trim()), "age_band_raw");
    let (band_part, exact) = match (s.find('('), s.rfind(')')) {
        (Some(o), Some(c)) if c > o => (s[..o].trim().to_string(), int(&s[o + 1..c])),
        _ => (s.clone(), None),
    };
    let mut ok = false;
    if let Some((a, b)) = band_part.split_once('-') {
        if let (Some(a), Some(b)) = (int(a), int(b)) {
            if a <= b {
                n.prop("age_band_min", serde_json::json!(a), "age_band_bounds");
                n.prop("age_band_max", serde_json::json!(b), "age_band_bounds");
                ok = true;
                if let Some(e) = exact {
                    if e >= a && e <= b {
                        n.prop("age_years", serde_json::json!(e), "age_exact_in_parentheses");
                    } else {
                        n.flags.push("age_band_mismatch");
                    }
                }
            }
        }
    } else if let Some(rest) = band_part.strip_prefix('<') {
        if let Some(a) = int(rest) {
            n.prop("age_band_max", serde_json::json!(a.saturating_sub(1)), "age_band_below");
            ok = true;
        }
    } else if let Some(rest) = band_part.strip_prefix('>') {
        if let Some(a) = int(rest) {
            n.prop("age_band_min", serde_json::json!(a + 1), "age_band_above");
            ok = true;
        }
    }
    n.flags.push(if ok { "age_band_only" } else { "age_unparseable" });
    n
}

/// Oui / non d'une ancienne colonne : sens conservé, vide = manquant.
pub fn yes_no(raw: &str) -> Option<&'static str> {
    match simplify(raw).as_str() {
        "oui" | "o" | "yes" | "y" | "x" | "1" => Some("yes"),
        "non" | "n" | "no" | "0" => Some("no"),
        _ => None,
    }
}

pub fn vomiting(raw: &str) -> Normalized {
    let mut n = Normalized::default();
    if raw.trim().is_empty() {
        n.flags.push("vomiting_missing");
        return n;
    }
    match yes_no(raw) {
        Some(v) => n.prop("vomiting_legacy_code", serde_json::json!(v), "legacy_yes_no"),
        None => n.flags.push("vomiting_ambiguous"),
    }
    n
}

pub fn sex(raw: &str) -> Normalized {
    let mut n = Normalized::default();
    let v = match simplify(raw).as_str() {
        "" => return n,
        "f" | "femme" | "feminin" => "female",
        "h" | "m" | "homme" | "masculin" => "male",
        "nb" | "non binaire" | "autre" => "other",
        _ => {
            n.flags.push("sex_ambiguous");
            return n;
        }
    };
    n.prop("sex_recorded", serde_json::json!(v), "legacy_sex_code");
    n
}

pub fn prevention(raw: &str) -> Normalized {
    let mut n = Normalized::default();
    if raw.trim().is_empty() {
        return n;
    }
    n.prop("prevention_protocol_legacy_raw", serde_json::json!(raw), "raw_kept");
    let s = simplify(raw);
    let code = if s.contains("avance") {
        "advanced"
    } else if s.contains("modere") {
        "moderate"
    } else if s.contains("hbd") || s.contains("hygiene") {
        "hbd"
    } else {
        "other"
    };
    n.prop("prevention_legacy", serde_json::json!(code), "legacy_prevention_label");
    n
}

pub fn hygiene(raw: &str) -> Normalized {
    let mut n = Normalized::default();
    let s = simplify(raw);
    if s.is_empty() {
        return n;
    }
    let code = if s.starts_with("insuffisant") {
        "insufficient"
    } else if s.starts_with("suffisant") {
        "sufficient"
    } else if s.starts_with("iatrogen") {
        "iatrogenic"
    } else {
        n.flags.push("hygiene_out_of_category");
        return n;
    };
    n.prop("hygiene_legacy", serde_json::json!(code), "legacy_hygiene_label");
    n
}

pub fn care(raw: &str) -> Normalized {
    let mut n = Normalized::default();
    let s = simplify(raw);
    if s.is_empty() {
        return n;
    }
    let spec = !s.contains("non specialise");
    let code = if s.starts_with("suivi") {
        if spec { "specialized_followup" } else { "nonspecialized_followup" }
    } else if s.starts_with("soin") {
        if spec { "specialized_care" } else { "nonspecialized_care" }
    } else {
        n.flags.push("care_out_of_category");
        return n;
    };
    n.prop("care_legacy", serde_json::json!(code), "legacy_care_label");
    n
}

pub fn diet(raw: &str) -> Normalized {
    let mut n = Normalized::default();
    let s = simplify(raw);
    if s.is_empty() {
        return n;
    }
    let code = if s.starts_with("sans risque erosif") {
        "no_erosive"
    } else if s.starts_with("avec risque erosif") {
        "erosive"
    } else if s.contains("carieux") {
        "caries_risk"
    } else {
        "other"
    };
    n.prop("diet_risk_legacy", serde_json::json!(code), "legacy_diet_label");
    n
}

pub fn other_wear(raw: &str) -> Normalized {
    let mut n = Normalized::default();
    if raw.trim().is_empty() {
        return n;
    }
    match yes_no(raw) {
        Some(v) => n.prop("other_wear_legacy", serde_json::json!(v), "legacy_yes_no"),
        None => n.flags.push("other_wear_ambiguous"),
    }
    n
}

pub fn dmft(raw: &str) -> Normalized {
    let mut n = Normalized::default();
    let t = raw.trim();
    if t.is_empty() {
        return n;
    }
    match t.parse::<u32>() {
        Ok(x) if x <= 32 => n.prop("dmft_total_historical", serde_json::json!(x), "legacy_dmft_numeric"),
        _ => n.flags.push("dmft_to_review"),
    }
    n
}

pub fn service(raw: &str) -> Option<&'static str> {
    let s = simplify(raw);
    Some(if s.contains("intensif") {
        "hdj_intensif"
    } else if s.starts_with("hdj") || s.contains("hopital de jour") {
        "hdj"
    } else if s.contains("longue") {
        "hospit_longue"
    } else if s.contains("hospit") {
        "hospit_complete"
    } else if s.contains("centre expert") || s == "ce" {
        "centre_expert"
    } else if s == "sas" {
        "sas"
    } else {
        return None;
    })
}

/// Date d'une cellule : ISO ou JJ/MM/AAAA. Une date hors 2010–aujourd'hui est signalée, jamais corrigée.
pub fn visit_date(raw: &str, today: chrono::NaiveDate) -> Normalized {
    let mut n = Normalized::default();
    let t = raw.trim();
    if t.is_empty() {
        n.flags.push("date_missing");
        return n;
    }
    let d = chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d")
        .or_else(|_| chrono::NaiveDate::parse_from_str(t, "%d/%m/%Y"))
        .or_else(|_| chrono::NaiveDate::parse_from_str(t, "%d/%m/%y"))
        .or_else(|_| chrono::NaiveDate::parse_from_str(t.split(' ').next().unwrap_or(""), "%Y-%m-%d"));
    match d {
        Ok(d) => {
            n.prop("visit_date", serde_json::json!(d.format("%Y-%m-%d").to_string()), "date_parsed");
            let min = chrono::NaiveDate::from_ymd_opt(2010, 1, 1).unwrap();
            if d < min || d > today {
                n.flags.push("date_suspect");
            }
        }
        Err(_) => n.flags.push("date_unparseable"),
    }
    n
}

/// Fréquence « 2 à 3 fois/jour » : min, max, unité — jamais 2,5.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Frequency {
    pub min: f64,
    pub max: Option<f64>,
    pub unit: Option<&'static str>,
}

pub fn frequency(raw: &str) -> Option<Frequency> {
    let s = simplify(&dashes(raw)).replace(" a ", "-");
    let unit = if s.contains("jour") || s.ends_with(" j") {
        Some("day")
    } else if s.contains("sem") {
        Some("week")
    } else if s.contains("mois") {
        Some("month")
    } else {
        None
    };
    let nums: Vec<f64> = s
        .split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .filter(|p| !p.is_empty())
        .filter_map(|p| p.parse().ok())
        .collect();
    match nums.as_slice() {
        [a] => Some(Frequency { min: *a, max: None, unit }),
        [a, b] if b >= a && s.contains('-') => Some(Frequency { min: *a, max: Some(*b), unit }),
        _ => None,
    }
}

pub fn flag_label(flag: &str) -> &'static str {
    match flag {
        "age_missing" => "Âge absent",
        "age_band_only" => "Classe d'âge seule (pas d'âge exact)",
        "age_band_mismatch" => "Âge entre parenthèses hors de la classe",
        "age_unparseable" => "Âge non interprétable",
        "vomiting_missing" => "Vomissements non consignés",
        "vomiting_ambiguous" => "Vomissements : réponse ambiguë",
        "sex_ambiguous" => "Sexe : valeur non reconnue",
        "hygiene_out_of_category" => "Hygiène : valeur hors rubrique",
        "care_out_of_category" => "Soins : valeur hors rubrique",
        "other_wear_ambiguous" => "Autres usures : valeur ambiguë",
        "dmft_to_review" => "CAO non numérique",
        "date_missing" => "Date absente",
        "date_suspect" => "Date suspecte",
        "date_unparseable" => "Date non interprétable",
        "duplicate_candidate" => "Doublon possible",
        "duplicate_exact" => "Ligne identique à une autre",
        "changed_since_previous_version" => "Modifiée depuis la version précédente du fichier",
        "no_identity" => "Ligne sans identité",
        _ => super::bewe::flag_label(flag),
    }
}

/// Anomalies qui interdisent la validation en lot (revue individuelle obligatoire).
pub fn is_blocking(flag: &str) -> bool {
    !matches!(flag, "date_missing" | "age_missing" | "vomiting_missing" | "age_band_only")
}
