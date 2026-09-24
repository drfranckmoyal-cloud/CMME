# Installation, construction, mise à jour

## Environnement vérifié

macOS 26.6.2, Apple M4 Pro (arm64), Xcode 26.5, Node 24.18, npm 11.16, Rust 1.98.1 (rustup, profil minimal).
Versions exactes des bibliothèques : `Cargo.lock` et `package-lock.json`.
Pile : Tauri 2.11, React 19, Vite 8, TypeScript 7, SQLCipher 4.14.0 (via rusqlite 0.40, OpenSSL embarqué),
calamine (XLSX), keyring 3 (trousseau macOS), argon2.

## Construire

```bash
cargo test -p cmme-core
```

```bash
npx tauri build
```

Produits : `target/release/bundle/macos/CMME Recueil.app` et `target/release/bundle/dmg/CMME Recueil_1.0.0_aarch64.dmg`.
Minimum déclaré : macOS 13. Architecture : Apple Silicon uniquement (un Mac Intel nécessiterait une construction
`x86_64` ou universelle, non faite).

**Signature** : signature *ad hoc* seulement (pas de compte développeur Apple, pas de notarisation). Au premier
lancement depuis le .dmg, macOS peut refuser l'ouverture : clic droit sur l'app › Ouvrir, ou Réglages Système ›
Confidentialité et sécurité › « Ouvrir quand même ». Aucun identifiant de signature n'a été inventé.

## Installer

Ouvrir le .dmg, glisser **CMME Recueil** dans Applications. Les données vont dans
`~/Library/Application Support/fr.cmme.recueil/` (hors iCloud) : `clinique/` et `demo/`, chacun chiffré.

## Mettre à jour

Faire une sauvegarde, remplacer l'app. Le schéma de base est versionné ; une base créée par une version plus récente
est refusée plutôt qu'altérée.

## Désinstaller

Supprimer l'app. Les données restent dans le dossier ci-dessus et la clé dans le trousseau (élément
« fr.cmme.recueil ») : ne les supprimer qu'après une sauvegarde vérifiée.

## Développement

`npx tauri dev` (profil de développement : clés dans un fichier local `dev-keys/`, **données fictives uniquement**).
`npx tauri build --debug --bundles app` produit une app de test avec le même comportement de développement.
