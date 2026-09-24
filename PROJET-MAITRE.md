# CMME — Fichier maître

Application de bureau de recueil clinique dentaire et de statistiques pour les consultations
TCA de la CMME (Sainte-Anne). Locale, hors ligne, un seul Mac. Porteur : Dr Franck Moyal.

Dépôt : `git@github.com:drfranckmoyal-cloud/CMME.git` (privé).
Dossier : `~/Desktop/Claude-Projects/CMME`.
Spécifications de départ : `specs/` (copie intacte du dossier de transmission V1.0 du 23/09/2026).

**À lire en premier par toute conversation qui reprend le projet**, puis `specs/docs/00_DECISIONS_FINALES.md`.

## Tableau de bord

| Lot | Contenu | État |
|---|---|---|
| 0 | Environnement, preuve SQLCipher | Fait (24/09/2026) |
| 1–3 | Consultation page unique, BEWE, prévention, sauvegarde, amendements | En cours |
| 4 | Import XLSX/CSV, revue des anomalies, doublons | À faire |
| 5 | Tableau filtrable, statistiques, export pseudonymisé figé | À faire |
| 6 | Sauvegarde/restauration chiffrée, verrouillage, build macOS | À faire |
| 7 | Guides, recette, livraison | À faire |

## Verrous (décisions de Franck, non renégociables)

- V1. Une seule page continue : Contexte → Habitudes → Expositions → Examen → Prévention → Observations.
- V2. Aucune valeur clinique présélectionnée ; « non renseigné » n'est ni « non » ni zéro.
- V3. BEWE : six sextants anatomiques ; total seulement si les six sont scorés ; total historique séparé.
- V4. HBD : case indépendante en tête de Prévention. Protocoles : Aucun / Modéré / Avancé / Personnalisé.
- V5. Modéré = dentifrice anti-érosion + bain de bouche anti-érosion après vomissement.
  Avancé = Modéré + Tooth Mousse quotidien (direct ou gouttières 10 min/j).
  Gouttières semi-rigides pendant les vomissements : option indépendante, jamais cochée d'office.
- V6. Aucun document (compte rendu, ordonnance, PDF, impression) en fin de consultation.
- V7. Aucune donnée réelle de patient dans le dépôt, les logs, les captures.
- V8. Pas de cloud, pas d'IA, pas de télémétrie. Tout reste sur le Mac, chiffré.
- V9. Thème Iris par défaut, Lagune disponible ; choix final ouvert (non bloquant).

## Décisions techniques

- D1 (24/09) Pile : Tauri 2 + React + TypeScript ; cœur métier en Rust (`crates/core`) ; base SQLCipher
  4.14 via rusqlite (`bundled-sqlcipher-vendored-openssl`). Tauri 3 écarté : encore en alpha.
- D2 (24/09) Mac cible vérifié : macOS 26.6.2, Apple M4 Pro (arm64), Xcode 26.5.
- D3 (24/09) Rust installé par l'installateur officiel rustup (profil minimal) dans `~/.cargo`.
- D4 (24/09) Le dossier `target/` (compilation) et `node_modules/` sont exclus d'iCloud
  (`xattr com.apple.fileprovider.ignore#P`), comme pour GEO.

## Questions ouvertes

| Question | Qui tranche |
|---|---|
| Thème définitif Iris ou Lagune | Franck, quand il veut |
| Fenêtres de référence (28 j vomissements, 7 j habitudes) | Franck / équipe TCA |
| Destination autorisée des sauvegardes | Franck / DSI Sainte-Anne |
| Cadre institutionnel (DPO, MR-004) pour l'usage réel | Franck / établissement |

## Journal

- 24/09/2026 — Lecture complète du dossier de transmission. Inspection du Mac. Installation de Rust.
  Preuve SQLCipher : fichier illisible sans clé (test automatique). Création du dépôt.

## Garde-fous

- Développement uniquement sur données fictives (codes DEMO-, SYN-).
- Données réelles : jamais sous `~/Desktop` ni `~/Documents` (iCloud). L'application range ses bases
  dans `~/Library/Application Support`.
- Ne jamais annoncer un test ou un build non exécuté.
