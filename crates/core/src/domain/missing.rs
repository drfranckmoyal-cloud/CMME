//! États de recueil : une valeur absente porte toujours une raison, jamais un zéro ou un « non ».

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingReason {
    NotRecorded,
    NotAsked,
    Unknown,
    Declined,
    NotAssessable,
    NotApplicable,
    AmbiguousSource,
}

impl MissingReason {
    pub const ALL: [MissingReason; 7] = [
        MissingReason::NotRecorded,
        MissingReason::NotAsked,
        MissingReason::Unknown,
        MissingReason::Declined,
        MissingReason::NotAssessable,
        MissingReason::NotApplicable,
        MissingReason::AmbiguousSource,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            MissingReason::NotRecorded => "not_recorded",
            MissingReason::NotAsked => "not_asked",
            MissingReason::Unknown => "unknown",
            MissingReason::Declined => "declined",
            MissingReason::NotAssessable => "not_assessable",
            MissingReason::NotApplicable => "not_applicable",
            MissingReason::AmbiguousSource => "ambiguous_source",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.as_str() == s)
    }
    pub fn label_fr(self) -> &'static str {
        match self {
            MissingReason::NotRecorded => "Non consigné (ancien support)",
            MissingReason::NotAsked => "Non renseigné",
            MissingReason::Unknown => "Inconnu",
            MissingReason::Declined => "Refus de répondre",
            MissingReason::NotAssessable => "Non évaluable",
            MissingReason::NotApplicable => "Non applicable",
            MissingReason::AmbiguousSource => "Source ambiguë",
        }
    }
}

pub const SOURCE_TYPES: [&str; 6] = [
    "patient_report",
    "medical_record",
    "clinical_exam",
    "instrument_measurement",
    "legacy_import",
    "clinician_adjudication",
];

pub const CERTAINTIES: [&str; 4] = ["documented", "reported", "suspected", "unresolved"];
