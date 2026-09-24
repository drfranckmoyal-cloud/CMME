# Recette de livraison

Les cas de `tests/cas_metier.json` sont synthétiques. Ils doivent devenir des tests de domaine ; ce dossier ne prétend pas qu'ils sont déjà exécutés contre une application.

## Parcours fonctionnels

1. Créer un dossier vide : six rubriques visibles dans l'ordre convenu, aucune réponse clinique inventée.
2. Atteindre Habitudes depuis le raccourci : défilement, Contexte toujours dans la page, saisies préservées.
3. Saisir une consultation partielle, fermer après confirmation de sauvegarde et rouvrir : tous les champs retrouvés.
4. Provoquer une erreur disque : aucun faux « enregistré », brouillon conservé et possibilité de réessayer.
5. Ouvrir A, puis B, puis A : aucune valeur de B dans A ; multisélections et textes fidèles.
6. Tous les seuils BEWE et le mapping mandibulaire sont corrects. Total incomplet = null, jamais somme partielle affichée comme totale.
7. HBD seul avec Aucun protocole proposé est enregistrable et analysable distinctement.
8. HBD coché, puis Modéré, puis Avancé : HBD conservé ; gouttières pendant vomissement toujours non renseignées.
9. Tooth Mousse direct : pas de durée en gouttières appliquée par erreur ; route gouttière : proposition 10 min modifiable.
10. Ancien protocole avancé : composantes non reconstruites.
11. Choisir plusieurs boissons et aliments ; ajouter textes et fréquences ; décocher sans perte des autres choix. « Aucun » et catégories positives exclusifs.
12. Terminer avec observations libres : stockage fidèle, aucun PDF/ordonnance/dialogue d'impression.
13. Modifier une visite validée : amendement avec motif, ancien état accessible ; export figé précédent inchangé.

## Import et statistiques

14. Importer la source synthétique : tranches, valeurs exactes, zéro, manque, intervalle et hors-domaine distingués.
15. Réimporter le même fichier : aucune duplication. Changer une valeur dans une nouvelle version : revue du changement.
16. Deux homonymes : pas de fusion automatique. Fusion manuelle erronée annulable.
17. Les dates suspectes, lignes déplacées et synthèses ne deviennent pas silencieusement des visites valides.
18. Filtrer tableau/statistiques : mêmes critères, N affichés, individus et visites distincts, historique/prospectif identifiables.
19. Zéro résultat ou zéro score disponible : afficher « aucune donnée analysable », pas 0 % présenté comme résultat.
20. Pour le jeu démo BEWE [0,3,11,2,14,5,null,7] : N=7 ; médiane=5 ; catégories 0–2=2, 3–8=3, 9–13=1, 14–18=1 ; ≥9=2/7.
21. Export depuis liste blanche : aucune identité, note, chemin local ou texte libre inattendu ; injection de formule neutralisée.
22. Chaque export contient dictionnaire, manifest, filtres, versions, dénominateurs et hashes. Recalcul externe identique.

## Fiabilité, sécurité et interface

23. Ouvrir la base sans clé appropriée : pas de contenu clinique en clair. Vérifier également journaux et fichiers temporaires.
24. Sauvegarder, modifier une base test, restaurer : retrouver exactement l'état sauvegardé. Tester la récupération avec un profil système vierge.
25. Mauvais mot de passe, backup tronqué, migration échouée : état courant intact, erreur compréhensible.
26. Déconnecter le réseau : consultation, tableau, import local, export et backup fonctionnent ; aucune requête distante nécessaire.
27. Navigation complète au clavier ; focus visible ; absence de débordement des actions aux largeurs prévues ; BEWE lisible sans couleur.
28. Un pilote utilisateur confirme la faisabilité du recueil pendant 10–15 minutes. Ne pas prétendre valider ce temps par un test automatisé.

## Conditions pour déclarer la V1 utilisable

Les parcours et invariants ci-dessus sont implémentés et vérifiés ; le build et les limites de plateforme sont documentés ; sauvegarde/restauration démontrées ; mode démo séparé ; aucune donnée réelle dans le dépôt. Le rapport de livraison indique les tests passés, échoués et non exécutables. Ne pas déclarer l'application utilisable sur données réelles en présence d'une simulation de persistance ou de chiffrement.
