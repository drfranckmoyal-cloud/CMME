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
| 1–3 | Consultation page unique, BEWE, prévention, sauvegarde, amendements | Fait, vérifié à l'écran |
| 4 | Import XLSX/CSV, revue des anomalies, doublons | Fait (fichiers synthétiques) |
| 5 | Tableau filtrable, statistiques, export pseudonymisé figé | Fait |
| 6 | Sauvegarde/restauration chiffrée, verrouillage, build macOS | Fait ; .app + .dmg arm64 non signés |
| 7 | Guides, recette, livraison | Fait ; pilote réel et validation institutionnelle à venir |

Recette détaillée : `docs/RECETTE_RESULTATS.md`. Limites : `docs/LIMITES.md`. Guide : `docs/GUIDE_UTILISATEUR.md`.

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

- D5 (24/09) Cœur métier entièrement en Rust (validation à la frontière native) ; l'interface n'écrit jamais de SQL.
  Catalogue des champs unique (`crates/core/src/domain/catalog.rs`) pour écran, validation et export.
- D6 (24/09) Valeurs cliniques dans une table `field_value` typée (valeur XOR raison de manque, contrainte en base),
  tables dédiées pour sextants BEWE, expositions et mesures de prévention. Absence de ligne = « non renseigné »
  (prospectif) ou « non consigné » (historique).
- D7 (24/09) Clé de base aléatoire dans le trousseau macOS + mot de passe applicatif (Argon2id). En développement
  seulement, clé dans un fichier local (évite les demandes du trousseau à chaque recompilation ; données fictives).
- D8 (24/09) Sauvegarde = base SQLCipher autonome chiffrée par une phrase de récupération (restaurable sans le trousseau
  d'origine), vérifiée avant renommage atomique. Destinations iCloud (Bureau, Documents) refusées.
- D9 (24/09) « Vomissements » prospectif = jamais rapportés / anciens uniquement / actuels (+ détails). L'ancien oui/non
  reste un champ distinct à temporalité inconnue.
- D10 (24/09) Consultation historique importée = validée (révision 1) après revue ; correction par amendement.
- D11 (24/09) Export : liste blanche, année de consultation par défaut (dates exactes sur option), identifiants d'étude
  aléatoires par projet, instantané figé stocké dans la base chiffrée.

## Questions ouvertes

| Question | Qui tranche |
|---|---|
| Thème définitif Iris ou Lagune | Franck, quand il veut |
| Fenêtres de référence (28 j vomissements, 7 j habitudes) | Franck / équipe TCA |
| Destination autorisée des sauvegardes | Franck / DSI Sainte-Anne |
| Cadre institutionnel (DPO, MR-004) pour l'usage réel | Franck / établissement |
| Signature/notarisation Apple (compte développeur) si diffusion | Franck |

## Journal

- 24/09/2026 — Lecture complète du dossier de transmission. Inspection du Mac. Installation de Rust.
  Preuve SQLCipher : fichier illisible sans clé (test automatique). Création du dépôt.
- 24/09/2026 — V1 construite : 27 tests automatiques + test trousseau passent ; parcours vérifiés à l'écran
  (démo : saisie, BEWE, prévention, terminer, import CSV synthétique, tableau, statistiques, thèmes). Build release
  .app/.dmg arm64. Poussé sur GitHub.
- 24/09/2026 — Import du « Tableau recap consultations » dans l'espace clinique, à la demande et avec l'accord de
  Franck (noms déclarés pseudonymisés). Export XLSX dans `~/CMME-donnees` (hors iCloud), copie chiffrée de la base
  avant import au même endroit. Outil : `crates/core/examples/import_historique.rs` (règles conservatrices, décisions
  justifiées, aucune fusion). Résultat : 251 consultations historiques ; 9 lignes exclues (5 sans nom, 4 sans donnée
  clinique) ; 34 valeurs marquées indisponibles ; 2 groupes d'identités répétées laissés à l'arbitrage de Franck.
- 24/09/2026 — Corrections de Franck (code auto, sans IPP, date du jour JJ/MM/AAAA, sans type de visite ni source du
  diagnostic, Mérycisme + commentaire TCA, dentiste traitant, sans douleurs dentaires). Mot de passe d'ouverture
  désactivé à sa demande (réglage réversible, base toujours chiffrée). Blocage au démarrage corrigé (deux verrous
  internes pris l'un dans l'autre).
- 24/09/2026 — Intégration des 183 fiches Pages (2024-2025, 2025-2026), lues directement dans les fichiers .pages
  (`research/scripts/pages_en_texte.py`, vérifié identique à l'export Pages sur 56 fiches) puis
  `lire_fiches_pages.py` et `examples/integrer_fiches.rs`. Résultat : 135 consultations du tableau complétées
  (date, CAO, texte de la fiche…, amendement tracé), 5 rattachées malgré une orthographe voisine, 20 nouveaux
  dossiers, 9 conflits laissés à Franck (8 BEWE divergents, 1 nom porté par deux dossiers), 19 fiches écartées
  (18 « template copie » sans nom, 1 non remplie). Base : 273 consultations, 151 datées.
- 24/09/2026 — App installée dans /Applications et ajoutée au Dock. Franck confirme que la version finale s'ouvre.

## Garde-fous

- Développement uniquement sur données fictives (codes DEMO-, SYN-).
- Données réelles : jamais sous `~/Desktop` ni `~/Documents` (iCloud). L'application range ses bases
  dans `~/Library/Application Support`.
- Ne jamais annoncer un test ou un build non exécuté.
