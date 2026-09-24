# Design et parcours

## Référence visuelle

Ouvrir `design/Iris.html` et `design/Lagune.html`. Les versions en page unique sont la référence la plus récente. Iris : navigation latérale prune, surfaces lilas et blancs doux, accent anis dans la marque. Lagune : navigation horizontale pétrole, surfaces ivoire et vert pâle, accent abricot. Les deux ne doivent pas imposer un changement du modèle de données.

Le design est une interface de travail soignée : marque discrète « cmme. », hiérarchie typographique, champs lisibles, regroupements sobres, absence d'illustrations décoratives. Ne pas confondre qualité commerciale avec une page marketing, de grands slogans ou des métriques sans utilité. Le nom de produit n'a pas fait l'objet d'un choix commercial définitif.

## Page de consultation

Toutes les rubriques sont montées et visibles dans le même document : Contexte, Habitudes, Expositions, Examen, Prévention, Observations. Navigation par ancres, pas par panneaux masqués. Les blocs de détail peuvent être repliables uniquement pour les compléments optionnels ; les données courantes restent accessibles directement. Dans les références livrées, les détails cliniques sont ouverts.

En-tête : code du dossier, date, service, statut brouillon/validé/amendé. Aucun âge ou diagnostic inventé pour remplir l'interface. Une nouvelle consultation démarre sans sélection clinique ; date du jour et praticien peuvent être proposés, car ce sont des métadonnées de saisie, avec modification possible.

Actions : Nouveau dossier, ouvrir un dossier, enregistrement automatique avec état exact, Terminer la saisie. Un bouton Terminer en bas est requis ; raccourci de sauvegarde clavier si utile. Terminer ne ferme pas un dossier contenant une erreur technique et ne génère aucun document. Les données manquantes ne bloquent pas le brouillon.

Le résumé BEWE reste proche du formulaire sur écran large ; présentation compacte sur petit écran. Ne pas donner une grande hauteur à la page uniquement pour aligner les variantes. Défilement naturel unique ; pas de formulaire dans une petite fenêtre scrollable. Les ancres doivent être accessibles au clavier et ne pas perdre la saisie.

## Saisie rapide

- Clavier : ordre naturel suivant les sections ; Tab et Shift-Tab fiables, focus visible, aucun piège.
- Zones numériques : unités visibles ; décimale française acceptée si pertinente ; sauvegarde non déclenchée comme validation définitive à chaque frappe partielle.
- Oui/non/inconnu : absence de présélection ; distinguer réponse inconnue et non recherchée.
- Menus multiples : cases à cocher, résumé des choix, fermeture sans effacement, champ libre associé.
- BEWE : six zones anatomiques sélectionnables, chiffre explicite et état manquant ; 0 distinct de « — » ; ne pas utiliser la couleur seule.
- Prévention : HBD en première position ; protocole et actions séparés ; bouton de confirmation explicite ou confirmation intégrée à la clôture clairement libellée.
- Observations : zone libre en fin de page, qui grandit raisonnablement ; aucune longueur arbitrairement faible.

## Tableau des dossiers

Une ligne par consultation par défaut, pas une confusion avec le nombre de patients. Colonnes principales : code, date et précision, service, âge ou classe, vomissements avec temporalité, BEWE et origine, prévention, statut/qualité. Une valeur absente est visible comme telle. Ajouter colonnes masquables et ordre mémorisé sans altérer les données.

Recherche locale par code ; identité seulement dans l'espace clinique si activée. Tri stable, filtres combinables par période, service, historique/prospectif, statut, BEWE manquant/≥9 et anomalies. Afficher l'effectif filtré et permettre de réinitialiser. Les filtres, tris et ouvertures ne doivent pas perdre un brouillon.

Ouverture d'une ligne : réhydrater tous les champs de cette visite, y compris textes, sélections multiples, statuts et raisons de manque. Ne jamais hériter des champs du dossier ouvert précédemment. Un retour au tableau retrouve filtres et position.

## Statistiques et import

Statistiques : effectifs patients/visites clairement séparés, N analysable pour chaque variable, distribution des classes BEWE, âge exact séparé des classes, taux de complétude. Les filtres sont affichés. Ne pas mélanger un graphique de données fictives et une base réelle.

Import : parcours dédié à aperçu → mapping → anomalies → validation → bilan. La consultation demeure sur une page ; cela n'interdit pas un assistant d'import. Revue à deux panneaux source/champs proposés, décisions traçables et réversibles.

## Accessibilité et états

Prévoir écran vide, chargement, erreur de sauvegarde, disque indisponible, sélection sans résultat, base verrouillée et données en cours d'import. Texte usuel 14–16 px ; labels au moins 12 px dans l'app finale, contraste vérifié. Cibles tactiles adaptées si utilisées ; priorité à l'ordinateur. Tester 1440×900 et 1280×800 ; vérifier reflow à 768 et 390 px sans troncature des actions. Respecter réduction des animations.

## Limites des HTML fournis à corriger dans l'app

1. Saisie en mémoire, sans chiffrement, sans persistance, sans verrouillage ni autosauvegarde réelle.
2. Données fictives présélectionnées uniquement pour montrer le design ; les supprimer dans les vrais dossiers neufs.
3. Certaines rubriques et sources manquent dans la maquette ; implémenter le dictionnaire, pas seulement les champs visibles.
4. Cases binaires d'examen et statut global de prévention simplifiés ; remplacer par états métier explicites et statuts par action.
5. BEWE dessiné comme six zones schématiques ; l'odontogramme final doit conserver le mapping, l'état non évaluable et l'accès clavier.
6. Les imports, exports, backups et restrictions d'accès ne sont pas implémentés dans ces HTML.
7. Polices et aides d'aperçu peuvent nécessiter Internet. Les assets de l'application finale doivent être embarqués localement.
8. Vérification syntaxique et structurelle effectuée sur les maquettes ; pas de validation visuelle automatisée dans un navigateur annoncée. Tester l'application finale dans un navigateur/WebView réel.
