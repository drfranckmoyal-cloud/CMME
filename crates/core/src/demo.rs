//! Jeu de démonstration entièrement fictif (profil « Démonstration », base séparée).
//! Totaux BEWE [0, 3, 11, 2, 14, 5, incomplet, 7], comme dans les maquettes.

use crate::domain::bewe::SEXTANTS;
use crate::domain::values::FieldInput;
use crate::error::Result;
use crate::store::encounters::SextantInput;
use crate::store::Store;
use serde_json::json;

struct DemoCase {
    service: &'static str,
    age: u32,
    vomiting: &'static str,
    sextants: [Option<u8>; 6],
    protocol: Option<&'static str>,
    hbd: bool,
    validate: bool,
}

const CASES: [DemoCase; 8] = [
    DemoCase { service: "centre_expert", age: 22, vomiting: "current", sextants: [Some(0); 6], protocol: Some("moderate"), hbd: true, validate: true },
    DemoCase { service: "sas", age: 31, vomiting: "past_only", sextants: [Some(1), Some(1), Some(1), Some(0), Some(0), Some(0)], protocol: Some("moderate"), hbd: false, validate: true },
    DemoCase { service: "centre_expert", age: 28, vomiting: "current", sextants: [Some(3), Some(2), Some(2), Some(1), Some(2), Some(1)], protocol: Some("advanced"), hbd: true, validate: true },
    DemoCase { service: "hdj", age: 40, vomiting: "never_reported", sextants: [Some(1), Some(0), Some(1), Some(0), Some(0), Some(0)], protocol: Some("none"), hbd: true, validate: true },
    DemoCase { service: "centre_expert", age: 24, vomiting: "current", sextants: [Some(3), Some(3), Some(2), Some(2), Some(2), Some(2)], protocol: Some("advanced"), hbd: false, validate: true },
    DemoCase { service: "sas", age: 35, vomiting: "past_only", sextants: [Some(1), Some(1), Some(1), Some(1), Some(1), Some(0)], protocol: Some("moderate"), hbd: false, validate: true },
    DemoCase { service: "hdj", age: 19, vomiting: "", sextants: [Some(1), Some(2), Some(1), None, Some(2), Some(1)], protocol: None, hbd: false, validate: false },
    DemoCase { service: "centre_expert", age: 26, vomiting: "current", sextants: [Some(1), Some(2), Some(1), Some(1), Some(2), Some(0)], protocol: None, hbd: false, validate: false },
];

pub fn seed_demo(store: &mut Store) -> Result<usize> {
    if !store.all_rows(false)?.is_empty() {
        return Ok(0);
    }
    let today = crate::store::today();
    for (i, c) in CASES.iter().enumerate() {
        let e = store.create_dossier(Some(format!("DEMO-{:03}", i + 1)), None, true)?;
        let id = e.meta.id.clone();
        let date = today - chrono::Duration::days(7 * (8 - i as i64));
        let mut inputs = vec![
            FieldInput { field: "visit_date".into(), value: Some(json!(date.format("%Y-%m-%d").to_string())), ..Default::default() },
            FieldInput { field: "service_code".into(), value: Some(json!(c.service)), ..Default::default() },
            FieldInput { field: "visit_type".into(), value: Some(json!("initial")), ..Default::default() },
            FieldInput { field: "age_years".into(), value: Some(json!(c.age)), ..Default::default() },
            FieldInput { field: "panoramic_review_status".into(), value: Some(json!("reviewed")), ..Default::default() },
            FieldInput { field: "intraoral_exam_status".into(), value: Some(json!("done")), ..Default::default() },
        ];
        inputs.push(if c.vomiting.is_empty() {
            FieldInput { field: "vomiting_lifetime".into(), missing_reason: Some("unknown".into()), ..Default::default() }
        } else {
            FieldInput { field: "vomiting_lifetime".into(), value: Some(json!(c.vomiting)), ..Default::default() }
        });
        if c.hbd {
            inputs.push(FieldInput { field: "hbd_teaching".into(), value: Some(json!("done")), ..Default::default() });
        }
        let mut v = store.save_fields(&id, e.meta.version, inputs)?.version;
        for (k, (key, _, _)) in SEXTANTS.iter().enumerate() {
            if let Some(s) = c.sextants[k] {
                v = store.set_sextant(&id, v, SextantInput { sextant: key.to_string(), score: Some(json!(s)), missing_reason: None, unassessable_reason: None })?.version;
            }
        }
        if let Some(p) = c.protocol {
            v = store.apply_protocol(&id, v, Some(p.to_string()))?.save.version;
            if p == "advanced" {
                v = store.update_prevention_action(&id, v, "tooth_mousse", crate::store::encounters::PreventionPatch { route: Some("tray".into()), tray_minutes: Some(json!(10)), ..Default::default() })?.version;
            }
            v = store.confirm_prevention(&id, v)?.version;
        }
        if c.validate {
            store.validate_encounter(&id, v)?;
        }
    }
    Ok(CASES.len())
}
