//! Vérification réelle du trousseau macOS (lancée à la main : cargo test --test trousseau -- --ignored).

use cmme_core::keystore::{generate_key, KeyStore, KeychainStore};

#[test]
#[ignore]
fn trousseau_macos_ecrire_lire_supprimer() {
    let ks = KeychainStore { service: "fr.cmme.recueil.test".into() };
    let k = generate_key().unwrap();
    ks.set("essai", &k).unwrap();
    assert_eq!(ks.get("essai").unwrap(), Some(k));
    ks.delete("essai").unwrap();
    assert_eq!(ks.get("essai").unwrap(), None);
}
