//! Repère les dossiers aux identités proches (orthographe voisine, même nom de famille). Affiche des codes seulement.
use cmme_core::domain::legacy::identity_key;
use cmme_core::keystore::{KeyStore, KeychainStore};
use cmme_core::Store;

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

fn main() {
    let db = std::env::args().nth(1).expect("base");
    let key = KeychainStore { service: "fr.cmme.recueil".into() }.get("db-clinique").unwrap().unwrap();
    let s = Store::open(std::path::Path::new(&db), &key, false, "lecture").unwrap();
    let rows = s.all_rows(true).unwrap();
    let mut pats: Vec<(String, String, String)> = vec![];
    for r in &rows {
        if let Some(n) = &r.display_name {
            if !pats.iter().any(|p| p.0 == r.patient_id) {
                pats.push((r.patient_id.clone(), r.patient_code.clone(), identity_key(n)));
            }
        }
    }
    let mut n = 0;
    for i in 0..pats.len() {
        for j in i + 1..pats.len() {
            let (a, b) = (&pats[i].2, &pats[j].2);
            if a == b { continue; }
            let d = lev(a, b);
            let mut wa: Vec<&str> = a.split(' ').collect(); wa.sort();
            let mut wb: Vec<&str> = b.split(' ').collect(); wb.sort();
            if d <= 2 || wa == wb {
                n += 1;
                println!("{} ~ {}  (écart {} lettre(s){})", pats[i].1, pats[j].1, d, if wa == wb { ", mots inversés" } else { "" });
            }
        }
    }
    println!("{n} paire(s) d'identités proches");
}
