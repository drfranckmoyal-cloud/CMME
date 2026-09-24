# Contrats de données prioritaires

Le dictionnaire détaillé est dans la section 6 de la spécification consolidée. Le catalogue JSON reprend ses lignes et les ajouts récents ; certaines lignes décrivent plusieurs colonnes ou une table relationnelle. Ne pas les convertir aveuglément en colonnes SQL.

## États de recueil

Chaque valeur clinique doit préserver son statut. Conceptuellement :

```ts
type MissingReason = 'not_recorded' | 'not_asked' | 'unknown' |
  'declined' | 'not_assessable' | 'not_applicable' | 'ambiguous_source';
type Observation<T> =
  | { value: T; missingReason: null }
  | { value: null; missingReason: MissingReason };
```

Le domaine valide aussi type, bornes, unités, date de référence, provenance, certitude et version de formulaire. Les types TypeScript ne suffisent pas : valider à la frontière native et contraindre la base. Un nouveau champ jamais touché prend `not_asked`, un ancien champ absent `not_recorded`. Une négation explicite est `value:false` avec `missingReason:null`.

Les métadonnées auteur/date/source doivent être conservées sans demander de les retaper à chaque clic. Un amendement documente le changement ; la dernière valeur ne remplace pas l'historique.

## Clés BEWE anatomiques

| Clé permanente | Nom | Dents | Position écran |
|---|---|---|---|
| upper_right | Supérieur droit | 17–14 | Haut gauche |
| upper_anterior | Antérieur supérieur | 13–23 | Haut centre |
| upper_left | Supérieur gauche | 24–27 | Haut droite |
| lower_left | Inférieur gauche | 37–34 | Bas droite |
| lower_anterior | Antérieur inférieur | 33–43 | Bas centre |
| lower_right | Inférieur droit | 44–47 | Bas gauche |

Les scores admis sont les entiers 0,1,2,3. Six réponses numériques : total dérivé, classe 0–2 / 3–8 / 9–13 / 14–18. Un manque ou non-évaluable : total dérivé null ; raison conservée. C'est la politique conservatrice de ce projet. Le BEWE historique exact de 0 à 18 est stocké séparément. Ne jamais répartir un total entre six zones.

`bewe_analysis_value` est une vue déterministe, pas une saisie libre : total dérivé complet validé en priorité ; sinon total historique validé. Si les deux sont disponibles et discordants, statut de conflit jusqu'à adjudication, pas de choix silencieux. Conserver l'origine retenue et le motif d'exclusion des analyses standard. Le rapport exploratoire antérieur a inclus certaines valeurs signalées ; ce comportement ne dispense pas de la validation finale.

## Prévention

`prevention_protocol`: `none | moderate | advanced | custom`, avec état de manque séparé.
`hbd_teaching`: observation booléenne indépendante. `prevention_protocol_legacy_raw`: texte historique intact.

Une table `PreventionAction` contient un type (`hbd_teaching`, `anti_erosion_toothpaste`, `anti_erosion_rinse`, `tooth_mousse`, `vomiting_semirigid_tray`, `other`), un statut, date/source, produit éventuel et note. Les statuts **conseillé aujourd'hui / déjà utilisé selon le patient / réalisé pendant la consultation / remis aujourd'hui / refusé / inconnu** sont des faits différents. Ils peuvent coexister pour une action : utiliser plusieurs événements ou attributs distincts, pas un sélecteur exclusif qui perd l'information.

Les cases de la maquette représentent une sélection rapide, pas ce modèle complet. Choisir Avancé propose trois actions ; la confirmation enregistre leurs statuts. La gouttière semi-rigide n'est jamais proposée automatiquement par ce bouton. Route Tooth Mousse = directe/gouttières/les deux/inconnue ; la proposition 10 min/j ne concerne que les gouttières. Préserver les adaptations si le protocole change ; afficher les changements avant de remplacer des valeurs confirmées.

Importer un ancien « enseignement HBD » conserve ce libellé et crée une proposition d'action historique, **sans affirmer à lui seul qu'un enseignement a effectivement été réalisé**. Même principe pour les composantes d'un ancien « Avancé ».

## Alimentation : deux sélections multiples

Boissons : sodas sucrés, sodas sans sucre, jus de fruits, boissons énergisantes, boissons pour sportifs, eau citronnée/aromatisée acide, autre.

Aliments : agrumes, autres fruits acides, vinaigre/vinaigrettes, pickles/aliments marinés, bonbons acidulés, autre.

Chaque groupe possède son état de recueil (renseigné, aucune exposition rapportée, manquant avec raison) et son champ libre. Les options multiples créent des lignes d'exposition, pas une chaîne de texte concaténée. Une catégorie positive et « aucune » ne peuvent pas coexister. Le choix Autre n'est pas obligatoire pour saisir une précision. Un texte libre seul reste du texte, pas une catégorie inventée.

Pour chaque exposition, fréquence et quantité peuvent être exactes ou des plages : `min`, `max`, `unit`, `reference_period`, `precision`. Ne pas convertir « 2 à 3 » en 2,5 observé. Une zone de précision globale peut accélérer la consultation ; le texte ne remplace pas les champs structurés lorsqu'ils sont renseignés.

## Observations et export

`consultation_observations` = texte libre final, enregistré avec la visite. `referral_note` et notes de prévention sont distinctes. Les textes libres sont exclus de l'export recherche par défaut, y compris profession précise, noms de produits saisis avec notes et chemins de fichiers.

Un export pseudonymisé suit une liste blanche de variables. Inclure précision, raisons de manque, origine historique/prospective et sélection de la visite index. Aucun hachage simple du nom comme identifiant d'étude. Les exports figés ne changent pas après correction d'une visite.
