# Correctifs de parcours UI — PR #45

## Revue UI du 2026-11-17 (parcours complet en dev mode)

Branche `fix/ui-walkthrough`. P1 = fonctionnel / risque de données, P2 = le dev mode agit sur la vraie machine, P3 = finitions.

**P1**
- [x] Création de pool : nombre de disques validé par type de RAID **avant** tout effacement (`RaidType::validate_devices`, code `INVALID_POOL_LAYOUT`), aussi en dev mode
- [x] Capacité d'un pool = capacité utile du niveau RAID (`RaidType::usable_capacity`), pas la somme des disques (création et agrandissement)
- [x] Assistant de pool : choix RAID valides, capacité propre à chaque option, recommandation selon le nombre de disques et confirmation explicite de destruction des données
- [x] `Window.svelte` : couleur de texte explicite (Connexion appareil, Sécurité)
- [x] Setup : hostname validé, appliqué et persisté ; noms réservés refusés dès le formulaire ; erreurs traduites et retour à l’étape fautive
- [x] Proxy Vite : `ws: true` pour les statistiques et événements en direct

**P2**
- [x] App Docker en dev mode : connexion au vrai démon désactivée (y compris après reconnexion)
- [x] `kill_process` simulé en dev ; PID 0/1 et PiNAS protégés dans les deux modes
- [x] Terminal : racine dev fournie par `AppConfig.files_root`

Validation P1/P2 : 76 tests Rust, Clippy et build backend verts ; 51 tests frontend, build et svelte-check verts (0 erreur, 280 avertissements). Vérifié en navigateur : choix/capacités RAID, confirmation avant création, couleur des fenêtres et réception WebSocket. Les opérations système réelles restent à valider sur LibreELEC.

**P3**
- [x] Traductions : connexion, menu utilisateur et modales, erreurs de politique de mot de passe/hostname/noms réservés, rôles, groupes système et « Mes fichiers »
- [x] Connexion appareil : chemin SMB à deux antislashs ; hostname en lecture seule, édition dans Réseau ; À propos utilise la même identité réseau
- [x] FolderPicker : chemin réactif à l’emplacement, réponses obsolètes ignorées et confirmation bloquée pendant le chargement
- [x] Permissions : sélecteur de dossiers accessible et chemins d’exemple corrigés ; édition des droits depuis Partages ; refus explicite distinct de l’accès par défaut dans les deux écrans
- [x] Lanceur : Tableau de bord présent parmi les dix apps ; préférences lues sans 404 sur les clés absentes
- [x] Stockage : onglets vides et date de vérification fictive retirés, icône visible, unités binaires cohérentes (KiB/MiB/GiB/TiB), pool unique présélectionné à la création de volume
- [x] Fichiers : zéro octet affiché ; séparateur de libellé français avec espace insécable ; jours de fonctionnement localisés

Validation P3 : 76 tests Rust, Clippy et build backend verts ; 57 tests frontend, build et svelte-check verts (0 erreur, 279 avertissements). Parcours navigateur : dix apps dans le lanceur, menu utilisateur français, chemin de dossier affiché = chemin choisi, enregistrement puis restauration d’un droit de test, icône de pool, identité réseau et uptime français. `last_scrub` absent confirmé après redémarrage du backend dev. La validation SMB/NFS et stockage sur LibreELEC reste matérielle.

---


Implémentés et publiés dans la [PR #45](https://github.com/kameka22/pinas/pull/45). Leur publication ne vaut pas validation sur LibreELEC.
