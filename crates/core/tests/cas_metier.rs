//! Les cas de recette fournis (specs/tests/cas_metier.json) exécutés tels quels.

mod common;
use cmme_core::domain::bewe::{self, SEXTANTS};
use serde_json::Value;

fn cases() -> Vec<Value> {
    let raw = std::fs::read_to_string(common::specs_dir().join("tests/cas_metier.json")).unwrap();
    let v: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v["synthetic"], true);
    v["cases"].as_array().unwrap().clone()
}

#[test]
fn tous_les_cas_metier_passent() {
    let mut n = 0;
    for c in cases() {
        let id = c["id"].as_str().unwrap();
        match c["kind"].as_str().unwrap() {
            "bewe_sum" => {
                let mut scores = [None; 6];
                for (i, (k, _, _)) in SEXTANTS.iter().enumerate() {
                    scores[i] = c["input"][k].as_u64().map(|x| x as u8);
                }
                let total = bewe::derived_total(&scores);
                let exp_total = c["expected"]["total"].as_u64().map(|x| x as u8);
                assert_eq!(total, exp_total, "{id} total");
                let cat = total.and_then(bewe::category);
                assert_eq!(cat, c["expected"]["category"].as_str(), "{id} catégorie");
            }
            "legacy_bewe_proposal" => {
                let p = bewe::parse_legacy(c["input"].as_str().unwrap());
                assert_eq!(p.proposed_total.map(|x| x as u64), c["expected"]["proposed_total"].as_u64(), "{id} total proposé");
                assert_eq!(p.flag, c["expected"]["flag"].as_str(), "{id} signalement");
                assert_eq!(p.requires_review, c["expected"]["requires_review"].as_bool().unwrap(), "{id} revue");
                // Aucun sextant n'est jamais reconstruit : le type ne le permet même pas.
                assert_eq!(c["expected"]["sextants_reconstructed"], false);
            }
            "bewe_sextant_validation" => {
                let accepted = bewe::validate_score(&c["input"]).is_some();
                assert_eq!(accepted, c["expected"]["accepted"].as_bool().unwrap(), "{id}");
            }
            k => panic!("type de cas inconnu {k}"),
        }
        n += 1;
    }
    assert_eq!(n, 21, "tous les cas fournis doivent être exécutés");
}

#[test]
fn seuils_bewe_de_la_specification() {
    // Section 7.3 de la spécification consolidée.
    let t = |s: [u8; 6]| bewe::derived_total(&s.map(Some));
    assert_eq!(t([0; 6]), Some(0));
    assert_eq!(t([1, 0, 1, 0, 0, 0]).and_then(bewe::category), Some("0-2"));
    assert_eq!(t([1, 1, 1, 0, 0, 0]).and_then(bewe::category), Some("3-8"));
    assert_eq!(t([2, 2, 1, 1, 1, 1]), Some(8));
    assert_eq!(t([2, 2, 2, 1, 1, 1]).and_then(bewe::category), Some("9-13"));
    assert_eq!(t([3, 2, 2, 2, 2, 2]), Some(13));
    assert_eq!(t([3, 3, 2, 2, 2, 2]).and_then(bewe::category), Some("14-18"));
    assert_eq!(t([3; 6]), Some(18));
}

#[test]
fn mapping_anatomique_de_chaque_sextant() {
    // Une dent de chaque sextant ; l'ordre visuel mandibulaire diffère de la numérotation.
    let expected = [
        (16, "upper_right"), (14, "upper_right"), (11, "upper_anterior"), (13, "upper_anterior"), (23, "upper_anterior"),
        (24, "upper_left"), (27, "upper_left"), (36, "lower_left"), (34, "lower_left"), (33, "lower_anterior"),
        (41, "lower_anterior"), (43, "lower_anterior"), (44, "lower_right"), (47, "lower_right"),
    ];
    for (tooth, s) in expected {
        assert_eq!(bewe::sextant_of_tooth(tooth), Some(s), "dent {tooth}");
    }
    assert_eq!(bewe::sextant_of_tooth(19), None);
}

#[test]
fn formats_historiques_supplementaires() {
    assert_eq!(bewe::parse_legacy("6-11 (9)").proposed_total, Some(9));
    assert_eq!(bewe::parse_legacy(" 12–18 (14) ").band, Some((12, 18)));
    assert_eq!(bewe::parse_legacy("7").flag, None);
    assert_eq!(bewe::parse_legacy("19").flag, Some("out_of_range"));
    assert_eq!(bewe::parse_legacy("1, 0, 2, 1, 1").flag, Some("unparseable"));
    assert_eq!(bewe::parse_legacy("6–11 (9) (8)").flag, Some("unparseable"));
}
