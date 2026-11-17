# PiNAS — Plan actif : cohérence visuelle et navigation

Actualisé le 17 novembre 2026. Ce plan remplace le suivi historique de remédiation ; il ne déclare pas la refonte implémentée.

## Travaux sortis du plan actif

Les correctifs de parcours P1/P2/P3 sont implémentés et publiés dans la [PR #45](https://github.com/kameka22/pinas/pull/45). Détail, commits et validations : [historique de la revue UI](docs/archive/UI_WALKTHROUGH_PR45.md). Ne pas les refaire dans ce chantier ; conserver leurs protections et leurs tests.

Le [plan historique de septembre](docs/archive/REMEDIATION_PLAN_2026-11-17.md) est conservé comme archive, avec ses décisions et descriptions datées, dont certaines sont désormais obsolètes. Il ne constitue plus une consigne de travail. La roadmap fonctionnelle à long terme reste dans [TODO.md](TODO.md).

## Cadre du chantier

- Travailler sur une branche dédiée, jamais sur `master`. Push, merge et publication exigent l’accord explicite de Laurent.
- Conserver le bureau à fenêtres. Une application = une fenêtre de travail ; listes, fiches et assistants dans cette fenêtre. Une seule confirmation courte à la fois.
- Réutiliser les fonctions et les contrôles de sécurité existants. Aucun changement implicite des droits, des validations RAID, des chemins ou du comportement des opérations destructives.
- Les maquettes utilisent des données fictives et ne font aucun appel à l’API. Elles proposent une disposition, pas de nouvelles capacités backend.
- Référence détaillée et inventaire complet : [UI_REDESIGN_PROPOSAL.md](docs/UI_REDESIGN_PROPOSAL.md).

## Lots et critères de fin

| Lot | Travail | État | Critère de fin |
|---|---|---|---|
| R0 | Audit et maquettes Partages / Utilisateurs / Disques | Première version produite, validation visuelle en attente | Laurent valide les styles, la densité, la navigation et les formulaires sur le prototype |
| R1 | Socle commun : couleurs, typographie, espacements, boutons, champs, statuts, tableaux, en-têtes, onglets, formulaires et confirmations | À faire après R0 | Composants réutilisables, états focus/chargement/erreur/vide documentés, contraste et clavier vérifiés |
| R2 | Partages et Utilisateurs/groupes : liste → fiche ; création et sélection de dossier intégrées | À faire après R1 | Plus de modales empilées ; retour préserve le contexte ; droits par défaut et refus explicite restent distincts ; SMB/NFS respectent le modèle existant |
| R3 | Stockage : synthèse, listes et fiches disque/pool/volume ; assistants, SMART, alimentation, snapshots et maintenance intégrés | À faire après R1 | Aucun débordement à 900 px ; états matériel absent/non supporté explicites ; avertissements et confirmations destructives conservés |
| R4 | Douze rubriques du panneau : en-têtes, sections, champs et barre Enregistrer/Annuler communs | À faire après R2 | Chaque rubrique et sous-vue de l’inventaire migrée ; réglages non enregistrés protégés lors de la navigation |
| R5 | Fichiers, Dashboard, Docker, Centre d’applications, processus/services, Kodi, affichage externe et terminal | À faire après R1 | Cadre commun pour chaque écran et sous-vue ; aperçu/propriétés intégrés ; service indisponible distinct d’une liste vide |
| R6 | Bureau, lanceur, dock, recherche, widgets, notifications, compte, connexion et sept étapes d’installation | À faire après R4/R5 | Cohérence complète de l’inventaire, affichage étroit et zoom, accès clavier, focus et mouvement réduit vérifiés |
| R7 | Revue finale et validation de release | À faire après R2–R6 | Tous les parcours et états relus en français/anglais ; validations logicielles et matérielles consignées |

Le thème sombre est un chantier de produit à estimer séparément ; le prototype peut suivre le thème du lecteur sans annoncer son support dans PiNAS.

## Maquettes livrées (R0)

[Prototype interactif autonome](docs/mockups/ui-redesign/index.html) : changer d’application dans la barre latérale, rechercher, ouvrir une fiche et ses onglets, parcourir la création d’un partage en quatre étapes. Les commandes indiquent leur caractère simulé. Les options avancées SMB/NFS et le sélecteur de dossier restent des intentions de disposition à détailler pendant R2.

- [x] Listes et fiches Partages, Utilisateurs et Disques.
- [x] Assistant de partage intégré : Emplacement → Accès → Options → Résumé.
- [ ] Validation visuelle par Laurent et ajustements éventuels.
- [ ] Implémentation Svelte des écrans validés.

## Validation de chaque lot implémenté

Build et tests Rust, Clippy/rustfmt, tests frontend, svelte-check et build frontend. Parcours navigateur ciblés, français/anglais, fenêtre 900 × 600, largeur étroite, zoom 200 %, navigation clavier. Confirmer les états chargement, vide, erreur, service arrêté, accès refusé et matériel non supporté. Les maquettes/documentations seules ne nécessitent pas de reconstruire le produit ; vérifier leur rendu et leurs interactions.

## Validations et dépendances encore ouvertes

- Image LibreELEC : suivi de la [PR #29](https://github.com/kameka22/pinas/pull/29), distinct du chantier visuel.
- Exécuter [RELEASE_CHECKLIST.md](docs/RELEASE_CHECKLIST.md) sur VM x86_64 puis Pi : démarrage/systemd, SMB/NFS, stockage et mises à jour signées. Une case historique cochée pour une checklist écrite ne vaut pas exécution matérielle.
- Conserver les limites connues hors refonte : redimensionnement de volume, champ `CreateVolumeRequest.size` ignoré, historique métriques et événements Stockage WebSocket (TODO).
