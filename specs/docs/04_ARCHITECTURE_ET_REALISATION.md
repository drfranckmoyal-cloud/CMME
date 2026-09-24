# Architecture et réalisation

## Cible

Application de bureau locale mono-utilisateur, UI française. Proposition : Tauri, React, TypeScript, couche native Rust et SQLCipher. **Vérifier les versions, licences, minimum macOS et intégration SQLCipher au moment du développement** ; aucune version de dépendance n'est arbitrairement figée dans ce dossier. La pile peut être adaptée au dépôt existant si les garanties et le rendu sont maintenus ; noter toute divergence.

Séparer domaine pur, cas d'usage, stockage et UI. Les invariants doivent être partagés entre formulaire/import/export. L'interface ne doit pas écrire directement du SQL ni déclarer un succès sans résultat de la commande native. Isoler le mode démo du profil clinique réel, avec fichiers de données distincts.

## Stockage et transactions

Entités : Patient, Encounter, révisions d'Encounter, ClinicalAssessment, Exposure, Medication, BeweAssessment, BeweSextant, OralFinding, PreventionAction, CareNeed, Referral, SourceDocument, SourceRecord, FieldProvenance, Adjudication, ResearchProject, ResearchEligibility, ExportSnapshot, AuditEvent.

- Patient possède N visites ; l'âge et le diagnostic sont datés à la visite, jamais recalculés en écrasant l'historique.
- Un score de sextant est unique par évaluation et clé anatomique. Le total dérivé est calculé côté domaine.
- Les imports bruts sont immuables ; normalisation proposée et données validées séparées.
- Révisions et journal de modification sont écrits dans la même transaction que les données.
- Contrôle de version optimiste pour éviter les écrasements entre deux fenêtres locales ; afficher conflit plutôt qu'écraser.
- Clôture = transition de statut avec contrôle de validité ; une correction d'une visite clôturée crée un amendement.
- Migrations versionnées, testées sur copies, sauvegarde préalable, aucune destruction silencieuse.

## Sauvegarde en cours de saisie

Sauvegarder après une courte temporisation de saisie et aux sorties de champ pertinentes. Afficher `modifications en cours`, `enregistrement`, `enregistré à…`, ou `échec, réessayer`. Flush avant changement de dossier et fermeture normale. Tester une fermeture forcée : récupérer le dernier état confirmé, sans promettre la survie d'une frappe jamais écrite.

Valider les valeurs finales sans effacer les saisies temporairement incomplètes (exemple décimale en cours). Conserver le brouillon si une valeur n'est pas encore acceptable pour clôture. Les textes libres ne doivent pas être tronqués ni transformés silencieusement.

## Chiffrement et récupération

Clé de base générée par bibliothèque éprouvée, coffre OS et déverrouillage local cohérents. Verrouillage après inactivité/veille paramétrable. La mise en veille doit fermer l'accès visible aux données. Journaux exempts de données cliniques.

Vérifier WAL/journaux/temporaires et documents source, pas seulement le fichier principal. Sauvegarde cohérente d'une base ouverte via mécanisme supporté ; archive chiffrée, écriture atomique, contrôle d'intégrité et succès journalisé. Prévoir restauration sur nouveau Mac avec mécanisme de récupération indépendant du seul coffre de l'ancien poste. Ne jamais envoyer la clé ou le mot de passe à un service.

Restauration : examiner version/intégrité, indiquer remplacement et effectifs, sauvegarder l'état courant, puis restaurer de façon transactionnelle. Corruption, mauvais secret ou version non supportée = refus sans destruction de l'état actuel. Test de restauration obligatoire avant affirmation de disponibilité pour les données réelles.

## Lots de travail

| Lot | Résultat démontrable | Porte de validation |
|---|---|---|
| 0 | Environnement inspecté, architecture et modèle documentés | Dossier de décisions, commandes de build réalistes |
| 1 | Coquille Iris/Lagune, page continue, référentiels et dossier synthétique | Navigation clavier, ordre des rubriques, aucune régression de style |
| 2 | Base locale, sauvegarde, reprise, validation et amendements | Relance, erreur disque, absence de valeurs inventées |
| 3 | BEWE complet, prévention, multi-choix et observations | Cas métier et seuils de recette passent |
| 4 | Import XLSX/CSV, provenance, revue et doublons | Réimport idempotent, anomalies conservées |
| 5 | Tableau, filtres, statistiques, export figé | Dénominateurs vérifiables, aucune fuite de notes/identité |
| 6 | Sauvegarde/restauration chiffrée et packaging | Restauration sur profil vierge, test réseau hors ligne |
| 7 | Validation utilisateur et livraison | Guide, limites, données démo séparées, installation testée |

Le chiffrement doit être prototypé tôt (lots 0–2), même si la recette complète de récupération est au lot 6. Ne pas développer toute la persistance en clair en espérant ajouter le chiffrement à la fin.

## Livrables de Claude

Code source, lockfiles et licences, schéma et migrations, composants réutilisables et tokens, tests exécutables, journal de validation, guide installation/désinstallation/mise à jour, guide recueil/import/export, procédure de récupération, liste des limites. Fournir un build de démonstration et, si l'environnement le permet, un installateur macOS ; distinguer build non signé, signé et notarié. Aucun identifiant de signature ne doit être inventé.

La validation institutionnelle d'un usage de données hospitalières reste une démarche distincte, déjà signalée dans la spécification ; elle ne bloque pas le développement et la recette sur données fictives.
