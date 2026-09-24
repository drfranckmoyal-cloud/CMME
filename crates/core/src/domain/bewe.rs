//! BEWE (Bartlett, Ganss, Lussi 2008) : six sextants anatomiques, total 0–18.
//! Politique conservatrice du projet : aucun total si un sextant manque ou n'est pas évaluable.

use serde::{Deserialize, Serialize};

/// Clés anatomiques permanentes, dans l'ordre de numérotation.
pub const SEXTANTS: [(&str, &str, &str); 6] = [
    ("upper_right", "Supérieur droit", "17–14"),
    ("upper_anterior", "Antérieur supérieur", "13–23"),
    ("upper_left", "Supérieur gauche", "24–27"),
    ("lower_left", "Inférieur gauche", "37–34"),
    ("lower_anterior", "Antérieur inférieur", "33–43"),
    ("lower_right", "Inférieur droit", "44–47"),
];

pub const UNASSESSABLE_REASONS: [(&str, &str); 4] = [
    ("edentulous", "Édentement"),
    ("restoration", "Restauration masquant l'évaluation"),
    ("limited_access", "Accès limité"),
    ("other", "Autre"),
];

pub fn is_sextant_key(k: &str) -> bool {
    SEXTANTS.iter().any(|(key, _, _)| *key == k)
}

/// Dent FDI → sextant anatomique. Sert à tester le mapping (la position à l'écran n'est jamais utilisée).
pub fn sextant_of_tooth(fdi: u8) -> Option<&'static str> {
    let quadrant = fdi / 10;
    let n = fdi % 10;
    if !(1..=8).contains(&n) {
        return None;
    }
    let anterior = n <= 3;
    Some(match (quadrant, anterior) {
        (1, false) => "upper_right",
        (2, false) => "upper_left",
        (1, true) | (2, true) => "upper_anterior",
        (3, false) => "lower_left",
        (4, false) => "lower_right",
        (3, true) | (4, true) => "lower_anterior",
        _ => return None,
    })
}

/// Valide un score saisi : entiers 0, 1, 2, 3 uniquement.
pub fn validate_score(v: &serde_json::Value) -> Option<u8> {
    match v {
        serde_json::Value::Number(n) => {
            let i = n.as_i64()?;
            if n.is_f64() && n.as_f64()?.fract() != 0.0 {
                return None;
            }
            (0..=3).contains(&i).then_some(i as u8)
        }
        _ => None,
    }
}

/// Total dérivé : seulement si les six sextants ont un score numérique.
pub fn derived_total(scores: &[Option<u8>; 6]) -> Option<u8> {
    let mut sum = 0u8;
    for s in scores {
        sum += (*s)?;
    }
    Some(sum)
}

pub fn category(total: u8) -> Option<&'static str> {
    match total {
        0..=2 => Some("0-2"),
        3..=8 => Some("3-8"),
        9..=13 => Some("9-13"),
        14..=18 => Some("14-18"),
        _ => None,
    }
}

pub const CATEGORIES: [&str; 4] = ["0-2", "3-8", "9-13", "14-18"];

/// Proposition de normalisation d'une cellule BEWE historique.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegacyBewe {
    pub raw: String,
    pub proposed_total: Option<u8>,
    pub band: Option<(u8, u8)>,
    pub flag: Option<&'static str>,
    pub requires_review: bool,
}

fn normalize_dashes(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '–' | '—' | '‐' | '‑' | '−' => '-',
            _ => c,
        })
        .collect()
}

fn parse_band(s: &str) -> Option<(u8, u8)> {
    let (a, b) = s.split_once('-')?;
    let a: u8 = a.trim().parse().ok()?;
    let b: u8 = b.trim().parse().ok()?;
    (a <= b).then_some((a, b))
}

fn parse_int_0_99(s: &str) -> Option<u32> {
    let t = s.trim();
    if t.is_empty() || !t.chars().all(|c| c.is_ascii_digit()) || t.len() > 3 {
        return None;
    }
    t.parse().ok()
}

/// Grammaire contrôlée : `N`, `A–B`, `A–B (N)`, `(N)`. Tout le reste est à revoir.
pub fn parse_legacy(raw: &str) -> LegacyBewe {
    let mut out = LegacyBewe { raw: raw.to_string(), proposed_total: None, band: None, flag: None, requires_review: true };
    let s = normalize_dashes(raw.trim());
    if s.is_empty() {
        out.flag = Some("not_recorded");
        return out;
    }
    // Nombre seul
    if let Some(n) = parse_int_0_99(&s) {
        if n <= 18 {
            out.proposed_total = Some(n as u8);
            out.requires_review = false;
        } else {
            out.flag = Some("out_of_range");
        }
        return out;
    }
    // Parties : avant parenthèse, contenu de la parenthèse
    let (before, inside) = match (s.find('('), s.rfind(')')) {
        (Some(o), Some(c)) if c > o && s[c + 1..].trim().is_empty() && s.matches('(').count() == 1 => {
            (s[..o].trim().to_string(), Some(s[o + 1..c].trim().to_string()))
        }
        (None, None) => (s.clone(), None),
        _ => {
            out.flag = Some("unparseable");
            return out;
        }
    };
    let band = if before.is_empty() { None } else { parse_band(&before) };
    if !before.is_empty() && band.is_none() {
        out.flag = Some("unparseable");
        return out;
    }
    if let Some((a, b)) = band {
        if b > 18 {
            out.flag = Some("out_of_range");
            return out;
        }
        out.band = Some((a, b));
    }
    match inside {
        None => {
            // Tranche seule : pas de total exact, pas de point milieu.
            out.flag = Some("band_only");
        }
        Some(inner) => {
            if let Some(n) = parse_int_0_99(&inner) {
                if n > 18 {
                    out.flag = Some("out_of_range");
                    return out;
                }
                out.proposed_total = Some(n as u8);
                match out.band {
                    Some((a, b)) if (n as u8) < a || (n as u8) > b => {
                        out.flag = Some("band_mismatch");
                    }
                    _ => out.requires_review = false,
                }
            } else if !inner.is_empty() && inner.chars().any(|c| c.is_ascii_digit()) {
                out.flag = Some("ambiguous_exact_value");
            } else {
                out.flag = Some("unparseable");
            }
        }
    }
    out
}

pub fn flag_label(flag: &str) -> &'static str {
    match flag {
        "not_recorded" => "BEWE non consigné",
        "band_only" => "Tranche seule, sans total exact",
        "band_mismatch" => "Total entre parenthèses hors de la tranche",
        "ambiguous_exact_value" => "Valeur entre parenthèses ambiguë (plage)",
        "out_of_range" => "Valeur hors du domaine 0–18",
        "unparseable" => "Format non interprétable",
        _ => "Anomalie BEWE",
    }
}
