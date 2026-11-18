# PiNAS — Plan actif : cohérence visuelle et navigation

Actualisé le 17 novembre 2026. Ce plan remplace le suivi historique de remédiation ; il ne déclare pas la refonte implémentée.

## Travaux sortis du plan actif

Les correctifs de parcours P1/P2/P3 sont intégrés à `master` par la [PR #45](https://github.com/kameka22/pinas/pull/45), mergée le 17 novembre 2026 (squash `6b54347`). Détail, commits et validations : [historique de la revue UI](docs/archive/UI_WALKTHROUGH_PR45.md). Ne pas les refaire dans ce chantier ; conserver leurs protections et leurs tests.

Le [plan historique de septembre](docs/archive/REMEDIATION_PLAN_2026-11-17.md) est conservé comme archive, avec ses décisions et descriptions datées, dont certaines sont désormais obsolètes. Il ne constitue plus une consigne de travail. La roadmap fonctionnelle à long terme reste dans [TODO.md](TODO.md).

## Cadre du chantier

- Les sept phases R1–R7 se font sur une seule branche : `feature/ui-consistency`, jamais sur `master`. Push, merge et publication exigent l’accord explicite de Laurent.
- Conserver le bureau à fenêtres. Une application = une fenêtre de travail ; listes, fiches et assistants dans cette fenêtre. Une seule confirmation courte à la fois.
- Réutiliser les fonctions et les contrôles de sécurité existants. Aucun changement implicite des droits, des validations RAID, des chemins ou du comportement des opérations destructives.
- Les maquettes utilisent des données fictives et ne font aucun appel à l’API. Elles proposent une disposition, pas de nouvelles capacités backend.
- Référence détaillée et inventaire complet : [UI_REDESIGN_PROPOSAL.md](docs/UI_REDESIGN_PROPOSAL.md).

## Objectif et ordre de travail retenu

La priorité demandée par Laurent est la cohérence **partout** : structure, composants et comportements communs dans l’interface existante. Les premières maquettes ont été jugées insuffisantes ; elles restent une exploration non validée et ne sont pas la cible à reproduire. La direction sera examinée dans l’instance de développement réelle.

Conserver l’identité et le bureau PiNAS. Réutiliser les composants existants quand ils conviennent et centraliser leurs variantes. Ne pas transformer tous les écrans en tableaux : synthèse, catalogue, explorateur et terminal gardent une structure adaptée à leur usage, avec des commandes cohérentes.

## Lots et critères de fin

Les numéros R2–R4 sont réordonnés ici : le panneau complet vient avant les refontes de parcours détaillées. Décision de Laurent : une seule branche `feature/ui-consistency` pour les sept phases R1–R7, avec des commits distincts par phase et un état utilisable à chaque jalon. Ne pas créer une branche ou une PR par lot.

| Lot | Travail et livrable | Dépendance | État / critère de fin |
|---|---|---|---|
| R0 | Audit global et cadrage | — | Audit fait ; premières maquettes exploratoires non retenues comme cible ; présent plan actualisé |
| R1 | Socle commun : styles et composants d’en-tête, actions, boutons, champs, onglets, sections, statuts, tableaux et sauvegarde | R0 | ✅ Fait (`ui/settings/*`, démontré sur Réseau et Heure/langue) : composants réutilisables démontrés dans deux rubriques existantes ; conventions vérifiables et états clavier/chargement/erreur définis |
| R2 | Harmonisation des douze rubriques du panneau, accueil et navigation compris | R1 | ✅ Fait pour les douze rubriques (Utilisateurs : en-tête, onglets et états seulement, le reste en R3 ; accueil et navigation latérale inchangés) : premier livrable global consultable en dev ; titres, espacements, contrôles et actions homogènes dans chaque rubrique et sous-vue |
| R3 | Partages, Utilisateurs/groupes et permissions : listes, fiches, créations et choix de dossier intégrés | R2 | À faire : aucun empilement de formulaires/modales ; retour préserve le contexte ; accès par défaut et refus explicite distincts ; SMB/NFS respectent le modèle existant |
| R4 | Stockage : synthèse, disques/pools/volumes, SMART, alimentation, snapshots, montage, maintenance et assistants | R3 | À faire : actions contenues à 900 px ; fiches et assistants dans la fenêtre ; confirmations destructives et validations RAID conservées |
| R5 | Dashboard, Fichiers, Centre d’applications, Docker, processus/services, Kodi, affichage externe, terminal et cadres des apps tierces | R4 | À faire : styles et comportements communs sur toutes leurs sous-vues, fonctionnalités spécifiques préservées, service indisponible distinct d’une liste vide |
| R6 | Bureau, fenêtres, dock, lanceur, recherche, widgets, notifications, compte, connexion et sept étapes d’installation | R5 | À faire : cohérence des éléments transversaux et parcours de première utilisation ; focus, zoom et mouvement réduit vérifiés |
| R7 | Revue globale et validation de release | R6 | À faire : inventaire intégral contrôlé, comparaisons avant/après, FR/EN et tests logiciels ; validation VM/Pi tracée séparément |

État R2 : toutes les rubriques utilisent `SettingsHeader`, sections, lignes, interrupteurs (contrôlés quand ils déclenchent une action serveur), badges, tableaux et états chargement/erreur/vide communs ; les erreurs passent par toasts, plus de chaîne anglaise en dur ni de `console.error` silencieux ; formulaires intégrés au lieu de modales (mot de passe SSH, ajout d’imprimante, export NFS) ; suppressions (export NFS, imprimante, planification) confirmées par `ConfirmModal` ; quitter une rubrique avec des modifications non enregistrées demande confirmation (`stores/unsavedChanges.ts`).

Le socle R1 est développé avec deux cas réels (par exemple Réseau et Heure/langue), puis généralisé en R2. Les changements de structure des permissions et du stockage attendent R3/R4 ; R2 harmonise déjà leur présentation lorsqu’ils sont accessibles depuis le panneau.

## Règles communes à appliquer dès R1

- **En-tête** : titre, description courte et emplacement stable de l’action principale ; éviter les titres et boutons de création dupliqués.
- **Commandes** : mêmes tailles, variantes, icônes, libellés et états désactivé/chargement ; actions dangereuses séparées.
- **Formulaires** : alignements, aides et erreurs au même endroit ; barre Enregistrer/Annuler commune pour les modifications différées. Les actions immédiates restent identifiables et ne reçoivent pas un faux bouton Enregistrer.
- **Navigation** : onglets et retours prévisibles ; filtre, sélection et position de liste conservés ; modifications en attente protégées lors d’un départ.
- **États** : mêmes présentations de chargement, vide, erreur, accès refusé, service arrêté et matériel non supporté, avec une action utile quand elle existe.
- **Densité et lisibilité** : palette et espacements centralisés, une famille d’icônes, statuts avec texte, tableaux sans débordement de la fenêtre.

R3 et R4 appliqueront la règle : une application, une fenêtre de travail ; fiches et assistants intégrés ; une seule confirmation courte à la fois. Toute exception utile à un parcours est documentée.

## Suivi écran par écran

L’inventaire de [la proposition UI, section 6](docs/UI_REDESIGN_PROPOSAL.md#6-revue-de-toutes-les-familles-décrans) sert de liste de couverture. Pour chaque lot implémenté, consigner ses écrans et sous-vues, composants partagés adoptés, modales restantes, états vérifiés et captures avant/après. Ne pas déclarer une famille terminée après la seule migration de son écran d’accueil.

## Hors périmètre

Pas de nouvelle fonctionnalité backend, de changement des droits ou de nouveau modèle de stockage. Thème sombre complet, métriques historiques et roadmap fonctionnelle restent des chantiers distincts. Le prototype [historique](docs/mockups/ui-redesign/index.html) utilise des données fictives ; son thème et ses indicateurs ne constituent pas des fonctionnalités livrées.

## Validation de chaque lot implémenté

Build et tests Rust, Clippy/rustfmt, tests frontend, svelte-check et build frontend. Parcours navigateur ciblés, français/anglais, fenêtre 900 × 600, largeur étroite, zoom 200 %, navigation clavier. Confirmer les états chargement, vide, erreur, service arrêté, accès refusé et matériel non supporté. Les maquettes/documentations seules ne nécessitent pas de reconstruire le produit ; vérifier leur rendu et leurs interactions.

## Validations et dépendances encore ouvertes

- Image LibreELEC : suivi de la [PR #29](https://github.com/kameka22/pinas/pull/29), distinct du chantier visuel.
- Exécuter [RELEASE_CHECKLIST.md](docs/RELEASE_CHECKLIST.md) sur VM x86_64 puis Pi : démarrage/systemd, SMB/NFS, stockage et mises à jour signées. Une case historique cochée pour une checklist écrite ne vaut pas exécution matérielle.
- Conserver les limites connues hors refonte : redimensionnement de volume, champ `CreateVolumeRequest.size` ignoré, historique métriques et événements Stockage WebSocket (TODO).
