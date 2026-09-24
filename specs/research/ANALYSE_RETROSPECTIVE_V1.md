# CMME — Première analyse rétrospective des consultations dentaires

Analyse du 23 septembre 2026 — version exploratoire V1

**Résultat principal : 239 scores BEWE numériques sont exploitables dans 245 dossiers cliniques après regroupement provisoire des identités répétées. Parmi ces scores, 155/239 (64,9 %) sont supérieurs à zéro et 53/239 (22,2 %) sont au moins égaux à 9.**

Ces résultats décrivent les dossiers transmis, après les règles de nettoyage ci-dessous. Ils ne constituent pas encore les résultats définitifs d’une étude publiée. Les rapprochements d’identité, le périmètre de recrutement, les dates et quelques anomalies doivent être confirmés. Aucun nom de patient n’est reproduit dans ce rapport.

## 1. Source et méthode

Le fichier « Tableau recap consultations.numbers » a été converti localement en XLSX avec LibreOffice, puis lu cellule par cellule. Le classeur original a été conservé intact. Le tableau de synthèse vide n’a pas été compté comme des patients. Les lignes vides et séparateurs de date ont été écartés du décompte des dossiers. Le document Pages fourni a servi au cadrage des rubriques, sans être ajouté comme une consultation supplémentaire ni utilisé pour modifier les valeurs du tableau.

Un nom normalisé identique (casse, accents, espaces et ponctuation neutralisés) constitue ici une identité provisoire. Aucune fusion approximative de noms n’a été appliquée. Cette méthode ne remplace pas la vérification par identifiant hospitalier et ne garantit pas l’absence d’homonymes ou de variantes non rapprochées.

Trois identités se répètent chacune deux fois. Deux associent une ligne sans données cliniques à une ligne renseignée. La troisième correspond à deux lignes dont les variables cliniques communes sont identiques, y compris le BEWE, l’âge, le CAO, les expositions et l’observation : elle n’est comptée qu’une fois dans les résultats principaux. Les sources sont conservées ; aucun score discordant n’a été départagé arbitrairement.

Cinq lignes sans nom ont été exclues. Elles présentent un bloc de valeurs correspondant à des lignes du Centre Expert et paraissent être des copies. Elles n’ont pas été transformées en cinq patients supplémentaires. Cette exclusion reste à confirmer sur le classeur.

## 2. Effectifs et disponibilité

| Étape | Nombre |
|---|---:|
| Lignes nominatives, avant rapprochement | 257 |
| Lignes cliniques sans nom, exclues | 5 |
| Identités normalisées distinctes | 254 |
| Identités sans aucune donnée clinique exploitable | 9 |
| Dossiers avec au moins une donnée clinique | 245 |
| Dossiers avec BEWE numérique exploitable | 239 |
| Dossiers cliniques sans BEWE exploitable | 6 |

**Le dénominateur principal est donc 239 pour le BEWE, et non 257 ou 245.** La disponibilité du score est de 239/245, soit 97,6 %, parmi les dossiers cliniques provisoirement retenus. Les 9 identités sans données cliniques peuvent correspondre à des inscriptions, consultations non renseignées ou autre situation : leur statut ne peut pas être établi à partir du fichier seul.

Répartition par feuille avant rapprochement des doublons :

| Feuille | Lignes nominatives | Avec donnée clinique | Avec BEWE numérique |
|---|---:|---:|---:|
| Feuille 2 | 68 | 68 | 68 |
| Centre Expert | 155 | 145 | 141 |
| SAS | 9 | 9 | 9 |
| Hospit longue durée | 19 | 19 | 18 |
| HDJ | 3 | 2 | 2 |
| HDJ Intensif | 3 | 3 | 2 |

La somme des scores de ce tableau est 240 avant dédoublonnage ; un score présent deux fois ramène le total principal à 239. Le nom de la feuille décrit la source, pas nécessairement un groupe de recrutement indépendant.

## 3. BEWE : distribution et fréquence

Conformément à la précision de Franck, le nombre entre parenthèses a été retenu comme score historique exact. Les cellules contenant directement un nombre ont également été utilisées. Parmi les 239 scores, 126 viennent du nombre entre parenthèses et 113 d’une cellule numérique. Aucun score n’a été remplacé par le milieu d’une tranche ; aucun sextant manquant n’a été reconstruit.

| Indicateur | Valeur |
|---|---:|
| Médiane [Q1–Q3] | 4 [0–7] |
| Moyenne ± écart-type | 4,62 ± 4,83 |
| Étendue | 0–18 |
| Score nul | 84/239 — 35,1 % |
| BEWE > 0 | 155/239 — 64,9 % ; IC 95 % 58,6–70,6 % |
| BEWE ≥ 9 | 53/239 — 22,2 % ; IC 95 % 17,4–27,9 % |
| BEWE ≥ 14 | 16/239 — 6,7 % ; IC 95 % 4,2–10,6 % |

Catégories recalculées à partir du total précis selon les seuils de la publication BEWE de référence [1] :

| Total BEWE | Effectif | Part des 239 scores |
|---|---:|---:|
| 0-2 | 102 | 42,7 % |
| 3-8 | 84 | 35,1 % |
| 9-13 | 37 | 15,5 % |
| 14-18 | 16 | 6,7 % |

Un score >0 est ici un indicateur descriptif de lésions cotées, pas un seuil de maladie nécessitant un traitement. La classe 0–2 ne signifie pas nécessairement absence de toute lésion. Les catégories ne constituent pas une prédiction individuelle de progression.

## 4. Comparaison selon la colonne « Vomissements »

Les résultats ci-dessous utilisent exactement le codage oui/non du tableau. Sa fenêtre temporelle n’est pas standardisée. Il ne faut donc pas traduire « non » par « aucun vomissement au cours de la vie ».

| Indicateur | Codage « Oui » | Codage « Non » |
|---|---:|---:|
| Dossiers cliniques | 129 | 115 |
| Avec BEWE exploitable | 128 | 110 |
| BEWE médian | 5 | 0 |
| BEWE moyen | 6,67 | 2,28 |
| BEWE >0 | 108/128 — 84,4 % | 47/110 — 42,7 % |
| BEWE ≥9 | 45/128 — 35,2 % | 8/110 — 7,3 % |
| BEWE ≥14 | 14/128 — 10,9 % | 2/110 — 1,8 % |

Un dossier possède un BEWE à zéro mais pas de codage vomissements interprétable ; il figure dans le descriptif global, pas dans cette comparaison.

**Le contraste descriptif est marqué.** Le rapport de proportions non ajusté de BEWE >0 est de 1,97 (IC 95 % approximatif 1,57–2,48). Pour BEWE ≥9, il est de 4,83 (2,38–9,81). Ces rapports comparent les proportions observées ; ce ne sont ni des risques de développer une lésion au cours du temps, ni des effets causaux. Les intervalles reposent sur une approximation logarithmique, sous hypothèse d’observations indépendantes après le rapprochement provisoire.

Les données actuelles n’autorisent pas à isoler l’effet des vomissements de l’ancienneté du TCA, de l’âge, du reflux, des médicaments ou du mode de recrutement. Aucun modèle multivariable n’a été ajusté pour cette première analyse.

Une hétérogénéité selon la source apparaît : dans Feuille 2, le BEWE médian est 5,5 chez les « Oui » contre 5 chez les « Non » ; dans Centre Expert, il est respectivement 5 et 0. Cela justifie de vérifier période, sélection et méthode de recueil avant de présenter une association globale comme uniforme dans le service.

## 5. Alimentation et boissons : signal exploratoire

Les deux formulations « Avec risque érosif » et « Avec risque érosif associé » ont été regroupées. « Risque carieux élevé » n’a pas été assimilé à une exposition érosive.

| Codage alimentaire | BEWE disponible | BEWE médian | BEWE ≥9 |
|---|---:|---:|
| Avec risque érosif | 66 | 6 | 24/66 — 36,4 % |
| Sans risque érosif associé | 171 | 2 | 27/171 — 15,8 % |

Deux autres scores appartiennent à des dossiers dont le codage alimentaire ne permet pas cette comparaison. Ce signal reste exploratoire : la catégorie repose sur un jugement clinique dont la définition et l’indépendance vis-à-vis de l’examen dentaire doivent être précisées.

Croisement descriptif des deux codages :

| Vomissements | Risque alimentaire codé | N avec BEWE | Médiane | BEWE ≥9 |
|---|---|---:|---:|---:|
| Oui | Avec | 50 | 7,0 | 21/50 — 42,0 % |
| Oui | Sans | 76 | 5,0 | 22/76 — 28,9 % |
| Non | Avec | 16 | 2,5 | 3/16 — 18,8 % |
| Non | Sans | 94 | 0,0 | 5/94 — 5,3 % |

Le sous-groupe « Non » + risque alimentaire comporte seulement 16 scores. Il ne permet pas de conclure à lui seul à un mécanisme d’érosion sans vomissements. Avant une publication dédiée, il faut revoir l’histoire des vomissements, le reflux/régurgitations et les expositions anciennes.

Parmi les huit dossiers codés « Non » avec BEWE ≥9, les observations incluent notamment des mentions de reflux possible, de régurgitation/mérycisme possible ou d’hyperémèse ancienne. Ces mentions ne sont pas des diagnostics confirmés ; elles illustrent la nécessité de séparer les sources et périodes d’exposition. Les dossiers sans ces mentions ne constituent pas pour autant un groupe négatif pour ces facteurs.

## 6. Autres données déjà mobilisables

### Profil

- 177 dossiers ont un âge numérique : médiane 26 ans, Q1–Q3 21–32, étendue 18–57 ans dans ce sous-ensemble.
- Les 68 autres ont seulement une classe d’âge. Six sont classés « <20 » : on ne peut pas affirmer que toute la population est adulte.
- La colonne « Sexe » comporte 235 F, 8 H, 1 NB et 1 valeur absente. Ces modalités sont rapportées telles quelles ; elles ne représentent pas une distinction standardisée entre sexe et identité de genre.

### Hygiène

| Libellé | Nombre |
|---|---:|
| Suffisante | 186 |
| Insuffisante | 35 |
| Iatrogène | 21 |
| Valeur absente | 2 |
| Valeur hors rubrique à vérifier | 1 |

Les jugements d’hygiène sont presque complets mais leur définition doit être explicitée. Ils ne remplacent pas fréquence de brossage, dureté de brosse, fluor ou comportement après vomissement.

### Soins et prévention

La rubrique soins compte 236 valeurs interprétables : 110 « suivi spécialisé », 74 « suivi non spécialisé », 36 « soins spécialisés » et 16 « soins non spécialisés » (après harmonisation du singulier/pluriel). Huit sont absentes et une ressemble à un score BEWE mal placé. Les 36 libellés « soins spécialisés » représentent 15,3 % des 236 valeurs interprétables, mais il faut confirmer s’il s’agit d’un besoin, d’une proposition ou d’un soin réalisé.

La prévention est codée dans 238/245 dossiers : 89 protocoles avancés, 69 modérés et 80 enseignements d’hygiène bucco-dentaire. On peut décrire les actions consignées, mais ni leur réalisation certaine ni leur efficacité à partir de ces seuls codes.

La rubrique « autres usures » donne 48 réponses positives sur 228 oui/non textuels interprétables (21,1 %). Les valeurs 0, C, absentes et celles d’un bloc décalé n’ont pas été transformées arbitrairement en non. Ce résultat ne constitue pas une prévalence validée du bruxisme : les signes d’usure mécanique, le serrement déclaré et le diagnostic du bruxisme sont différents.

### CAO

229 valeurs de CAO directement numériques : médiane 2, Q1–Q3 0–7, étendue 0–28 ; 125/229 sont >0. Les autres cellules ont été laissées à revoir. Le CAO est un indicateur cumulé et ne mesure pas, à lui seul, les caries actives ni le besoin actuel de traitement. Vérifier le sens historique de la composante A avant toute interprétation scientifique.

## 7. Complétude réelle : présence et validité

| Domaine | Disponible/interprétable sur 245 | Commentaire |
|---|---:|---|
| BEWE exact | 239 | Très bonne disponibilité ; exceptions identifiées |
| Vomissements oui/non | 244 | Temporalité à préciser |
| Âge exact | 177 | 68 classes seules |
| Risque alimentaire binaire | 241 | 1 codage carieux et 3 absences exclus du binaire |
| Hygiène dans une des trois catégories | 242 | 2 absences, 1 modalité hors rubrique |
| Soins | 236 | Signification clinique des libellés à préciser |
| Prévention | 238 | Contenu des protocoles à définir |
| CAO directement numérique | 229 | Ne comprend pas les valeurs textuelles composites |

Classification TCA, durée de maladie, vomissements anciens, reflux, traitements, sécheresse buccale, profession, tabac, alcool, autres substances et habitudes détaillées sont parfois présents en texte libre mais n’ont pas encore fait l’objet d’une abstraction systématique. Aucun taux d’absence de ces expositions ne peut être calculé à partir du silence des notes. Le contenu libre du classeur n’a pas été recodé exhaustivement pour cette analyse.

## 8. Anomalies localisées à vérifier

Les numéros ci-dessous sont ceux des lignes du tableau converti, ligne d’en-tête comprise ; le nom de feuille permet de retrouver la source. Les intitulés de table peuvent différer légèrement dans Numbers.

| Localisation | Point à vérifier | Traitement dans cette analyse |
|---|---|---|
| Feuille 2, ligne 56 | BEWE « 1–5 (7) » | Total 7 retenu selon la convention donnée ; anomalie conservée |
| Feuille 2, ligne 64 | BEWE 4 et texte « 1–5 (3) » dans Soins | Score 4 de la colonne BEWE retenu ; soins non interprétables ; analyse de sensibilité sans ce dossier |
| Centre Expert, ligne 117 | « Avec risque érosif » dans Hygiène | Hors catégories d’hygiène |
| Centre Expert, lignes 191–197 | Prévention répétée et colonnes auxiliaires déplacées | BEWE et colonnes stables conservés ; autres usures/observations non recodées automatiquement |
| Hospit longue durée, lignes 36–40 | Cinq lignes sans nom, ressemblant à un bloc du Centre Expert | Exclues des effectifs |
| Centre Expert 38 et SAS 4 | Même identité et variables cliniques communes identiques | Comptées une fois ; sources conservées |
| Centre Expert 81 et 158 | Même identité, première ligne sans examen | Une identité ; seule ligne clinique utilisée, sans prétendre dater une première consultation |
| Centre Expert 203 et SAS 12 | Même identité, première ligne sans examen | Une identité ; seule ligne clinique utilisée |
| Centre Expert, ligne 76 | Date décodée 05/01/2025 au milieu de blocs 2025–2026 | Non corrigée ; pas d’analyse chronologique |
| Centre Expert, ligne 105 | Date décodée 01/11/2002 | Non corrigée ; vérifier date et format dans Numbers |
| Centre Expert 65, 119, 152, 165 ; Hospit longue durée 33 ; HDJ Intensif 9 | BEWE absent ou symbole non numérique | Exclus du dénominateur BEWE |

La présence de dates en séparateurs ne prouve pas à elle seule qu’elles s’appliquent à toutes les lignes suivantes. La période exacte d’inclusion ne doit pas être publiée avant validation de cette convention et des anomalies.

## 9. Robustesse des chiffres principaux

| Variante | N scores | BEWE >0 | BEWE ≥9 | Médiane |
|---|---:|---:|---:|---:|
| Analyse principale | 239 | 155/239 — 64,9 % | 53/239 — 22,2 % | 4,0 |
| Exclusion de toutes les identités répétées | 236 | 152/236 — 64,4 % | 51/236 — 21,6 % | 4,0 |
| Exclusion des deux anomalies BEWE/colonne soins | 237 | 153/237 — 64,6 % | 53/237 — 22,4 % | 4,0 |
| Toutes les lignes nominatives, sans dédoublonnage | 240 | 156/240 — 65,0 % | 54/240 — 22,5 % | 4,0 |

Ces variantes montrent que les quelques doublons et anomalies repérés ne modifient pas fortement le portrait global. Elles ne corrigent pas les biais de sélection ni les erreurs possibles de mesure ou d’histoire clinique.

## 10. Ce que cette base permet de publier

**Projet le plus réaliste :** une étude rétrospective transversale décrivant les scores BEWE, leur distribution selon les expositions documentées et les besoins/orientations de soins dans le parcours CMME.

Titre de travail : *Erosive tooth wear and documented dental care needs in patients with eating disorders attending a specialist hospital service: a retrospective cross-sectional study.*

L’effectif disponible permet d’envisager sérieusement ce projet. La publication dépendra de la qualité du protocole, du recrutement, de la mesure et de la pertinence du message, pas du seul nombre de dossiers.

Le message scientifique provisoire peut être formulé ainsi : une part importante des patients dépistés présente un score d’usure érosive non nul ; les scores sont plus élevés dans le groupe codé avec vomissements, mais des scores non nuls existent aussi parmi les patients codés sans vomissements. La seconde observation appelle une caractérisation des expositions anciennes et des autres sources acides, plutôt qu’une conclusion immédiate sur une érosion indépendante des vomissements.

**À ne pas revendiquer actuellement :** incidence, progression, efficacité de la prévention, causalité, comparaison de sous-types TCA diagnostiques, prévalence représentative de tous les TCA, ou absence de vomissements au cours de la vie dans le groupe « Non ».

## 11. Suite prioritaire

1. Valider les trois rapprochements d’identité, le bloc sans nom et les deux anomalies BEWE.
2. Confirmer le sens des feuilles, la période, l’exhaustivité du dépistage et les dates de blocs.
3. Revoir en priorité les huit dossiers codés « Non » avec BEWE ≥9, puis établir un manuel d’abstraction identique pour tous les dossiers. Une revue ciblée seule ne doit pas créer un biais de recherche des facteurs chez les seuls sujets sévères.
4. Récupérer si disponibles dans les sources autorisées : diagnostic TCA, ancienneté, histoire des vomissements, reflux/régurgitations, traitements et expositions alimentaires. Ne pas reconstruire les réponses manquantes par supposition.
5. Définir les catégories de besoins de soins et le contenu des protocoles de prévention.
6. Arrêter un protocole et un plan d’analyse datés ; distinguer les hypothèses explorées ici de celles testées ultérieurement ; valider le cadre institutionnel avant la réutilisation pour publication.
7. Figer l’extraction, préparer les tableaux STROBE/RECORD et discuter un ajustement limité avec un méthodologiste si la qualité des covariables le permet.

## Annexe : conventions statistiques et traçabilité

- Dénominateurs disponibles propres à chaque variable ; aucune imputation des manques.
- Quartiles calculés par interpolation linéaire (méthode NumPy par défaut) ; médiane et moyenne calculées sur les seuls scores exacts.
- Écart-type d’échantillon avec n−1 au dénominateur.
- IC de proportions : Wilson à 95 %. Ils quantifient une incertitude d’échantillonnage sous les hypothèses usuelles, pas les biais de sélection ou de codage.
- Rapports de proportions : estimations non ajustées et IC logarithmiques approximatifs ; pas de correction pour comparaisons multiples, résultats exploratoires.
- Aucune p-value utilisée pour sélectionner un sujet de publication ou décider quelles variables conserver.
- Aucun diagnostic extrait automatiquement des notes ; aucune date suspecte corrigée sans vérification.

Source originale SHA-256 : `fce69850264ded4d123afe060311045d96c3d400b40a78ae8165dd9a10f5eefa`.

### Référence BEWE

[1] Bartlett D, Ganss C, Lussi A. Basic Erosive Wear Examination (BEWE): a new scoring system for scientific and clinical needs. Clinical Oral Investigations. 2008;12 Suppl 1:S65–S68. DOI : 10.1007/s00784-007-0181-5. https://pmc.ncbi.nlm.nih.gov/articles/PMC2238785/

Toutes les valeurs numériques de ce rapport proviennent du fichier transmis et des règles explicitement décrites ci-dessus ; la référence bibliographique soutient les catégories BEWE, pas les résultats de cette patientèle.
