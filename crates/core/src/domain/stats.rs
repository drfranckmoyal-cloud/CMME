//! Statistiques descriptives : aucune imputation, dénominateurs explicites.

use serde::{Deserialize, Serialize};

/// Quantile par interpolation linéaire (méthode NumPy par défaut, (n−1)·p).
pub fn quantile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let h = (sorted.len() as f64 - 1.0) * p;
    let lo = h.floor() as usize;
    let hi = h.ceil() as usize;
    Some(sorted[lo] + (h - lo as f64) * (sorted[hi] - sorted[lo]))
}

pub fn mean(xs: &[f64]) -> Option<f64> {
    (!xs.is_empty()).then(|| xs.iter().sum::<f64>() / xs.len() as f64)
}

/// Écart-type d'échantillon (n−1).
pub fn sd(xs: &[f64]) -> Option<f64> {
    if xs.len() < 2 {
        return None;
    }
    let m = mean(xs)?;
    Some((xs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (xs.len() as f64 - 1.0)).sqrt())
}

/// Intervalle de Wilson à 95 %.
pub fn wilson(k: usize, n: usize) -> Option<(f64, f64)> {
    if n == 0 {
        return None;
    }
    let z = 1.959_963_984_540_054_f64;
    let n_f = n as f64;
    let p = k as f64 / n_f;
    let denom = 1.0 + z * z / n_f;
    let centre = (p + z * z / (2.0 * n_f)) / denom;
    let half = z * ((p * (1.0 - p) / n_f) + z * z / (4.0 * n_f * n_f)).sqrt() / denom;
    Some(((centre - half).max(0.0), (centre + half).min(1.0)))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Proportion {
    pub k: usize,
    pub n: usize,
    pub pct: Option<f64>,
    pub ci_low: Option<f64>,
    pub ci_high: Option<f64>,
}

pub fn proportion(k: usize, n: usize) -> Proportion {
    let ci = wilson(k, n);
    Proportion { k, n, pct: (n > 0).then(|| 100.0 * k as f64 / n as f64), ci_low: ci.map(|c| 100.0 * c.0), ci_high: ci.map(|c| 100.0 * c.1) }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Distribution {
    pub n: usize,
    pub median: Option<f64>,
    pub q1: Option<f64>,
    pub q3: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
    pub sd: Option<f64>,
}

pub fn distribution(values: &[f64]) -> Distribution {
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Distribution {
        n: v.len(),
        median: quantile(&v, 0.5),
        q1: quantile(&v, 0.25),
        q3: quantile(&v, 0.75),
        min: v.first().copied(),
        max: v.last().copied(),
        mean: mean(&v),
        sd: sd(&v),
    }
}
