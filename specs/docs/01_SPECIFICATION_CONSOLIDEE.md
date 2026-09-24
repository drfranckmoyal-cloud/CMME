# CMME — Dépistage dentaire, TCA et usure érosive
## Spécification consolidée clinique, scientifique et logiciel — V1.0

Date : 23 septembre 2026. Porteur : Dr Franck Moyal. Lieu : CMME, hôpital Sainte-Anne.

**Décisions exprimées par le porteur :** exploitation principalement rétrospective des consultations déjà réalisées ; poursuite d'un recueil amélioré ; moins de dix nouveaux patients par mois ; patients habituellement vus une seule fois ; application utilisée sur son ordinateur uniquement. Ces décisions priment sur les propositions ci-dessous.

**Précision du porteur sur le BEWE historique :** les valeurs avec tranche comportent normalement le score numérique précis entre parenthèses. Ce nombre est donc la cible principale de l'extraction. Il ne faut pas traiter l'ensemble du classeur comme des données uniquement regroupées en classes. L'audit vérifiera seulement la présence, la validité et les exceptions à ce format.

**Statut :** spécification de travail pour construire un prototype local et préparer le protocole de recherche. Ce fichier ne contient aucune donnée nominative de patient. Il ne constitue ni une application déjà développée, ni un protocole approuvé, ni un rapport statistique sur la patientèle. Les arbitrages encore ouverts sont explicitement identifiés. Ne pas leur substituer des suppositions silencieuses.

## Priorité des documents

Lire d'abord `00_DECISIONS_FINALES.md` dans ce dossier. La présente consolidation conserve les détails du cadrage initial en intégrant les dernières décisions. Le dictionnaire des champs et les rubriques scientifiques restent utiles ; les simplifications des maquettes ne les remplacent pas. Les documents voisins précisent la recette et l'architecture.

Page continue : **Contexte → Habitudes → Expositions → Examen → Prévention → Observations**. HBD = case indépendante en tête de Prévention. Menus boissons et aliments multisélections, chacun avec texte libre. Observations libres en fin. Aucun thème définitivement choisi ; Iris pour démarrer, Lagune disponible.

## 0. Décisions de consultation confirmées — prioritaires

- **Finalité :** outil personnel de recueil et d'analyse statistique. Pas de production de documents en fin de consultation et pas de substitution au dossier hospitalier.
- **Durée et usage :** 10 à 15 minutes pour la consultation, saisie pendant l'entretien.
- **Examen habituel déclaré :** panoramique et examen endobuccal. Enregistrer la disponibilité et l'examen effectif ; ne pas programmer une radiographie automatique ni supposer qu'un cliché est réalisé à chaque saisie.
- **Rubrique FDI :** retirée. La notation des numéros dentaires pour localiser un constat reste distincte.
- **Prévention :** protocoles modéré/avancé et options définis section 6.7 ; observations des conseils et pratiques, sans automatisation thérapeutique.
- **Identité :** code patient stable privilégié dans les écrans statistiques. Le nom peut rester dans une correspondance locale restreinte si nécessaire au rapprochement des anciens dossiers ; aucune coordonnée de contact n'est nécessaire au formulaire.
- **Fin de visite :** enregistrer et terminer. Aucun compte rendu, courrier, ordonnance ou fiche patient à générer.
- **Ancien recueil :** conserver les données brutes et les codes historiques sans reconstruire des détails non documentés.

### Noyau affiché et indicateurs complémentaires

Une page unique continue, six sections simultanément présentes, état de sauvegarde visible. Les raccourcis font défiler sans masquer les autres rubriques. Tous les champs ont une réponse inconnue/non évaluée ; aucune valeur clinique par défaut. Le brouillon s'enregistre sans obligation de tout remplir.

| Section | Visible pendant la consultation | Compléments sur demande |
|---|---|---|
| Contexte | Code, date, service, âge, profession/statut, diagnostic TCA et ancienneté | Source du diagnostic ; poids/taille/IMC issus du dossier si disponibles |
| Habitudes et symptômes | Brossages/jour, brosse/dureté, dentifrice, dernière visite dentaire ; bouche sèche, sensibilités, douleur ; bruxisme déclaré | Habitudes après vomissement, brossage énergique, accès aux soins, produits complémentaires |
| Expositions | Vomissements actuels/anciens/jamais rapportés ; fréquence ; reflux/régurgitations ; alimentation/boissons ciblées ; tabac/alcool/autres substances | Quantités, durée et arrêt des expositions, traitements, détails du TCA |
| Examen | Panoramique consultée et date ; examen endobuccal ; six scores BEWE ; autres usures, caries suspectées/observées, constats parodontaux et muqueux | CAO détaillé, CPITN si réellement mesuré, récessions localisées ; mesures salivaires et photos facultatives |
| Prévention | Enseignement HBD indépendant ; aucun protocole/modéré/avancé/personnalisé, confirmation des composantes ; option gouttière semi-rigide séparée | Produit exact, adaptations, usage déjà déclaré |
| Observations | Champ libre final, besoins de soins et orientation si documentés ; bouton Terminer | Aucune sortie documentaire |

Ne pas imposer un CAO complet ou un sondage CPITN : le praticien n'a confirmé que panoramique et examen endobuccal comme examen habituel. Les anciennes valeurs restent importables. Le coefficient masticatoire n'appartient pas au noyau futur.

### Indicateurs radiographiques à enregistrer

| Variable | Type / comportement |
|---|---|
| panoramic_review_status | Consultée/non disponible/non consultée ; aucun défaut |
| panoramic_date | Date du cliché si connue ; distincte de la consultation |
| panoramic_source | Dossier existant/réalisée dans le parcours/autre/inconnu |
| panoramic_interpretability | Exploitable/partiellement exploitable/non exploitable/non évalué |
| panoramic_findings | Aucun constat particulier renseigné/carie suspectée/lésion périapicale suspectée/perte osseuse/autre/non évalué ; plusieurs constats possibles |
| panoramic_note | Texte court, dent/localisation si utile ; constat radiographique séparé du diagnostic clinique |
| intraoral_exam_status | Réalisé/partiel/non réalisé ; aucune présélection |
| finding_source | Endobuccal/panoramique/les deux/autre, pour chaque constat utile |

Ne pas déduire BEWE ou diagnostic d'érosion du panoramique. L'import d'images/DICOM et l'analyse radiographique automatique ne sont pas requis pour la V1 ; seuls les constats saisis sont recueillis.

### Recette complémentaire

1. Modéré prépare deux composantes ; la confirmation enregistre les conseils effectivement retenus.
2. Avancé ajoute Tooth Mousse ; le mode d'application doit être renseigné ou explicitement inconnu ; la durée 10 minutes concerne seulement le mode gouttière proposé.
3. La gouttière semi-rigide pendant les vomissements reste sans réponse par défaut après sélection d'Avancé.
4. Changer de protocole ne détruit pas une adaptation déjà confirmée sans affichage du changement ; tout amendement reste traçable.
5. Un ancien libellé Avancé importé conserve des composantes inconnues : aucun Tooth Mousse ni usage de gouttière inventé.
6. Terminer une consultation ne génère aucun document ni dialogue d'impression.
7. Panoramique absente ou examen partiel : enregistrer le statut et les données disponibles ; ne pas inventer des constatations normales.

## 1. Instruction de départ pour Claude Code

Construis une véritable application de bureau de recueil clinique et d'exploitation de consultations de dépistage bucco-dentaire chez des patients présentant des TCA. Elle doit être utilisable par un chirurgien-dentiste pendant une consultation et permettre la reprise traçable des anciennes données. Le produit doit fonctionner localement, hors connexion, sur un seul ordinateur. Ne déploie pas de service web public et n'introduis pas de cloud dans la V1.

Lis ce fichier entièrement. Respecte les décisions du porteur. Inspecte le dépôt et ses instructions avant toute modification. Si un projet existe déjà, préserve ses choix cohérents. Commence par un prototype complet avec des patients fictifs, comprenant création d'un dossier, saisie d'une consultation, BEWE, reprise d'un ancien enregistrement, clôture de saisie sans document, extraction pseudonymisée et restauration d'une sauvegarde. La qualité des données et la sécurité doivent être intégrées au fonctionnement réel, pas représentées par des boutons décoratifs.

Traite les parties cliniques comme des règles métier : n'invente pas de classification, de questionnaire validé, de diagnostic ou de traitement. N'infère pas un TCA à partir du poids ou des lésions dentaires. N'infère pas l'absence de vomissements à partir d'une cellule vide. Ne transforme pas un texte ambigu en valeur certaine. N'entraîne aucun modèle et ne transmets aucune donnée de patient à une API d'IA.

Implémente d'abord le noyau clinique, la reprise rétrospective et les exports. Les modules salivaires instrumentaux, photos et suivi restent extensibles, sans alourdir le parcours initial. Fournis un guide d'installation, un guide utilisateur, les limites connues, un journal de décisions et les tests des règles à conséquence clinique ou scientifique. Pour chaque jalon, montre un parcours fonctionnel et signale précisément les éléments simulés ou incomplets.

## 2. Ce que montrent les supports existants

### 2.1 Lecture effectuée et limites de cet audit

Le cadrage initial reposait sur les aperçus et textes internes des supports. Une analyse ultérieure du classeur converti localement en XLSX a été réalisée cellule par cellule le 23 septembre 2026. Le rapport exploratoire joint dans `../research/ANALYSE_RETROSPECTIVE_V1.md` est la référence pour ses effectifs provisoires et ses anomalies. La fiche Pages sert au cadrage des rubriques ; elle n'a pas été ajoutée comme une consultation supplémentaire.

Les feuilles incluent notamment Centre Expert, SAS, Hospit longue durée, HDJ et HDJ Intensif. Le titre d'une feuille propose une source/service, pas un recrutement indépendant confirmé. Les répétitions d'identité ne prouvent ni l'unicité ni le suivi. Les fusions du rapport exploratoire restent à valider pour la base finale.

### 2.2 Inventaire des données retrouvées

| Support | Rubriques ou informations identifiées | Conséquence pour le produit |
|---|---|---|
| Numbers | Identité, sexe, âge ou classes d'âge, service | Dossier patient distinct de la consultation ; conservation de la précision d'origine |
| Numbers | Vomissements, alimentation/boissons, hygiène | Champs structurés, historique et source de l'information |
| Numbers | BEWE sous forme de tranche avec normalement un total précis entre parenthèses, confirmé par le porteur | Extraire prioritairement ce total ; conserver le texte brut et vérifier les exceptions |
| Numbers | CAO, CPITN, récessions ; ancienne rubrique FDI | Conserver les domaines utiles en complément ; supprimer la rubrique FDI du formulaire et des exports standard |
| Numbers | Autres usures, bruxisme/serrement, besoins de soins, prévention, observations | Séparer constat, hypothèse clinique, orientation et intervention effectivement réalisée |
| Pages | Date, dernière visite chez le dentiste, occupation, tabac, alcool, brossage, alimentation, histoire du TCA | Réintégrer ces domaines dans le nouveau formulaire |
| Pages | Doléances, examens exo- et endobuccal, examen clinique, BEWE, CAO détaillé, coefficient masticatoire, recommandations | Conserver les notes utiles au recueil ; aucun compte rendu de sortie ; coefficient masticatoire historique hors noyau |

Le texte libre contient déjà des informations potentiellement importantes : expositions anciennes, arrêt des vomissements, nombre de crises et de vomissements, quantités de boissons acides, durée du TCA, brossage après vomissement, produits au charbon, sécheresse buccale et réhabilitations antérieures. Leur présence ne signifie pas qu'elles ont été recherchées systématiquement chez tous les patients.

### 2.3 Points à corriger lors de la reprise

- Les classes historiques affichées « 0 / 1–5 / 6–11 / 12–18 » ne sont pas les classes de référence BEWE. Conserver les anciennes classes avec un nom explicite ; ne pas les rebaptiser.
- Une formulation du type « 1–5 (7) » a été repérée : la tranche et le nombre divergent. Proposer 7 comme total selon la convention confirmée par le porteur, conserver le signalement et demander une validation de cette exception avant l'analyse. La tranche seule ne doit pas écraser le nombre précis.
- Une séquence de cinq scores accompagnée d'un total a été repérée. Ne pas compléter le sixième sextant par zéro ni reconstruire des sextants à partir du total.
- « Non » dans une ancienne colonne vomissements ne prouve pas l'absence de vomissements au cours de la vie. La définition historique de la colonne reste à documenter.
- « RAS » est lié au contexte de sa rubrique ; il ne constitue pas un examen négatif de tous les domaines.
- Une mention d'usure mécanique ou de bruxisme ne constitue pas un diagnostic instrumental du bruxisme.
- « Soins spécialisés », « suivi spécialisé » et « soins réalisés » sont des concepts différents.
- Un âge en classe n'autorise pas à inventer un âge exact. Conserver les bornes et leur caractère ambigu lorsque nécessaire.

Pour une reprise fiable : exporter toutes les feuilles Numbers en XLSX, conserver l'original, puis vérifier les valeurs affichées, les dates, les menus déroulants, les lignes masquées et les formules éventuelles. L'export sera une source d'import ; aucune modification de l'original n'est nécessaire. Le fichier Pages d'exemple ne permet pas de compléter par analogie les autres patients.

## 3. Stratégie de publication

### 3.1 Projet prioritaire : étude rétrospective descriptive

**Question :** quelle est la distribution de l'usure érosive dentaire et des besoins de soins documentés chez les patients avec TCA ayant bénéficié du dépistage à la CMME sur une période définie ?

**Titre de travail anglais :** “Erosive tooth wear and documented dental care needs in patients with eating disorders attending a specialist hospital service: a retrospective cross-sectional study”.

Le titre sera ajusté aux données effectivement disponibles. “Documented” évite d'assimiler l'absence d'une note à l'absence d'un besoin. Le terme transversal convient à une observation initiale par patient exploitée rétrospectivement. Il ne faut pas appeler ce travail une cohorte longitudinale en l'absence de mesures répétées.

La relation TCA–érosion a déjà été étudiée [2–4]. La contribution proposée est un portrait clinique transparent d'un parcours hospitalier réel, incluant les besoins de soins et les expositions retrouvées dans les dossiers. Une recherche bibliographique ciblée a été effectuée ; elle ne justifie pas de revendiquer une première mondiale ou l'absence de travaux similaires.

**Population proposée :** tous les patients éligibles effectivement dépistés pendant l'intervalle retenu. Pour l'analyse principale, retenir la première consultation éligible chronologique de chaque patient. Si sa date manque, résoudre l'ordre à partir d'une source vérifiable ; sinon identifier le dossier comme ambigu. Ne pas choisir la visite avec le score le plus élevé ni la plus complète pour améliorer artificiellement les résultats.

**Période :** à fixer à partir des premières et dernières dates réellement documentées. Les dates de création des fichiers ne remplacent pas les dates de consultation.

**Inclusion proposée :** consultation de dépistage dans le périmètre CMME, TCA documenté dans une source autorisée, période éligible, utilisation des données conforme au cadre institutionnel. Définir avant extraction le traitement des mineurs et des diagnostics non confirmés. L'absence de BEWE n'exclut pas automatiquement le patient du descriptif général ; elle réduit le dénominateur de l'analyse BEWE.

**Critère principal proposé :** distribution du total BEWE documenté et validé à la consultation index : effectif analysable, médiane, quartiles, étendue et représentation de la distribution. Présenter distinctement les totaux historiques seuls et les totaux calculables à partir de six sextants. Cette proposition doit être arrêtée avant les analyses d'association.

**Critères secondaires :** catégories de référence lorsqu'un total exact est disponible ; besoins de soins documentés ; orientations et mesures préventives consignées ; autres constats bucco-dentaires ; expositions présentes ou anciennes lorsque leur temporalité est interprétable. Ne pas présenter les catégories BEWE comme des probabilités individuelles de progression.

Si seule une tranche historique est disponible, elle peut servir à une description séparée, mais ni au calcul d'une médiane de scores exacts, ni à une conversion certaine vers une autre classification. Les analyses par sextant concernent exclusivement les sujets dont les sextants sont disponibles ; leur dénominateur sera explicite.

Cette situation « tranche seule » est un cas d'exception à rechercher, non le format principal annoncé. Avec les totaux précis entre parenthèses, le premier article pourra décrire leur distribution et recalculer les catégories de référence, sans disposer rétrospectivement du détail par sextant. Ne pas exclure ces totaux exacts uniquement parce que les sextants manquent.

**Portée :** résultats applicables à la population dépistée dans ce service, pas à tous les patients présentant un TCA ni à la population générale. Si l'exhaustivité du dépistage n'est pas vérifiable, utiliser “patients screened” ou “consecutive available records” uniquement selon les faits établis, sans revendiquer une série consécutive par défaut.

### 3.2 Pistes secondaires classées par faisabilité

| Priorité | Question | Données indispensables | Limite décisive |
|---|---|---|---|
| 1 | Profil BEWE, besoins de soins et prévention documentée | Patient unique, consultation index, BEWE validé, besoins et actions consignées | Faisable rétrospectivement si les dossiers sont suffisamment interprétables |
| 2 | Lésions selon vomissements actuels, anciens, jamais rapportés | Histoire explicite, période de référence, âge, durée du TCA, BEWE | Un simple oui/non historique ne suffit pas pour créer ces trois groupes |
| 3 | Expositions alimentaires acides chez les patients sans vomissements rapportés | Absence actuelle et passée documentée, aliments/boissons, fréquences, BEWE | Sous-groupe probablement petit ; absence de causalité démontrable |
| 4 | Accès aux soins et besoins non satisfaits | Dernier recours, dentiste habituel, besoin clinique, obstacle déclaré | Données souvent manquantes dans l'existant ; à améliorer prospectivement |
| 5 | Coexistence d'usure érosive et de signes mécaniques | Examen standardisé, auto-déclaration séparée, BEWE, âge | Une association ne démontre pas une interaction causale |
| Futur | Acceptabilité du dépistage ou évolution après prévention | Questionnaire dédié ou visites répétées, intervention définie | Pas démontrable avec une seule visite et les seuls dossiers actuels |

Ne pas multiplier des articles à partir de sous-groupes trop faibles. Un premier article cohérent avec quelques analyses secondaires est préférable. Les hypothèses issues de la lecture actuelle sont exploratoires ; ne pas prétendre qu'elles ont été préenregistrées avant cette lecture.

### 3.3 Plan d'analyse rétrospective proposé

1. Construire le diagramme de sélection : dossiers identifiés, doublons, consultations hors période, éligibilité non vérifiable, oppositions/exclusions applicables, patients inclus, BEWE analysables. Les nombres seront calculés après import validé.
2. Décrire les versions de formulaires, les secteurs et les périodes. Le changement de recueil peut expliquer une partie des différences observées.
3. Produire une matrice de disponibilité par variable et par version/source ; distinguer non recueilli, inconnu, non évalué, refus et non applicable.
4. Décrire les variables avec leurs dénominateurs propres. Une proportion sera toujours n/N ; les données manquantes ne sont pas intégrées aux “non”. Pour les proportions, utiliser des IC à 95 % appropriés, par exemple Wilson.
5. Présenter les âges exacts séparément des classes seules. Ne pas convertir les classes en points milieux pour obtenir une moyenne prétendument observée.
6. Examiner les associations seulement si les groupes et leur complétude le permettent. Privilégier tailles d'effet et intervalles de confiance. Les tests exacts peuvent être appropriés aux petites cellules ; éviter une batterie de p-values.
7. Aucun modèle multivariable automatique ni sélection de variables par p-value. Choisir avec un méthodologiste quelques facteurs sur justification clinique, après examen des effectifs, distributions, données manquantes et dépendances entre diagnostic et vomissements.
8. Ne pas imposer l'âge, l'IMC, les médicaments ou la durée du TCA dans un modèle s'ils sont trop peu documentés. Dire explicitement que les associations peuvent être confondues.
9. Ne pas imputer rétrospectivement un diagnostic, une exposition négative ou un score manquant. Une imputation multiple ne serait discutée qu'avec un plan statistique et des hypothèses défendables, pas pour combler un formulaire jamais utilisé.
10. Analyses de sensibilité : exclure les scores discordants non résolus ; comparer total historique exact versus six sextants ; décrire les inclus avec/sans BEWE ; stratifier par version de recueil si nécessaire. Ne pas laisser ces choix dépendre de la significativité.

**Effectif :** inclure l'ensemble des dossiers éligibles disponibles, après dédoublonnage ; aucun nombre minimal universel ne garantit une publication. À titre de planification uniquement, pour une proportion proche de 50 %, la demi-largeur approximative d'un IC à 95 % est 1,96 × racine(0,25/n) : environ 14 points à n=50, 10 à n=100 et 7 à n=200. Ces exemples ne sont ni les effectifs du fichier ni un calcul de puissance pour une association. Les sous-groupes auront une précision moindre.

**Livrables scientifiques après nettoyage :** protocole daté ; plan d'analyse ; dictionnaire ; diagramme de sélection ; tableau 1 descriptif ; tableau 2 lésions/besoins ; tableau 3 analyses secondaires si justifiées ; annexes de complétude et de recodage ; scripts reproductibles ; manuscrit selon STROBE et éléments RECORD pertinents [5–6].

### 3.4 Articulation avec les nouvelles consultations

L'application crée désormais des données mieux structurées. Marquer chaque consultation `legacy_retrospective` ou `structured_prospective` et sa version de formulaire. Les nouveaux dossiers ne doivent pas être mélangés aux anciens comme si leur qualité et leur méthode de recueil étaient identiques.

Le premier article peut porter sur une période rétrospective close. Un complément prospectif pourra être décrit ensuite, ou ajouté dans un plan annoncé à l'avance avec analyse de la période. Le recrutement futur reste transversal si chaque patient n'est vu qu'une fois. La possibilité technique de créer un suivi ne constitue pas un protocole de suivi.

### 3.5 Revues à considérer, sans promesse d'acceptation

- **BMC Oral Health** : première cible à examiner pour le volet descriptif, besoins de soins et épidémiologie clinique ; son périmètre officiel couvre ces sujets [9].
- **Clinical Oral Investigations** : à discuter si le phénotypage, la qualité des mesures et la portée analytique sont assez solides [10].
- **Journal of Eating Disorders** : pertinente si le message principal concerne le parcours TCA, ses comorbidités et l'intégration des soins bucco-dentaires [11].

La décision finale dépendra du nombre de dossiers analysables et du contenu. Vérifier alors les consignes, types d'articles et frais ; aucun tarif ni facteur d'impact n'est figé ici.

## 4. Recueil futur : court par défaut, détaillé quand utile

La consultation dure 10 à 15 minutes, saisie comprise, et les informations sont saisies pendant l'entretien. Le logiciel doit accompagner ce rythme : une page continue à sections ouvertes, compléments facultatifs repliables, navigation clavier, réponses rapides et compléments conditionnels. Tester le temps réel sur un pilote de dix consultations. Aucun module optionnel ne doit bloquer la clôture.

**Noyau à documenter, ou à marquer explicitement indisponible :** date/service ; âge avec précision ; diagnostic TCA et source ; histoire des vomissements ; exposition acide alimentaire ciblée ; hygiène ; symptômes ; BEWE ; besoins de soins ; prévention/orientation. Pour valider le recueil, un statut explicite de donnée manquante est acceptable. Ne jamais forcer une réponse inexacte pour terminer.

**Complément utile si disponible :** ancienneté du TCA, traitements, sécheresse buccale, IMC provenant du dossier, tabac/alcool/autres substances, bruxisme éveil/sommeil, recours dentaire et obstacles. Leur priorité scientifique varie selon la question ; “utile” ne signifie pas “tout mesurer systématiquement”.

**Module de recherche optionnel :** débit salivaire, pH, questionnaires validés, photos standardisées, quantification par surface. Ne l'activer qu'après choix d'un protocole réaliste. Une déclaration de bouche sèche n'est pas une mesure d'hyposialie ; une mesure ponctuelle de pH ne résume pas l'exposition acide habituelle.

## 5. Conventions du dictionnaire de données

Interface en français ; identifiants de variables stables en anglais ; libellés français et anglais dans les exports de dictionnaire. Les codes métier ne changent pas lorsqu'un libellé est corrigé.

Chaque champ clinique doit pouvoir associer : `value`, `missing_reason`, `source_type`, `source_ref`, `observed_at`, `recorded_at`, `author_id`, `certainty` et `form_version`. Implémenter ces métadonnées dans des colonnes ou une table de provenance, sans multiplier inutilement des objets JSON opaques. Les champs structurés importants doivent rester interrogeables.

`missing_reason` : `not_recorded` (ancien support muet), `not_asked`, `unknown`, `declined`, `not_assessable`, `not_applicable`, `ambiguous_source`. `null` avec raison n'est jamais égal à zéro, faux ou “non”. Valeur et raison de manque sont mutuellement exclusives, sauf note séparée d'incertitude.

`source_type` : `patient_report`, `medical_record`, `clinical_exam`, `instrument_measurement`, `legacy_import`, `clinician_adjudication`. Une source administrative ne valide pas un diagnostic. `certainty` : `documented`, `reported`, `suspected`, `unresolved` selon la nature du champ ; ne pas utiliser un score numérique inventé.

Dates ISO ; fuseau de l'interface Europe/Paris ; dates cliniques sans conversion décalant le jour. Distinguer date clinique et horodatage informatique. Dates approximatives avec précision `day/month/year/unknown`. Ne pas convertir “il y a deux ans” en date au jour près.

Catégories ci-dessous proposées pour le formulaire, non présentées comme des échelles validées. Les questionnaires validés éventuels doivent garder leur version, leur traduction autorisée et leur calcul officiel.

## 6. Dictionnaire clinique opérationnel

### 6.1 Patient, consultation et contexte

| Variable | Type / valeurs | Règle |
|---|---|---|
| patient_id | UUID local | Stable ; jamais dérivé du nom |
| study_id | Code aléatoire par projet | Table de correspondance séparée de l'export |
| clinical_identity | Nom/prénom et identifiant hospitalier si autorisé | Espace clinique restreint ; hors extraction recherche |
| birth_date | Date facultative | Seulement si nécessaire au dossier ; hors export standard |
| age_years | Nombre | Âge à la consultation ; origine calculée ou documentée |
| age_band_legacy | Texte source + bornes si interprétables | Pas de conversion arbitraire en âge exact |
| sex_recorded | Catégorie selon source + inconnu | Ne pas déduire d'un prénom ; identité de genre séparée si utile et justifiée |
| occupation_text | Texte court facultatif | Pour le soin ; éviter le texte précis dans les exports |
| occupational_status | Études/emploi/sans emploi/retraite/autre/inconnu | Recodage pour analyse ; nomenclature à valider |
| visit_id | UUID | Une observation clinique datée |
| visit_date | Date + précision | Pas de date inventée à l'import |
| visit_type | Initiale/suivi/non déterminé | Initiale clinique distincte de visite index d'une étude |
| service_code | Centre expert/SAS/hospitalisation complète/longue durée/HDJ/HDJ intensif/autre | Référentiel local à confirmer, garder libellé source |
| examiner_id | Référence praticien | Nécessaire à la traçabilité des scores |
| collection_mode | Historique rétrospectif/structuré prospectif | Non modifiable silencieusement |
| form_version | Chaîne | Version du formulaire effectivement utilisé |
| visit_status | Brouillon/validé/amendé | Une correction après validation crée une version |

### 6.2 TCA, temporalité et contexte somatique

| Variable | Type / valeurs | Règle |
|---|---|---|
| ed_diagnosis | AN/BN/BED/ARFID/OSFED/UFED/autre/non précisé | Reprendre le diagnostic clinique documenté, pas une auto-classification logicielle |
| ed_system_version | DSM/CIM + version/texte historique | Garder la nomenclature source et le mapping séparés |
| ed_subtype | Texte codifié selon nomenclature | Par exemple sous-type AN si documenté ; ne pas l'inférer d'un symptôme isolé |
| ed_diagnosis_source | Dossier/clinicien/patient/non confirmé | Avec date et référence |
| ed_onset_age ou ed_onset_date | Nombre/date + précision | Ne pas demander les deux si l'un suffit |
| ed_duration_months | Nombre facultatif | Calculé seulement sur données compatibles ; estimé identifié |
| ed_current_course | Actif/rémission rapportée/autre/inconnu | Statut clinique daté, pas déduit du BEWE |
| restrictive_pattern | Oui/non/inconnu | Comportement déclaré ; période de référence |
| binge_episodes_28d | Entier ou estimation min/max | Crises distinctes des vomissements |
| compensatory_behaviors | Liste : vomissements/laxatifs/diurétiques/exercice/restriction/autre | Présent/passé et période ; détails seulement pertinents |
| chew_spit | Jamais/ancien/actuel/inconnu | Fréquence si utile, sans le classer comme vomissement |
| weight_kg, height_cm | Décimaux + date/source | Facultatifs ; privilégier le dossier existant |
| bmi | Calculé kg/m² | Calcul si mesures disponibles et dates compatibles ; aucune conclusion nutritionnelle automatique |
| relevant_conditions | Liste/texte clinique | RGO documenté, antécédents digestifs, grossesse avec hyperémèse, autres expositions pertinentes |
| medications | Table répétable : nom, classe, dates, statut | Rechercher notamment traitements potentiellement sialoprives, sans leur attribuer automatiquement un effet |

Les diagnostics changent au cours du temps : les conserver à la visite, sans écraser l'histoire. La liste finale des nomenclatures doit être validée avec l'équipe TCA. Ne pas demander un nouveau questionnaire psychiatrique à tous les patients pour reproduire une information déjà présente.

### 6.3 Vomissements, reflux et régurgitations

| Variable | Type / valeurs | Règle |
|---|---|---|
| vomiting_current_28d | Oui/non/inconnu | Les 28 derniers jours constituent une fenêtre proposée pour le futur |
| vomiting_lifetime | Jamais rapporté/ancien uniquement/actuel/inconnu | “Jamais” seulement après recherche explicite ; historique ambigu reste inconnu |
| vomiting_days_28d | Entier 0–28 | Nombre de jours concernés |
| vomiting_episodes_28d | Entier ou min/max estimés | Nombre de vomissements, pas nombre de crises |
| vomiting_pattern | Quotidien/intermittent/périodes/autre + texte | Complète le nombre ; ne le remplace pas automatiquement |
| vomiting_onset | Date/âge + précision | Facultatif |
| vomiting_duration_months | Nombre + estimation | Cumul si documentable ; ne pas calculer une “dose acide” validée |
| vomiting_stop_interval | Nombre + unité + date de référence | Pour les anciens vomissements |
| vomiting_past_peak | Nombre/unité + période | Optionnel, charge ancienne ; jamais substitué à l'actuel |
| vomiting_context | Auto-induit/spontané/autre/inconnu | Distinct du diagnostic TCA |
| post_vomit_rinse | Oui/non/inconnu + produit | Déclaration ; pas preuve d'efficacité |
| post_vomit_brushing | Oui/non/inconnu | Si oui : délai estimé en minutes ou catégorie avec précision |
| reflux_symptoms | Oui/non/inconnu + fréquence | Symptômes, pas diagnostic prouvé |
| reflux_documented | Oui/non/inconnu + source | Distinguer diagnostic antérieur et suspicion |
| regurgitation_rumination | Rapportée/documentée/suspectée/absente/inconnue | Ne pas reclasser automatiquement en vomissements |

Si la fréquence est une plage, conserver `min`, `max`, `unit`, `period`. Ne pas prendre le milieu pour produire une valeur exacte. Le formulaire affiche un petit résumé de la temporalité afin que le praticien puisse confirmer qu'il correspond au récit.

### 6.4 Alimentation, habitudes et substances

Créer une table d'expositions répétables, limitée aux catégories positives ou incertaines, avec une action explicite “catégories recherchées : aucune rapportée”. Catégories proposées : sodas sucrés ; sodas sans sucre ; boissons énergisantes/sport ; jus ; agrumes/citron ; vinaigre/pickles ; autres fruits ; autres aliments/boissons acides ; compléments effervescents. Ne pas déduire une acidité quantitative de leur seule catégorie.

Chaque exposition comporte `category`, `product_text` optionnel, `current_or_past`, `days_per_week` (0–7), `episodes_per_day`, `quantity`, `unit`, `duration_months` si connue, `between_meals`, `prolonged_sipping_or_holding`, `reference_period` (7 derniers jours pour l'actuel proposé), source et précision. Ne pas additionner les fréquences de catégories co-consommées pour fabriquer un indice de risque non validé.

Les conseils alimentaires doivent rester compatibles avec la prise en charge des TCA et être coordonnés à l'équipe ; le logiciel ne génère pas de liste automatique d'aliments à interdire.

| Variable | Type / valeurs | Règle |
|---|---|---|
| tobacco_status | Jamais/ancien/actuel/inconnu | Cigarettes/jour et durée en années si documentées |
| tobacco_cigarettes_day | Nombre ou plage | Ne pas convertir autres produits en cigarettes arbitrairement |
| vaping | Jamais/ancien/actuel/inconnu | Séparé du tabac fumé |
| alcohol_status | Aucun déclaré/ancien/actuel/inconnu | Fenêtre de référence explicite |
| alcohol_units_week | Nombre/plage + définition | Unité française 10 g si retenue ; sinon garder quantité brute et format du verre |
| alcohol_beverage | Catégories | Pertinent aussi pour le type d'exposition ; ne pas extrapoler une bouteille non précisée |
| other_substances | Table : type, actuel/passé, fréquence, voie si pertinente | Recueil proportionné, confidentialité renforcée, option refus |

### 6.5 Hygiène, recours et symptômes

| Variable | Type / valeurs | Règle |
|---|---|---|
| brushing_daily | Nombre ou plage | Autoriser zéro, moins d'une fois/jour, plusieurs fois ; fréquence hebdomadaire possible |
| brush_type | Manuelle/électrique/les deux/autre/inconnu | Ne pas inférer la dureté |
| bristle_hardness | Souple/médium/dure/inconnue | Déclarée |
| brushing_duration_min | Nombre/plage | Optionnel |
| forceful_brushing_reported | Oui/non/inconnu | Séparé de lésions attribuées au brossage |
| toothpaste_name | Texte court | Nom exact si connu |
| toothpaste_fluoride_ppm | Nombre/inconnu | Seulement étiquette ou source vérifiable ; ne pas inférer de la marque |
| toothpaste_features | Charbon/blanchissant/désensibilisant/autre | Liste descriptive ; ne pas inventer le RDA |
| interdental_cleaning | Type et fréquence | Optionnel |
| mouthrinse | Produit/fréquence | Optionnel |
| hygiene_legacy | Insuffisante/suffisante/iatrogène/texte | Conserver jugement historique ; le nouveau formulaire documente ses composantes |
| last_dental_visit | Date ou intervalle/unité | Avec date de référence ; “5 mois” n'est pas un rendez-vous exact |
| usual_dentist | Oui/non/inconnu | Distinct d'une visite récente |
| access_barrier | Coût/anxiété/honte/disponibilité/autre/aucun/inconnu | Raisons déclarées, plusieurs possibles |
| dental_pain | Oui/non + intensité 0–10 | Période : 7 derniers jours proposée ; échelle numérique, pas EVA graphique |
| hypersensitivity | Oui/non/inconnu + intensité 0–10 | Déclencheurs ; auto-déclaration séparée du test clinique |
| chewing_difficulty | Oui/non/inconnu | Retentissement déclaré |
| aesthetic_concern | Oui/non/inconnu | Optionnel |
| dry_mouth_reported | Oui/non/inconnu + fréquence | Ne pas renommer hyposialie |

### 6.6 Examen, BEWE et autres domaines

| Variable | Type / valeurs | Règle |
|---|---|---|
| bewe_sextant | 0/1/2/3 ou non évaluable/manquant | Six enregistrements nommés, jamais un tableau positionnel anonyme |
| bewe_worst_tooth | Code FDI facultatif | Doit appartenir au sextant choisi |
| bewe_worst_surface | Vestibulaire/palatine-linguale/occlusale-incisale | Optionnel ; ne remplace pas l'examen de toutes les surfaces pertinentes |
| bewe_total_derived | Entier 0–18 ou null | Calcul contrôlé, jamais éditable directement |
| bewe_total_historical | Entier 0–18 ou null | Total exact documenté sans sextants ; provenance et validation |
| bewe_legacy_band | Texte/catégorie historique | Ne pas convertir automatiquement |
| restored_surfaces | Localisation/type/date si connue | Une restauration peut masquer le tissu perdu |
| missing_teeth | Liste FDI + cause si connue | Distinguer absence et érosion absente |
| mechanical_wear_signs | Oui/non/non évalué + localisation | Observation clinique ; distinction étiologique prudente |
| awake_bruxism_reported | Oui/non/inconnu | Serrement/grincement ; période précisée |
| sleep_bruxism_reported | Oui/non/inconnu | Patient/entourage/autre source |
| bruxism_clinical_signs | Texte/codes + date | Séparé du déclaratif |
| bruxism_instrumental | Résultat/source si existant | Optionnel ; ne pas créer un degré de certitude automatique |
| tmj_muscle_symptoms | Douleur ATM/muscles/autre | Optionnel |
| dmft_d, dmft_m, dmft_f | Entiers séparés | Nombre de dents ; M = absentes pour carie, pas toute dent absente |
| dmft_total | Calculé ou historique identifié | Calcul si trois composantes disponibles et protocole défini ; plafonds selon dentition retenue |
| periodontal_legacy | CPITN brut / autre indice | Garder l'indice d'origine ; ne pas l'appeler stade parodontal |
| plaque_gingival_findings | Présence/absence/non évalué + description | Un pourcentage exige une méthode et un dénominateur |
| recession_sites | Dent/site/profondeur mm si mesurée | Texte brut conservé si ancien classement incompris |
| extraoral_findings | Sialadénose/asymétrie/autres/non évalué | Observation distincte de diagnostic étiologique |
| mucosal_findings | Lésion/localisation/description/non évalué | Pas de diagnostic automatique |
| salivary_flow | Valeur mL/min + protocole/date/conditions | Module optionnel ; stimulus, durée, volume et heure nécessaires |
| salivary_ph | Nombre + méthode/heure/conditions | Module optionnel ; pas de diagnostic de reflux |
| masticatory_coefficient_legacy | Valeur brute + méthode inconnue si besoin | À réintroduire seulement après clarification de son calcul et de son intérêt |
L'ancienne rubrique FDI est supprimée du formulaire, des statistiques et des exports standard. Son contenu éventuel reste seulement dans la source brute d'import. La numérotation anatomique des dents du schéma BEWE est conservée : elle est distincte de cette ancienne colonne.

### 6.7 Prévention, besoins et orientations

`care_need` est une liste répétable : urgence/douleur ; carie ; parodonte ; usure/avis spécialisé ; autre ; aucun besoin identifié ; non évalué. Consigner le motif, la priorité clinique choisie et la source. Une recommandation ne prouve pas la réalisation d'un soin.

Les protocoles ci-dessous décrivent les pratiques déclarées par Franck pour le recueil. Leur enregistrement ne constitue pas une validation scientifique d'efficacité, ni une prescription automatique. Le BEWE ne sélectionne pas de protocole à la place du praticien.

| Protocole saisi | Composantes consignées |
|---|---|
| Modéré | Dentifrice anti-érosion ; bain de bouche anti-érosion après vomissement |
| Avancé | Composantes du modéré ; Tooth Mousse quotidien, en application directe ou en gouttières pendant 10 minutes par jour |

Le port de gouttières semi-rigides pendant les vomissements est une **option indépendante**, parfois utilisée selon Franck. Elle n'est jamais précochée par le protocole avancé et n'est pas confondue avec une gouttière d'application du produit ou une gouttière nocturne de bruxisme.

Interaction : cliquer « Modéré » ou « Avancé » prépare les composantes dans la saisie courante ; le praticien peut les décocher ou les adapter, puis confirme « mesures conseillées ». Sans cette confirmation, ne pas enregistrer les propositions comme effectivement conseillées. Pour un patient sans vomissements actuels, le bain de bouche après vomissement reste une composante contextuelle à confirmer ou à marquer non applicable, sans déduire une exposition positive du protocole choisi.

| Variable | Valeurs / règle |
|---|---|
| prevention_protocol | Aucun protocole proposé/modéré/avancé/personnalisé ; absence de réponse distincte |
| hbd_teaching | Action indépendante oui/non/manquante ; case au début de Prévention, conservée lors des changements de protocole |
| protocol_version | CMME-FM-2026-09 ; version du contenu du raccourci, pas preuve de validation clinique |
| anti_erosion_toothpaste | Conseillé/non conseillé/non renseigné ; nom du produit facultatif |
| anti_erosion_rinse | Conseillé/non conseillé/non applicable/non renseigné ; contexte après vomissement ; produit facultatif |
| tooth_mousse | Conseillé/non conseillé/non renseigné ; produit exact conservé |
| tooth_mousse_route | Directe/gouttière/les deux/non précisé ; visible si produit conseillé |
| tooth_mousse_frequency | Quotidien par défaut dans la proposition avancée ; fréquence réelle conseillée modifiable |
| tooth_mousse_tray_minutes | 10 proposé pour l'application en gouttière ; modifiable ; ne pas appliquer automatiquement à la voie directe |
| vomiting_semirigid_tray | Conseillée/non conseillée/non applicable/non renseigné ; option indépendante sans valeur initiale |
| bruxism_tray_existing | Oui/non/inconnu ; séparé des deux autres usages de gouttières |
| prevention_status | Conseillé ce jour/déjà utilisé selon le patient/réalisé pendant la consultation/refusé/inconnu, par composante |
| reported_use | Facultatif : usage actuel déclaré ; ne jamais le déduire du conseil |
| prevention_note | Texte court facultatif pour adaptation |

Ne pas demander une observance future à une consultation unique. Ne pas classer le patient en « traitement reçu » parce que le protocole a été conseillé. À l'import, le seul libellé historique modéré/avancé ne doit pas créer automatiquement des composantes ou une durée : contenu historique à confirmer, `components_unknown` par défaut.

Les autres actions possibles sont information/démonstration d'hygiène, conseil individualisé et coordination avec l'équipe. Aucune fiche ni ordonnance n'est produite. Une note « autre action » est disponible sans agrandir le formulaire principal.

`referral` : destination, motif, date, statut proposée/acceptée/rendez-vous rapporté/consultation confirmée/inconnu. Le logiciel ne suppose pas qu'une orientation a été suivie. Un accès ponctuel à une information ultérieure ne transforme pas tout l'échantillon en suivi longitudinal complet.

## 7. Module BEWE : contrat fonctionnel

### 7.1 Mesure et présentation

Référence : Bartlett, Ganss et Lussi [1]. Scores : 0 absence d'usure érosive ; 1 altération initiale de texture ; 2 perte tissulaire distincte sur moins de la moitié de la surface ; 3 perte sur au moins la moitié. Retenir le maximum du sextant. La dentine n'est pas une condition obligatoire des scores 2 ou 3.

Sextants : supérieur droit 17–14 ; antérieur supérieur 13–23 ; supérieur gauche 24–27 ; inférieur gauche 37–34 ; antérieur inférieur 33–43 ; inférieur droit 44–47. Total complet 0–18. Classes de référence : 0–2, 3–8, 9–13, 14–18. Le score décrit les lésions ; il n'établit pas leur cause et ses classes ne constituent pas un pronostic individuel validé.

### 7.2 Interface

Petit odontogramme vectoriel, deux arcades, orientation “droite du patient” et “gauche du patient” toujours visible. Chaque sextant est sélectionnable au clic ou au clavier et dispose de boutons 0, 1, 2, 3 et “non évaluable”. Une entrée non remplie a un aspect différent du score 0. Les chiffres doivent rester lisibles sans couleur.

Ne pas enregistrer les scores en fonction de leur seule position à l'écran. Chaque zone possède un identifiant anatomique permanent. L'ordre visuel des sextants mandibulaires peut différer de l'ordre de numérotation : le mapping anatomique doit être testé.

Afficher en permanence “x/6 sextants renseignés”. Afficher le total seulement lorsque les six sont scorés. Si un sextant est non évaluable, indiquer la raison : édentement, restauration masquant l'évaluation, accès limité, autre. Politique conservatrice propre à ce projet : ne pas proratiser ni substituer zéro ; total standard indisponible. Documenter cette politique dans le protocole, sans la présenter comme une règle universelle publiée.

Le total historique exact peut être visible dans un encadré distinct, même si les sextants sont absents. Une divergence entre un total historique et un calcul actuel doit rester visible jusqu'à adjudication. Aucun score ne déclenche automatiquement un traitement ou une conclusion diagnostique.

### 7.3 Exemples de recette synthétiques

| Entrée | Résultat attendu |
|---|---|
| Six zéros | Total 0, examen complet |
| 1, 0, 1, 0, 0, 0 | Total 2, classe 0–2 |
| 1, 1, 1, 0, 0, 0 | Total 3, classe 3–8 |
| 2, 2, 1, 1, 1, 1 | Total 8, classe 3–8 |
| 2, 2, 2, 1, 1, 1 | Total 9, classe 9–13 |
| 3, 2, 2, 2, 2, 2 | Total 13, classe 9–13 |
| 3, 3, 2, 2, 2, 2 | Total 14, classe 14–18 |
| Six 3 | Total 18 |
| Cinq valeurs et une absente | Pas de total standard ; jamais zéro implicite |
| Un sextant non évaluable | Pas de total standard ; raison conservée |
| Total historique 7 sans sextants | Valeur historique 7 ; zéro sextant reconstruit |
| Texte historique “1–5 (7)” | Anomalie ; aucun arbitrage automatique |
| Valeur 4, négative, décimale ou texte arbitraire | Rejet du score structuré ; texte d'import conservé |

## 8. Reprise rétrospective : procédure obligatoire

### 8.1 Trois couches séparées

1. **Source brute immutable** : fichier original/importé, empreinte SHA-256, feuille/table/ligne/colonne, valeur affichée, valeur technique si disponible, date d'import. Accès clinique seulement, car l'original peut contenir des identifiants.
2. **Proposition de normalisation** : valeur proposée, règle appliquée, statut de validation et anomalies. Une proposition n'est pas automatiquement une donnée d'étude.
3. **Donnée validée** : décision du relecteur, date, justification, lien vers source. Toutes les modifications sont historisées.

### 8.2 Importeur V1

Supporter XLSX et CSV en priorité. Le format Numbers natif ne sera pas annoncé comme supporté sans parseur fiable et tests sur les fichiers réels. Supporter UTF-8, accents, espaces, décimales françaises, cellules fusionnées et détection des lignes d'en-tête répétées. Les fichiers Pages historiques peuvent être référencés et relus manuellement après export en PDF/DOCX ; ne pas promettre une extraction clinique automatique.

L'assistant d'import montre un aperçu, laisse choisir les feuilles et les lignes de données, propose un mapping vers le dictionnaire, distingue données individuelles et tableaux de synthèse. Une feuille de totaux ne doit jamais créer des patients. Il affiche le nombre de lignes retenues, exclues et ambiguës avant validation.

Rejouer le même import ne doit pas dupliquer les données : clé de provenance `file_hash + sheet + table + row` et détection du fichier déjà importé. Un export différent du même historique requiert une comparaison, pas un contournement du contrôle par un nouvel identifiant de fichier.

### 8.3 Exemples de règles de normalisation

- Oui/non : normaliser les espaces et la casse ; conserver le sens de la rubrique. Les réponses vides restent manquantes.
- BEWE “6–11 (9)” : extraire total historique 9 et tranche originale selon la convention confirmée par le porteur, puis validation de la règle sur les formats rencontrés. Recalculer la catégorie de référence à partir de 9. Vérifier les discordances systématiquement. Garder un indicateur “total seul”, sans exclure ce total des analyses par défaut.
- BEWE “6–11” sans nombre exact : stocker seulement la tranche. Ne pas attribuer 8 ou 9.
- Fréquence “2 à 3 fois/jour” : min=2, max=3, unité=jour, estimée. Ne pas produire 2,5 comme observation.
- Texte “ancien vomissement” : coder ancien uniquement si le contexte temporel est explicite ; conserver inconnu pour le reste.
- “C:1 A:0 O:1” : proposer les trois composantes sous réserve du sens de A dans l'ancien recueil ; ne pas présumer que toute absence était liée à une carie.
- Classe d'âge seule : conserver classe et bornes si définies. Une borne “20–30” reste ambiguë quant aux inclusions des limites tant que le protocole historique n'est pas clarifié.
- Les dates d'une feuille ou de son nom ne deviennent dates de consultation que si Franck confirme leur signification.

### 8.4 Identification et doublons

Utiliser un identifiant institutionnel s'il est disponible et autorisé. Sinon, proposer des rapprochements par identité et éléments contextuels, sans fusion automatique sur le seul nom. Les variantes orthographiques créent une tâche de revue. Une personne ayant plusieurs consultations garde un seul dossier et plusieurs visites ; les visites identiques dupliquées sont identifiées séparément des retours véritables.

La fusion garde un historique et doit pouvoir être annulée. Aucun enregistrement brut n'est détruit. L'application conserve le motif d'exclusion de l'analyse, distinct du maintien du dossier de soin.

### 8.5 Abstraction du texte libre

Créer un manuel court de codage avant l'extraction systématique : définitions, fenêtres temporelles, exemples ambigus et règles de priorité. Tester sur une vingtaine de dossiers si l'effectif le permet ; ajuster le manuel puis recoder ce pilote avec les règles finales.

Proposition qualité : double lecture d'un sous-échantillon, ainsi que de tous les cas ambigus, par un second lecteur autorisé si disponible. À défaut, prévoir une seconde lecture différée par le même opérateur et rapporter cette limite. Décrire l'accord et l'adjudication ; ne pas annoncer une reproductibilité inter-examinateur sans l'avoir mesurée.

L'extraction ne complète que ce qui est explicitement documenté. Les notes contiennent parfois des informations très sensibles : aucune copie de ces notes dans le dépôt de code, les tickets, les captures de démonstration ou les outils d'IA.

## 9. Parcours et écrans de l'application

### 9.1 Accueil

Trois actions principales : “Nouvelle consultation”, “Rechercher un patient”, “Reprendre les anciennes données”. Indicateurs secondaires : brouillons, anomalies à vérifier, date de dernière sauvegarde réussie. Les statistiques d'étude sont dans un espace distinct, pas mélangées aux rappels cliniques.

### 9.2 Fiche patient

Identité clinique, identifiant, liste chronologique des visites, documents locaux et statut d'utilisation pour chaque projet de recherche. La dernière information connue est affichée avec sa date ; elle n'est pas automatiquement considérée actuelle. Recherche possible par nom dans l'espace clinique seulement.

### 9.3 Consultation

Sections visibles sur une page : Contexte ; Habitudes et symptômes ; Expositions ; Examen/BEWE ; Prévention/orientation ; Observations libres. Habitudes immédiatement après Contexte. Les raccourcis sont des ancres, pas des étapes masquant le formulaire. Navigation clavier fluide, boutons oui/non/inconnu sans sélection préalable, champs conditionnels. Une saisie négative n'efface pas silencieusement un historique positif.

Sauvegarde automatique locale avec indication “enregistré à…”, gestion explicite des erreurs disque. Si la sauvegarde échoue, ne pas afficher un faux succès. Reprise après fermeture imprévue. Consultation validée verrouillée ; bouton “corriger” créant un amendement.

### 9.4 Revue rétrospective

Vue à deux panneaux : extrait source à gauche, champs structurés à droite. Filtres : score discordant, date absente, doublon possible, donnée temporelle ambiguë, codage en attente. Raccourcis pour accepter une proposition ou marquer indisponible. La validation en lot n'est autorisée que pour une règle déterministe vérifiée et des lignes sans anomalie.

### 9.5 Fin de saisie

Bouton « Terminer la saisie ». Afficher seulement un récapitulatif interne compact : BEWE, champs essentiels non renseignés et mesures consignées. Permettre de compléter ou de terminer avec des manques explicites. Aucun compte rendu, ordonnance, fiche patient, PDF, impression ou envoi n'est généré à la fin de la consultation. L'objectif est le recueil statistique personnel.

### 9.6 Espace étude

Création d'un projet avec période, critères, consultation index, variables autorisées et version du plan. Vue des effectifs, exclusions, complétude et distributions descriptives. Toute figure affiche son N et son filtre. Bouton “figer une extraction” : produit un jeu de données versionné et sa traçabilité. Ne pas fournir un moteur automatique de découverte de résultats significatifs.

### 9.7 Direction visuelle

Références les plus récentes : `../design/Iris.html` et `../design/Lagune.html`, page unique. Niveau attendu : produit professionnel soigné, palette affirmée, hiérarchie typographique claire, saisie dense mais lisible, aucune illustration décorative. Iris (prune/lilas) est le point de départ technique ; Lagune (pétrole/ivoire) reste disponible. L'ancienne direction vert sauge n'est plus la référence principale. Le choix final de thème appartient à Franck et n'est pas bloquant.

Lire `03_DESIGN_ET_PARCOURS.md` pour les détails et limites des maquettes. Les composants de saisie doivent couvrir le dictionnaire complet, les raisons de manque et la provenance même si la maquette les simplifie.

## 10. Architecture locale proposée

**Choix recommandé à valider sur l'ordinateur cible :** application Tauri avec interface React/TypeScript et couche métier native ; base SQLite chiffrée via SQLCipher. Ces technologies sont une proposition de mise en œuvre, pas une contrainte de recherche. Tauri expose des permissions/capacités à restreindre ; SQLCipher apporte le chiffrement de base [12–13]. Leur simple présence ne garantit pas la sécurité de l'ensemble.

Avant de figer la pile, Claude vérifie le système d'exploitation, les versions supportées, le packaging, les licences et la compatibilité réelle du driver choisi avec SQLCipher. Ne pas substituer silencieusement SQLite en clair si le chiffrement pose problème. Sur Mac ancien, vérifier la cible minimale ; ne pas la déduire du seul navigateur.

La V1 fonctionne sans serveur externe, sans compte en ligne et sans port réseau ouvert. L'interface web embarquée n'implique pas un accès par Internet. Une version navigateur multi-poste est une évolution distincte, qui nécessiterait de revoir l'architecture et le cadre d'hébergement.

### 10.1 Organisation du code

- `domain/` : calculs BEWE, règles de manque, sélection de visite index, normalisation déterministe.
- `application/` : commandes patient/visite, validation, import, export, sauvegarde/restauration.
- `infrastructure/` : base chiffrée, migrations, coffre de clés, fichiers et exports de données.
- `ui/` : écrans, composants accessibles, odontogramme, revue de données.
- `research/` : dictionnaire, schémas d'export, spécification des extractions et scripts d'analyse.
- `tests/fixtures/` : cas entièrement fictifs, incluant cas limites et formats historiques synthétiques.

La couche interface n'exécute pas de SQL libre. Commandes typées et validées côté natif. Contraintes en base pour scores, références et unicité de provenance. Une migration est versionnée, transactionnelle quand possible et précédée d'une sauvegarde vérifiée.

### 10.2 Entités minimales

| Entité | Relations et contenu |
|---|---|
| Patient | Identifiant stable ; identité clinique séparée logiquement |
| Encounter | N visites pour 1 patient ; date, service, version, statut |
| ClinicalAssessment | Données structurées à la visite : TCA, habitudes, symptômes |
| Exposure | N expositions datées par visite ; fréquence, quantité, temporalité |
| Medication | N traitements documentés, dates/source |
| BeweAssessment | Une évaluation par version de visite, total calculé/historique séparés |
| BeweSextant | Six positions anatomiques uniques par évaluation |
| OralFinding | Constats complémentaires, localisation et méthode |
| CareNeed / PreventionAction / Referral | Besoin, intervention et orientation distincts |
| SourceDocument / SourceRecord | Provenance d'import et valeur brute immuable |
| FieldProvenance / Adjudication | Lien champ-source, décision, motif et auteur |
| ResearchProject / ResearchEligibility | Périmètre et statut d'utilisation par projet |
| ExportSnapshot | Filtres, variables, empreinte, version schéma et instantané |
| AuditEvent | Création/modification/validation/fusion/export/restauration |
| Attachment | Référence chiffrée, type, consentement si nécessaire, métadonnées minimales |

Ne pas créer un modèle générique “tous les champs dans un texte JSON” qui rendrait les exports incontrôlables. Un JSON versionné reste acceptable pour des extensions optionnelles explicitement décrites. La base clinique et les jeux d'étude doivent pouvoir évoluer sans réécrire l'histoire des consultations.

### 10.3 Protection et continuité

- Clé aléatoire générée par une bibliothèque reconnue, conservée dans le coffre du système ; jamais codée en dur, stockée dans le dépôt ou imprimée dans les logs.
- Stratégie de déverrouillage local et verrouillage à l'inactivité/veille ; réauthentification avant export nominatif. Le stockage dans le coffre OS ne remplace pas le contrôle d'accès de l'application.
- Fichiers joints, exports temporaires, journaux et sauvegardes inclus dans le modèle de menace. Ne pas chiffrer uniquement le fichier principal en oubliant WAL, temporaires et pièces jointes.
- Aucune télémétrie clinique, CDN, police distante, envoi de crash contenant des données, synchronisation grand public ou sauvegarde dans un dossier iCloud/Dropbox par défaut.
- Sauvegarde chiffrée atomique, horodatée, versionnée ; destination choisie et autorisée. Vérification d'intégrité et journal du succès. Ne pas copier une base ouverte avec une méthode susceptible de produire un état incohérent.
- Prévoir une récupération sur un autre ordinateur : sauvegarde avec mécanisme de récupération indépendant du seul coffre de l'ancien poste. Utiliser une implémentation cryptographique éprouvée ; aucune cryptographie maison. Tester la perte simulée du poste avec des données fictives.
- Restauration : vérifier version/intégrité, montrer ce qui sera remplacé, sauvegarder l'état actuel, puis restaurer sans fusion implicite. Mauvais mot de passe ou fichier corrompu : aucun écrasement.
- Journal d'audit conservé dans l'espace chiffré. Les logs techniques ne contiennent ni nom ni texte clinique ni secret. Ne pas prétendre à une inviolabilité face à l'administrateur du poste.
- Les CSV exportés peuvent être en clair : choix explicite de destination et avertissement contextuel ; pas de fichier nominatif créé automatiquement sur le bureau.

## 11. Cadre institutionnel de soin et de recherche

La conservation locale de données hospitalières sur un ordinateur personnel doit être validée par l'établissement. Le fonctionnement hors ligne ne dispense pas des règles de protection, des droits d'accès ou de la continuité du dossier. Associer la DSI/le RSSI, le DPO et la structure de recherche pour définir le circuit autorisé pour ce recueil statistique local, distinct du dossier de soin institutionnel.

Pour l'étude rétrospective : faire qualifier le projet et son traitement de données. La MR-004 peut être pertinente pour une réutilisation de données de soins, mais sa conformité ne se déduit pas du seul mot “rétrospectif” [7]. Documenter responsable de traitement, information des personnes et droits, éventuelles oppositions, accès, durée de conservation et avis/démarches requis. Les règles pour les mineurs sont à préciser si concernés. Ne pas assimiler avis éthique, formalité CNIL et conformité de l'application.

Ne pas ajouter un simple bouton “consentement RGPD” prétendant régler ces questions. L'application enregistre le statut prévu par le projet : information délivrée/version/date, opposition ou consentement si ce dernier est requis, restriction d'usage et référence du protocole. La prise en charge reste distincte de l'utilisation pour la recherche.

Une évolution vers un hébergement externalisé nécessitera une analyse HDS adaptée [8]. Ne pas annoncer une certification HDS de l'application locale. La pseudonymisation n'est pas une anonymisation : la combinaison âge/service/diagnostic/expositions peut rester identifiante.

Le prototype peut être construit et testé sur données fictives pendant ces démarches. L'utilisation réelle et la reprise des identités hospitalières doivent suivre le circuit institutionnel validé. Ce point ne bloque pas la conception ni les tests techniques.

## 12. Exports scientifiques et reproductibilité

La V1 propose un export statistique/recherche pseudonymisé, limité au projet. Aucun export clinique destiné au patient ou au dossier de soin n'est demandé. L'export statistique exclut par défaut nom, prénom, date de naissance complète, identifiant hospitalier, texte libre, chemins de fichiers, documents source et photos.

Schéma V1 de l'extraction recherche :

- `patients.csv` : study_id et caractéristiques nécessaires, avec précision des âges.
- `visits.csv` : study_id, visit_study_id, statut index, variables à la consultation, période généralisée ou date si autorisée et nécessaire.
- `bewe.csv` : visit_study_id, valeurs par sextant disponibles, total dérivé, total historique, origine retenue, complétude.
- `exposures.csv` : expositions positives, négatives explicitement recherchées et statuts de manque, temporalité et unité.
- `care_actions.csv` : besoins, prévention et orientations, statuts distincts.
- `dictionary.json` et version lisible : codes, libellés, unités, valeurs permises, définitions et règles dérivées.
- `manifest.json` : projet, horodatage, versions du logiciel/formulaire/dictionnaire, règles de sélection, nombre de lignes, empreintes des fichiers et exclusions.
- `quality_report` : effectifs et manques par variable, anomalies exclues, données historiques et prospectives séparées.

Un CSV ne contient pas de formules actives : neutraliser l'injection de formule à l'ouverture dans un tableur, en particulier pour les champs commençant par `=`, `+`, `-`, `@`, tout en conservant correctement les valeurs numériques typées. Documenter la transformation.

Les codes de manque sont exportés dans des colonnes dédiées ; pas de 999 ou de -1 indistincts. Les dates exactes ne sortent que si nécessaires et autorisées ; privilégier période ou délai relatif. Les identifiants de recherche ne sont pas des hachages simples des noms. Toute suppression de petites cellules pour une diffusion externe doit être cohérente, y compris pour empêcher leur déduction par soustraction.

Un instantané utilisé pour un article est figé. Une correction ultérieure crée un nouveau fichier et une nouvelle version, pas une substitution invisible. Fournir des scripts R ou Python reproductibles pour le rapport descriptif à partir de ces seuls exports ; ne pas intégrer une dépendance au fichier nominatif dans les scripts. Les statistiques de production ne sont implémentées qu'après validation du plan d'analyse.

## 13. Critères d'acceptation

### 13.1 Clinique et données

1. Créer un patient fictif, remplir une visite partielle, fermer et rouvrir : retrouver exactement les valeurs et leurs statuts.
2. Une question jamais renseignée reste manquante ; un zéro explicite reste zéro.
3. Saisir tous les cas BEWE de la section 7 : résultats exacts aux seuils et aucun total complet artificiel.
4. Sélectionner une dent dans chaque sextant : vérifier anatomie et orientation, au clavier et à la souris.
5. Modifier une consultation validée : voir ancienne et nouvelle valeur, auteur, date, motif ; conserver l'export antérieur.
6. Terminer une saisie incomplète : aucun document généré, manques conservés, aucune affirmation de normalité inventée.
7. Enregistrer un ancien vomissement arrêté et une absence actuelle : conserver les deux sans contradiction artificielle.
8. Importer un total historique sans sextants : aucun faux sextant n'est créé.

### 13.2 Import et recherche

9. Importer un classeur fictif avec plusieurs feuilles, une synthèse et des doublons : ne pas créer de patients depuis les totaux.
10. Réimporter le même fichier : pas de duplication ; importer une version modifiée : revue des différences.
11. “1–5 (7)”, cinq sextants, âge en classe et date ambiguë : anomalies visibles, rien n'est inventé.
12. Deux personnes homonymes : aucune fusion automatique ; fusion erronée manuelle annulable.
13. Définir un projet et choisir la visite index : une seule visite par patient dans l'analyse principale, selon la règle annoncée.
14. Exporter : vérifier toutes les colonnes contre une liste autorisée ; aucune identité, note libre ou métadonnée sensible inattendue.
15. Recalculer le rapport depuis l'instantané : mêmes N, mêmes exclusions, mêmes résultats. Afficher n/N pour les proportions.
16. Retirer l'éligibilité recherche d'un dossier : prochain export l'exclut selon les règles du projet, sans effacer les soins.

### 13.3 Sécurité et fiabilité

17. Base copiée hors application : illisible sans clé ; vérifier aussi pièces jointes et sauvegardes.
18. Mode avion : toutes les fonctions cliniques et d'export local restent fonctionnelles.
19. Mauvaise clé, sauvegarde corrompue, espace disque insuffisant : erreurs explicites, absence de faux succès et de perte silencieuse.
20. Restaurer une sauvegarde sur un environnement de test vierge avec mécanisme de récupération : vérifier patients, visites, audit et documents.
21. Vérifier l'absence de données dans les logs, captures de test et dépôt ; vérifier les sorties réseau.
22. Migration de schéma : sauvegarde préalable, comparaison des effectifs et valeurs, stratégie documentée en cas d'échec.

## 14. Jalons de réalisation

**Jalon 0 — Audit technique et preuve de stockage.** Confirmer OS et autorisations d'installation ; démontrer ouverture/fermeture d'une base chiffrée, verrouillage et récupération avec données fictives. Livrable : décision d'architecture et installation reproductible.

**Jalon 1 — Consultation fonctionnelle.** Dossier, formulaire noyau, BEWE, sauvegarde automatique, validation/amendement et clôture sans document. Livrable : application locale utilisable sur cas fictifs.

**Jalon 2 — Reprise rétrospective.** Import XLSX/CSV, provenance, dédoublonnage, revue des ambiguïtés et historique. Livrable : rapport de reprise sur jeux synthétiques ; préparation du mapping réel après export fiable du classeur.

**Jalon 3 — Projet de recherche.** Critères, visite index, complétude, export pseudonymisé et instantané. Livrable : rapport descriptif reproductible sur données fictives, sans faux résultats présentés comme réels.

**Jalon 4 — Préparation de l'usage réel.** Validation clinique par Franck, tests de restauration, décisions institutionnelles, guide de reprise et premier import contrôlé dans l'environnement autorisé. Livrable : anomalies restantes et limites documentées.

**Après V1 seulement :** photos standardisées, module salivaire, questionnaires validés, suivi organisé, utilisateurs multiples ou version Internet. Aucun de ces éléments n'est nécessaire pour commencer l'étude rétrospective descriptive.

## 15. Points à confirmer avec Franck et l'équipe

### Nécessaires pour fixer le protocole rétrospectif

- Période couverte et dates de consultation réellement disponibles.
- Nombre de dossiers après reconstruction du classeur et dédoublonnage ; pas de décompte fiable à partir des seules chaînes textuelles extraites.
- Sens des feuilles et caractère exhaustif ou sélectif du dépistage.
- Définition historique de « vomissements » et des classes d'âge ; conservation du coefficient masticatoire uniquement en source historique. Les protocoles futurs sont définis dans la section 6.7 ; ne pas les appliquer rétroactivement sans confirmation.
- Disponibilité d'autres fiches de consultation et d'un accès autorisé au dossier TCA pour compléter les données explicitement documentées.
- Inclusion éventuelle de mineurs ; interlocuteurs recherche/DPO ; circuit d'information et d'opposition.

### Vérifications techniques restantes, sans réouvrir les décisions

- Version et architecture réelles du Mac pour le packaging ; ne pas déduire depuis le navigateur.
- Destination autorisée des sauvegardes et mécanisme de récupération sur nouveau poste.
- Nomenclatures locales de service et source des diagnostics ; garder le texte d'origine si non confirmé.
- Préférence finale de thème facultative : conserver les deux propositions, démarrer avec Iris.

Ces vérifications ne bloquent pas la construction sur données fictives. Les décisions de durée, page unique, examen, protocoles, HBD et absence de document de sortie sont déjà fixées.

## 16. Références consultées et sources de vérification

Les références ci-dessous soutiennent les définitions et le cadrage. Les choix de produit, seuils de complétude, fenêtres 7/28 jours et modalités statistiques proposées sont des décisions de protocole à valider, pas des conclusions automatiquement tirées de ces sources.

1. Bartlett D, Ganss C, Lussi A. Basic Erosive Wear Examination (BEWE): a new scoring system for scientific and clinical needs. Clinical Oral Investigations. 2008;12 Suppl 1:S65–S68. DOI : 10.1007/s00784-007-0181-5. https://pmc.ncbi.nlm.nih.gov/articles/PMC2238785/
2. Eating disorders and oral health: a matched case-control study. Notice PubMed, PMID 22288922. https://pubmed.ncbi.nlm.nih.gov/22288922/ — antériorité d'une comparaison clinique TCA/témoins ; ne fournit pas de données sur la CMME.
3. Factors affecting the dental erosion severity of patients with eating disorders. BioPsychoSocial Medicine. 2014;8:25. DOI : 10.1186/1751-0759-8-25. https://pubmed.ncbi.nlm.nih.gov/25904974/ — étude des facteurs associés ; ne permet pas de conclure à l'efficacité causale d'une prévention à partir de dossiers transversaux.
4. Dental erosion, oral hygiene, and nutrition in eating disorders. Notice PubMed, PMID 9062844. https://pubmed.ncbi.nlm.nih.gov/9062844/ — comporte des patients revus ; souligne que des travaux de suivi existent déjà.
5. STROBE. Checklists et recommandations pour études observationnelles. https://www.strobe-statement.org/ et https://www.strobe-statement.org/checklists/
6. RECORD. Recommandations pour la description d'études utilisant des données de santé recueillies en routine. https://www.record-statement.org/checklist.php
7. CNIL. Recherches santé : formalités et méthodologie de référence MR-004. https://www.cnil.fr/fr/recherches-sante-quelles-formalites et https://www.cnil.fr/sites/default/files/2024-01/mr-004.pdf — qualification à vérifier institutionnellement, avec les textes en vigueur au lancement.
8. Agence du Numérique en Santé. Hébergement des données de santé. https://esante.gouv.fr/labels-certifications/hebergement-des-donnees-de-sante
9. BMC Oral Health. Aims and scope. https://link.springer.com/journal/12903/aims-and-scope
10. Clinical Oral Investigations. Aims and scope. https://link.springer.com/journal/784/aims-and-scope
11. Journal of Eating Disorders. Aims and scope. https://link.springer.com/journal/40337/aims-and-scope
12. Tauri. Documentation de sécurité. https://v2.tauri.app/security/
13. Zetetic. SQLCipher. https://www.zetetic.net/sqlcipher/

Sources consultées le 23 septembre 2026. La bibliographie clinique est une base ciblée pour le cadrage, à compléter pour l'introduction d'un manuscrit ; aucune revue systématique exhaustive n'est revendiquée. Les métadonnées complètes des références 2–4 seront importées depuis PubMed lors de la rédaction bibliographique finale.
