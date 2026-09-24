# CMME — Guide d'utilisation

## Premier lancement

1. Ouvrir **CMME Recueil**. Choisir « Créer l'espace clinique » : votre nom et un mot de passe (8 caractères au moins).
   Une base chiffrée est créée sur ce Mac ; sa clé est rangée dans le trousseau macOS.
2. Faire aussitôt une **sauvegarde chiffrée** (menu Sauvegarde) sur une clé USB, avec une phrase de récupération
   notée hors du Mac. Sans cette phrase, une sauvegarde ne peut pas être relue.
3. Le bouton « Ouvrir la démonstration » ouvre une base séparée, entièrement fictive, pour s'exercer.

## Pendant la consultation

- « Nouveau dossier » : code attribué automatiquement (ou saisi), identité facultative (jamais exportée).
- Une seule page : Contexte, Habitudes, Expositions, Examen, Prévention, Observations. Les onglets du haut font
  défiler la page, ils ne cachent rien.
- Rien n'est coché d'avance. Un clic sur une réponse déjà choisie l'efface. Le bouton « ••• » d'un champ permet de
  noter *Inconnu*, *Refus*, *Non applicable* ou *Non évaluable* : ce n'est jamais un « non ».
- Nombres : virgule acceptée ; « plage » pour saisir « 2 à 3 » sans le transformer en 2,5.
- **BEWE** : pour chaque sextant, 0 à 3 ou *N.É.* (non évaluable, avec motif). Au clavier : Tab jusqu'au sextant puis
  0–3, N, ou Effacement. Le total n'apparaît que si les six sextants sont scorés.
- **Prévention** : la case *Enseignement HBD* est indépendante. Choisir Modéré ou Avancé *prépare* les mesures ;
  elles ne comptent comme conseillées qu'après « Confirmer les mesures conseillées ». Les gouttières semi-rigides
  pendant les vomissements se choisissent à part.
- En haut à droite : « Enregistré à … » n'apparaît qu'une fois l'écriture confirmée. En cas d'échec, un bandeau
  rouge le dit et propose « Réessayer » ; vos saisies restent à l'écran.
- **Terminer la saisie** : récapitulatif (BEWE, manques, mesures non confirmées), puis verrouillage.
  **Aucun document n'est produit.** Pour corriger ensuite : « Corriger » avec un motif ; l'ancienne version reste
  consultable (« Historique »).

## Reprendre l'ancien tableau

1. Dans Numbers : Fichier › Exporter vers › Excel. Garder l'original intact. Enregistrer l'export **hors du Bureau et
   des Documents** (iCloud), par exemple dans `~/DentalLens` ou sur une clé.
2. Menu **Reprise** › « Importer un fichier… ». Vérifier pour chaque feuille : une ligne = une consultation, ou
   tableau de synthèse (jamais de patient), ou ignorer ; la ligne d'en-tête ; la correspondance des colonnes.
3. « Préparer l'import » : rien n'est encore validé. Les lignes sans anomalie peuvent être validées en lot.
4. Les autres se revoient une par une : source brute à gauche, propositions à droite. *Accepter*, *Corriger*
   (justification obligatoire) ou *Indisponible*. Exemple « 1–5 (7) » : 7 est proposé, l'anomalie reste visible.
5. Doublons possibles : choisir « Même patient que… », « Personne différente » ou « Dossier distinct ».
   Aucune fusion n'est automatique ; une fusion se défait (Reprise › Doublons et fusions).
6. Réimporter le même fichier est refusé. Une nouvelle version du fichier : lignes identiques reconnues, lignes
   modifiées à revoir.

## Dossiers, statistiques, étude

- **Dossiers** : une ligne par consultation, filtres (période, service, historique/prospectif, statut, BEWE,
  anomalies), colonnes masquables.
- **Statistiques** : mêmes filtres ; patients et consultations séparés ; chaque proportion avec son N et son IC 95 %.
- **Étude et exports** : créer un projet (période, recueils inclus, version du plan), lire le diagramme de sélection
  (une visite index par patient), retirer l'éligibilité d'un dossier si besoin, puis « Figer une extraction » et
  « Écrire dans un dossier… ». Les fichiers CSV sont en clair : les ranger dans un emplacement autorisé.
  Recalcul indépendant : `python3 research/scripts/rapport_descriptif.py <dossier>`.

## Sécurité au quotidien

- Verrouillage automatique après 10 minutes (réglable) et au retour de veille ; bouton cadenas pour verrouiller.
- Restauration : Sauvegarde › Restaurer (vérification, copie de sécurité de l'état actuel, puis remplacement).
- Nouveau Mac : installer l'application, « Restaurer une sauvegarde », phrase de récupération, puis le mot de passe
  de l'application en vigueur à la date de la sauvegarde.
