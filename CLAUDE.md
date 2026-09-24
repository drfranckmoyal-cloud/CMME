# Instructions du dépôt CMME

1. Lire `PROJET-MAITRE.md` en premier, le tenir à jour à chaque étape (tableau de bord, décisions, journal).
2. Puis `specs/CLAUDE_HANDOFF.md` (règles métier et sécurité non négociables) et `specs/docs/00_DECISIONS_FINALES.md`.
3. Les spécifications de `specs/` ne se modifient pas : ce sont les références d'origine.

## Organisation

- `crates/core` : domaine (BEWE, catalogue des champs, règles de manque, import, statistiques, export),
  stockage SQLCipher, sauvegarde. Tests : `cargo test -p cmme-core`.
- `src-tauri` : coquille Tauri, commandes typées exposées à l'interface.
- `src` : interface React/TypeScript. L'interface n'écrit jamais de SQL et n'affiche jamais « enregistré »
  sans réponse positive de la commande native.
- `research/` : scripts de recalcul à partir des seuls exports.
- `docs/` : guides, décisions, limites, résultats de recette.

## Règles

- Données fictives uniquement. Aucune donnée réelle dans le dépôt, les logs, les tests ou les captures.
- Commits en français, poussés sur `origin` à chaque étape qui tient debout.
