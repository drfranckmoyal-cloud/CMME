# Données de développement

`dictionnaire_champs.json` est un catalogue extrait des tableaux de la spécification consolidée, enrichi des dernières décisions. Il conserve les expressions métier, dont certaines représentent plusieurs variables ou une table. **Ce n'est pas un JSON Schema prêt à valider les formulaires.** Les contrats du dossier docs fixent les états de manque, clés BEWE, actions de prévention et exports.

`import_synthetique.csv` utilise UTF-8 avec BOM et point-virgule. Les codes SYN sont fictifs. Il sert à tester mapping, zéro explicite, parenthèses, tranche seule, incohérence tranche/score, âge en classe et absence de réponse. Il ne contient aucune identité réelle et ne reproduit pas les effectifs de la patientèle.

Le jeu de démonstration des maquettes est différent du CSV d'import : ses totaux BEWE sont [0,3,11,2,14,5,null,7]. Ne pas confondre ce jeu destiné à l'interface avec la fixture de parsing ou les résultats rétrospectifs du dossier research.

Les jeux de production, correspondances d'identité et fichiers sources hospitaliers restent en dehors du dépôt et de ce pack.
