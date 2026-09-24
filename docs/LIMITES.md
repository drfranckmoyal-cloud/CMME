# Limites connues de la V1 (24/09/2026)

## À vérifier ou faire avant l'usage réel

- **Usage réel non autorisé en l'état** : la conservation locale de données hospitalières doit être validée par
  l'établissement (DSI/RSSI, DPO, qualification MR-004 du projet rétrospectif). L'application ne le règle pas.
- **Pilote 10–15 minutes** sur dix consultations : non fait.
- **Mode avion** : non testé formellement (aucune fonction réseau n'existe dans le code).
- **Version finale** : installée dans Applications, ouverture confirmée par Franck le 24/09/2026.
- **Import réel** : testé seulement sur fichiers synthétiques. Le mapping du vrai classeur (noms de colonnes, feuilles,
  formats de dates) sera à contrôler sur l'export XLSX, dans l'espace clinique.

## Simplifications assumées

- Traitements : champ texte (pas encore de table répétable nom/classe/dates).
- Constats (caries, usures, muqueuses) : un état par domaine ; pas encore de localisation dent par dent ni de
  source par constat (endobuccal/panoramique).
- BEWE : pas de « dent la plus atteinte » ni de surface par sextant.
- Statuts de prévention : plusieurs cases cumulables par mesure, sans horodatage séparé de chaque événement.
- Orientation : une seule orientation par consultation.
- Fréquences d'exposition : exacte ou plage, par jour/semaine/mois ; durée d'exposition non structurée.
- Fusion de dossiers : proposée pour les identités identiques ; pas de rapprochement orthographique approché.
- Nouvelle version d'un fichier importé : une ligne modifiée peut être ignorée ou créée comme nouvelle visite ;
  la mise à jour d'une visite existante se fait par « Corriger » (amendement).
- Petites cellules : aucune suppression automatique dans les exports (à gérer avant toute diffusion externe).
- Verrouillage : minuterie d'inactivité et détection de veille par saut d'horloge ; pas de Touch ID.
- Reprise d'une frappe non enregistrée après fermeture forcée : non garantie (seul le dernier état confirmé l'est).
- Écrans : conçus pour 1280–1440 px ; en dessous de 768 px, l'affichage se réorganise mais n'a pas été vérifié.
- Mac Intel : non construit.

## Hors V1 (décidé)

Cloud, multi-utilisateur, iPhone, comptes rendus, ordonnances, photos, mesures salivaires, questionnaires validés,
IA, recherche automatique de significativité.
