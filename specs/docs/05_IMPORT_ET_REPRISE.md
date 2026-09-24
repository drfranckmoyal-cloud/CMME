# Import de l'historique

## Supports

Source principale connue : `Tableau recap consultations.numbers`. Une fiche Pages a fourni des détails de consultation. Ces originaux ne sont pas inclus dans le dossier de développement. Aucun parseur natif Numbers/Pages fiable n'est supposé. V1 : XLSX et CSV ; conversion locale vérifiée de Numbers, original conservé intact. La fiche Pages ne doit pas créer automatiquement une visite supplémentaire.

Le rapport exploratoire existant décrit une conversion locale suivie de lecture cellule par cellule. Il ne garantit pas qu'une nouvelle conversion aura exactement les mêmes coordonnées ; associer hash, feuille, table et ligne aux anomalies.

## Pipeline déterministe

1. Copier la source dans l'espace local chiffré, calculer SHA-256 ; conserver nom, date d'import, format et version du parseur.
2. Aperçu des feuilles/tables ; repérer en-têtes, synthèses, lignes séparatrices, lignes sans identité et cellules fusionnées. Le titre d'une feuille n'est qu'une proposition de service.
3. Mapper les colonnes ; préserver systématiquement la valeur brute et la localisation.
4. Proposer les valeurs normalisées et les flags. Aucune extraction clinique par IA dans la V1.
5. Revue des anomalies et doublons. Afficher source à gauche et proposition à droite. Accepter, corriger avec justification ou marquer indisponible.
6. Valider la transaction et produire bilan interne lignes retenues/exclues/en attente. Une validation partielle ne doit pas faire disparaître les lignes non résolues.

## BEWE historique

| Source fictive | Proposition | Traitement |
|---|---|---|
| `6–11 (9)` | exact 9, tranche 6–11 | Conserver brut, catégorie de référence 9–13 |
| `1–5 (7)` | exact proposé 7 | Flag tranche incompatible ; validation humaine avant jeu final |
| `0` | exact 0 | Score présent, pas manquant |
| `6–11` | tranche seulement | Pas de total exact, pas de point milieu |
| `6–11 (8 à 9)` | ambigu/plage | Pas de score exact |
| `(19)` | hors domaine | Aucun score numérique validé |
| cellule vide ou `◊` | indisponible | Raison de manque / non interprétable |

Extraire le nombre entre parenthèses uniquement avec une grammaire contrôlée et si un unique total entier 0–18 est identifiable. Un champ combinant plusieurs scores ou annotations contradictoires reste à revoir. Différencier le parsing syntaxique, le contrôle de cohérence et la validation d'étude.

## Dates, âges, expositions et identités

- Une date de séparation ne se propage pas aux lignes suivantes sans règle de bloc confirmée. Ne pas transformer une date suspecte en date plausible.
- Si une tranche d'âge comporte réellement un nombre exact entre parenthèses, garder ce nombre après validation du format. Sans nombre exact, conserver la tranche et ses bornes explicites ; aucun point milieu.
- Les fréquences en plages gardent min/max/unité. La précision de Franck sur les parenthèses ne justifie pas de traiter tout texte parenthétique comme numérique.
- Un « Non » dans l'ancienne colonne vomissements conserve `vomiting_legacy_code=no` avec temporalité inconnue. Il ne devient pas automatiquement « jamais rapportés ».
- Une catégorie alimentaire historique ne produit pas de sodas/agrumes individuels non documentés.
- Les protocoles historiques restent des libellés et des composantes inconnues. Pas de Tooth Mousse ou gouttières inventés.
- FDI ancienne colonne : exclue du recueil futur et des exports standard ; source brute conservée.
- Même identité normalisée = candidat à rapprochement. Fusion uniquement après revue ; distinguer homonymes, retours et duplications exactes. Fusion annulable et provenance préservée.

## Idempotence

Unicité source : hash du fichier + feuille + table + ligne. Réimport identique = aucune nouvelle visite. Nouvelle version du fichier = rapprochement et revue des différences, pas nouvelle cohorte par défaut. Conserver l'historique des lots d'import et la correspondance source–visite.

## Résultats connus, à ne pas coder en dur

L'analyse exploratoire du 23 septembre a trouvé 257 lignes nominatives, 245 dossiers cliniques provisoires et 239 BEWE exploitables après rapprochement provisoire. Ces nombres appartiennent à une version de la source et à des décisions de nettoyage, **pas à une constante de l'application**. Ils peuvent évoluer après validation d'identités, dates ou exclusions.

Le rapport de `research/` localise les anomalies déjà identifiées et expose ses règles. Le moteur d'import ne doit ni reproduire des fusions automatiques par nom ni forcer ces effectifs. Il doit permettre de revoir et justifier les décisions.
