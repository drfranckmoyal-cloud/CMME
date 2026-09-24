# CMME — Dossier de transmission Claude Code

Version 1.0 · 23 septembre 2026 · Dr Franck Moyal

## Démarrer

1. Décompresser ce dossier dans l'espace de travail destiné au projet.
2. Ouvrir ce dossier dans Claude Code, ou le placer dans un dépôt existant.
3. Copier le contenu de `PROMPT_DEMARRAGE.md` dans Claude Code.
4. Pour examiner le design, ouvrir `design/Iris.html` ou `design/Lagune.html` dans un navigateur.

Le dossier contient le cadrage consolidé, les références visuelles et des données de test synthétiques. **Il ne contient pas encore l'application de production ni un installateur.** Les maquettes conservent leurs saisies uniquement pendant leur exécution. Aucune donnée réelle de patient n'est incluse.

## Ordre de lecture et priorité

1. `CLAUDE.md` : règles de travail et limites.
2. `docs/00_DECISIONS_FINALES.md` : décisions les plus récentes, prioritaires.
3. `docs/01_SPECIFICATION_CONSOLIDEE.md` : cadrage complet clinique, logiciel et scientifique.
4. `docs/02_CONTRATS_DE_DONNEES.md` et `data/dictionnaire_champs.json` : données et règles.
5. `docs/03_DESIGN_ET_PARCOURS.md` et le dossier `design/` : design et interactions.
6. `docs/04_ARCHITECTURE_ET_REALISATION.md` : architecture, lots, livrables.
7. `docs/05_IMPORT_ET_REPRISE.md` : historique, revue et import.
8. `docs/06_RECETTE.md` et `tests/cas_metier.json` : critères vérifiables.
9. `research/` : contexte de recherche et analyse rétrospective exploratoire déjà produite.

Les décisions explicites de Franck priment. Les documents métier priment sur les simplifications des maquettes. Le fichier de dictionnaire JSON est un catalogue, pas un schéma de validation prêt à exécuter. Les tests fournis sont des cas de recette, pas une suite déjà implémentée.

## Ce qui est acté

Application personnelle de recueil statistique, consultation de 10 à 15 minutes, usage sur un seul ordinateur, fonctionnement local et hors ligne. Une page continue : **Contexte → Habitudes → Expositions → Examen → Prévention → Observations**. Aucun document de fin de consultation. Import et analyse de l'historique principalement rétrospectif.

Les thèmes Iris et Lagune sont des propositions livrées ; **aucun choix définitif de palette n'a été exprimé**. Iris peut servir de thème initial de développement, Lagune doit rester disponible. macOS est la cible de travail proposée ; vérifier la version réelle du système et l'architecture de la machine avant de fixer l'installateur.

## Contenu sensible et données source

Le classeur historique et la fiche clinique originale contiennent des données de patients et ne sont pas inclus. L'importeur sera testé d'abord sur les fichiers synthétiques. Pour reprendre les données réelles, utiliser une copie locale autorisée, hors dépôt, sans la transmettre à un service d'IA ni la reproduire dans les logs.

Le rapport de `research/` contient des résultats agrégés et des coordonnées de lignes à vérifier. Il conserve un statut exploratoire ; il ne doit pas servir à forcer les résultats de l'application. Le fichier `MANIFEST.json` décrit les fichiers livrés et leurs empreintes.
