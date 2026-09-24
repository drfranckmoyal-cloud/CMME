# Instructions du projet CMME

Construire une application de bureau utilisable, durable et soignée pour le recueil dentaire CMME. Le présent dossier est une transmission de spécifications ; les HTML sont des références interactives, pas le socle technique imposé.

## Travail attendu

- Lire les décisions et les contrats avant de coder. Inspecter le dépôt existant avant toute restructuration.
- Continuer les tâches réversibles sans redemander les arbitrages déjà fixés. Documenter les choix techniques ordinaires.
- Implémenter des parcours de bout en bout avec données fictives ; ne pas livrer un assemblage de boutons décoratifs.
- Ne jamais afficher « enregistré » avant confirmation durable du stockage. Une maquette en mémoire n'est pas une application persistante.
- Ne pas remplacer le design fourni par un tableau de bord générique. Préserver la lisibilité, les espaces et la navigation sur une page.
- À chaque lot : fichiers modifiés, fonctionnement disponible, vérifications exécutées, limites précises. Ne pas annoncer un test ou un build qui n'a pas été exécuté.

## Règles métier non négociables

- Six rubriques sur une page, Habitudes immédiatement après Contexte. Pas d'assistant étape par étape.
- Aucune valeur clinique initialement affirmative ou négative. Non renseigné n'est pas « non » et n'est pas zéro.
- Six scores BEWE anatomiquement identifiés ; total calculé seulement si tous sont évaluables et renseignés.
- Total historique exact séparé, sans reconstruction de sextants. Nombre entre parenthèses prioritaire dans les valeurs historiques avec tranche ; anomalie visible si incohérent.
- HBD est une action indépendante, placée au début de Prévention. Pas un protocole exclusif.
- Prévention = actes et conseils consignés, jamais prescription automatique selon le BEWE.
- Champs libres, nombres exacts, plages et provenance doivent être conservés fidèlement.
- Aucun compte rendu, ordonnance, courrier, impression ou fiche patient en fin de consultation.

## Données et sécurité

- Développement et démonstration sur données fictives uniquement.
- Ne pas inclure de noms de patients, documents cliniques, base réelle, texte libre clinique ou clés dans le dépôt, les prompts, les logs et les captures.
- Pas de cloud, API d'IA, télémétrie, synchronisation ou mise en ligne dans la V1.
- Chiffrement, sauvegarde et restauration sont des fonctions à implémenter et à vérifier ; ne pas simuler leur succès.
- Aucun remplacement silencieux d'une base chiffrée par du stockage en clair. Une difficulté de chiffrement doit être déclarée.
- Ne pas fusionner automatiquement deux dossiers à partir d'un nom identique.
- Utiliser des bibliothèques éprouvées pour la cryptographie ; ne pas inventer de chiffrement.

## Livraison

Fournir code source, versions et lockfiles, migrations, cas de test implémentés, procédure de lancement et de build, guide utilisateur, sauvegarde/restauration, limites connues et journal de décisions. Un binaire macOS doit être construit et testé sur un environnement compatible avant d'être annoncé comme disponible.
