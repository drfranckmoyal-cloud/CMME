# Recette V1 — résultats réellement exécutés (24/09/2026)

Légende : **Auto** = test automatique exécuté (`cargo test -p cmme-core`, 27 tests + 1 test trousseau manuel) ;
**Écran** = vérifié dans l'application réelle sur ce Mac (profil démonstration) ; **Non exécuté** = à faire.

Cas fournis `specs/tests/cas_metier.json` : les 21 cas sont lus directement dans le fichier et passent
(`tests/cas_metier.rs::tous_les_cas_metier_passent`).

| # (06_RECETTE) | Contrôle | Résultat |
|---|---|---|
| 1 | Dossier vide : six rubriques dans l'ordre, rien d'inventé | Auto (`dossier_vierge_sans_reponse_inventee`) + Écran |
| 2 | Raccourci de rubrique : défilement, rien masqué | Écran |
| 3 | Saisie partielle, fermeture, réouverture à l'identique | Auto (`saisie_partielle_fermee_puis_rouverte_a_l_identique`) |
| 4 | Erreur disque : pas de faux « enregistré », réessayer | Auto (`erreur_disque_sans_faux_succes`) ; affichage d'erreur non provoqué à l'écran |
| 5 | A puis B puis A sans mélange | Auto (`a_puis_b_puis_a_sans_melange`) |
| 6 | Seuils BEWE, mapping mandibulaire, total incomplet nul | Auto (3 tests) + Écran (9 → classe 9–13 ; N.É. → pas de total) |
| 7 | HBD seul + Aucun protocole | Auto (`prevention_hbd_independant_et_protocoles`) |
| 8 | HBD, Modéré, Avancé : HBD conservé, gouttière non renseignée | Auto + Écran |
| 9 | Tooth Mousse direct sans durée ; gouttière 10 min modifiable | Auto (+ contrainte en base) |
| 10 | Ancien « Avancé » : composantes non reconstruites | Auto (`csv_synthetique_de_bout_en_bout`) |
| 11 | Boissons/aliments multiples, « aucun » exclusif, décocher sans perte | Auto (`alimentation_…`) |
| 12 | Terminer avec observations : aucun document | Auto + Écran (récapitulatif, verrouillage, aucun fichier produit) |
| 13 | Amendement motivé, ancien état consultable, export figé inchangé | Auto (`terminer_valide_…`, `export_liste_blanche_…`) |
| 14 | Import synthétique : tranches, exacts, zéro, manque, plage, hors domaine | Auto + Écran (aperçu, revue, validation en lot) |
| 15 | Réimport identique refusé ; version modifiée soumise à revue | Auto |
| 16 | Homonymes non fusionnés ; fusion manuelle annulable | Auto (`classeur_multi_feuilles_…`) |
| 17 | Synthèses, séparateurs, en-têtes répétés ne créent pas de visites | Auto |
| 18 | Filtres communs tableau/statistiques, N affichés | Auto + Écran |
| 19 | Zéro résultat : « aucune donnée analysable » | Auto (`jeu_demo_statistiques_de_recette`) |
| 20 | Démo BEWE [0,3,11,2,14,5,null,7] : N=7, médiane 5, 2/3/1/1, ≥9 = 2/7 | Auto + Écran |
| 21 | Export liste blanche, aucune identité/note, formules neutralisées | Auto (fonction de neutralisation en place ; aucun champ texte n'est exportable) |
| 22 | Dictionnaire, manifeste, empreintes ; recalcul externe identique | Auto (script Python lancé par le test) |
| 23 | Base illisible sans clé ; aucun fichier annexe en clair | Auto (`base_illisible_sans_la_bonne_cle`) |
| 24 | Sauvegarder, modifier, restaurer ; poste vierge | Auto (`sauvegarder_modifier_restaurer_sur_un_poste_vierge`) |
| 25 | Mauvaise phrase, sauvegarde tronquée ou altérée : état intact | Auto |
| 26 | Hors réseau | **Non exécuté en mode avion.** Par construction : aucune dépendance réseau, CSP limitée à l'IPC local |
| 27 | Clavier, focus, BEWE lisible sans couleur | Partiel (Écran : chiffres et « — »/« N.É. » ; parcours clavier complet non testé) |
| 28 | Pilote 10–15 min | **À faire par Franck** (non automatisable) |

Trousseau macOS : écriture/lecture/suppression réelles vérifiées (`tests/trousseau.rs`, lancé avec `--ignored`).
Version finale (`target/release`) : construite, se lance, affiche « stockage : trousseau macOS ». L'ouverture d'un
profil dans cette version n'a pas pu être cliquée pendant la session (fenêtre de notification macOS au premier plan) :
**à confirmer par un clic de Franck**.
