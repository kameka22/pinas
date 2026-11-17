# PiNAS — proposition de cohérence visuelle et de parcours

Proposition du 17 novembre 2026, fondée sur `52c408c` (PR #45). Ce document propose une direction ; il ne modifie pas le produit.

## 1. Périmètre et méthode

Revue du bureau, des neuf applications intégrées, de Docker installé, des douze rubriques du panneau, des sous-vues accessibles et des composants transversaux. Parcours dans le navigateur sur l’instance locale en mode dev, généralement dans une fenêtre de 900 × 600 px. Lecture complémentaire des composants pour les formulaires, confirmations et états qui nécessitent du matériel ou une opération système.

La connexion et les sept étapes d’installation ont été affichées dans un aperçu isolé des composants existants : aucune déconnexion, soumission ou réinitialisation de la configuration. Aucun formatage, changement réseau, arrêt de service, installation ou modification de droits pour cette revue.

Référence consultée : [UGREEN — Folder Management](https://support.ugnas.com/knowledgecenter/detail/article/en-US/770), notamment la liste des dossiers partagés et les réglages d’un dossier. Les propositions ci-dessous sont des choix pour PiNAS, pas des fonctions que cette documentation garantirait pour PiNAS. Synology est une référence d’intention donnée par Laurent, sans nouvel audit de DSM dans cette revue.

Légende : **V** = vue ouverte dans le navigateur ; **C** = parcours ou état complémentaire examiné dans le code. Les applications tierces du catalogue ne sont pas toutes installées : leur interface propre ne fait pas partie de la refonte de PiNAS.

## 2. Constats qui justifient une refonte commune

- Le parcours Utilisateurs → Permissions → Ajouter un dossier → Parcourir affiche réellement **trois modales simultanées**. Le contexte est masqué par plusieurs voiles sombres.
- Dans Stockage → Disque dur, les gros boutons de chaque disque poussent certaines actions hors de la fenêtre à 900 px ; les métadonnées passent sur plusieurs lignes.
- Les rubriques du panneau alternent entre un titre avec icône et description, des onglets seuls et des cartes sans titre de page.
- Les positions du bouton principal, les tailles de boutons, l’espacement des formulaires et la forme des interrupteurs varient d’un composant à l’autre.
- Le Centre d’applications possède déjà un bon parcours liste → fiche dans la même fenêtre. C’est une base à généraliser.
- Partages affiche une très grande tuile « Créer un partage » en plus du bouton de création de l’en-tête. La densité est très différente de celle des utilisateurs.
- Affichage externe introduit un bandeau violet très présent, puis sa propre navigation ; les autres apps utilisent principalement le bleu.
- À propos présente un décor de serveur qui gêne la lecture du bloc propriétaire à cette taille. Les informations sont dispersées en grandes cartes.
- Plusieurs écrans de service deviennent presque vides quand le service est désactivé ; Docker affiche des listes vides alors que le démon est indisponible.
- Les contrastes de certains textes secondaires et icônes sont faibles. Certaines traductions et unités restent locales, notamment dans Docker, les processus et les données renvoyées par Kodi.

## 3. Direction recommandée : un bureau NAS clair et soigné

Conserver le bureau à fenêtres, le dock et le lanceur. Donner aux applications une identité commune, avec un fond de travail gris bleuté très léger, des surfaces blanches et un bleu unique pour l’action et la sélection.

| Élément | Proposition de départ |
|---|---|
| Accent principal | Bleu `#2563EB` ; variante légère pour les sélections |
| Fond de travail | `#F6F8FC` ; cartes et zones d’édition blanches |
| Texte | `#172033` ; texte secondaire contrasté, sans multiplier les gris trop pâles |
| Statuts | Vert = sain/actif, ambre = attention, rouge = erreur/destruction ; toujours un libellé |
| Typographie | Conserver Inter/system ; corps 14 px, sous-titres 16 px, titre de page 20 px |
| Espacement | Échelle commune de 4/8/12/16/24/32 px |
| Commandes | Hauteur commune proche de 36 px ; lignes de tableau proches de 44 px |
| Arrondis | Champs et boutons 8 px ; sections 12 px ; fenêtre 14 px |
| Ombres et flou | Fenêtres et menus principalement ; surfaces de travail sobres et lisibles |
| Icônes | Même famille, tailles 16/20/24 px ; couleur surtout pour les identités d’apps et les statuts |
| Animation | Transitions courtes, environ 150–200 ms, respect du mouvement réduit |

Ces dimensions sont des valeurs à valider sur les maquettes, pas des contraintes déjà appliquées. Vérifier les contrastes, le zoom et le clavier avant de figer les couleurs.

Le côté « sexy » viendrait de la composition : alignements précis, hiérarchie claire, détails bien espacés, états soignés et transitions discrètes. Les dégradés restent utiles sur le bureau et les icônes ; ils ne deviennent pas le décor de chaque formulaire.

## 4. Quatre structures d’écran à réutiliser

1. **Vue d’ensemble** : quelques indicateurs utiles, alertes prioritaires, activité récente et accès directs. Dashboard et synthèse de stockage.
2. **Liste et fiche** : en-tête, recherche/filtres, tableau, puis fiche de la ressource dans la même fenêtre. Utilisateurs, groupes, partages, disques, conteneurs et processus.
3. **Réglages** : titre et courte explication, sections alignées, aide au niveau des champs, barre Enregistrer/Annuler quand des changements sont en attente. Réseau, sécurité, heure et services.
4. **Catalogue et fiche** : cartes ou liste de produits, fiche détaillée et progression intégrée. Centre d’applications.

Une fiche peut devenir un panneau latéral sur une grande fenêtre pour une consultation rapide. Pour une édition longue, elle remplace la liste avec un retour explicite. Une fenêtre étroite affiche une seule zone à la fois.

## 5. Règle de navigation et de modales

**Une application, une fenêtre de travail ; zéro formulaire long dans une modale ; une seule confirmation bloquante à la fois.**

| Action | Présentation cible |
|---|---|
| Créer un utilisateur ou un groupe | Page de formulaire dans la fenêtre du panneau |
| Créer un partage | Assistant dans la fenêtre Partages, avec étapes Emplacement → Accès → Options → Résumé |
| Choisir un dossier pendant un formulaire | Explorateur intégré à l’étape ou sous-vue temporaire, retour vers le formulaire préservé |
| Modifier un partage | Fiche avec Général / Accès / SMB / NFS selon les fonctions existantes |
| Modifier des droits | Section Accès commune aux fiches partage/utilisateur ; accès effectif et origine de la règle clairement distingués si les données le permettent |
| Créer ou agrandir un pool | Assistant dans la fenêtre Stockage, résumé persistant et confirmation destructive finale |
| SMART, historique et alimentation d’un disque | Onglets de la fiche disque |
| Options de montage, vérification et snapshots | Onglets ou actions de la fiche volume |
| Logs et détails d’un conteneur | Fiche avec onglets ; journal lisible et défilant |
| Renommer un fichier | Édition en ligne, avec validation et annulation clavier |
| Aperçu/propriétés d’un fichier | Volet de consultation ou vue d’aperçu dans la fenêtre Fichiers |
| Supprimer, effacer, redémarrer | Confirmation courte commune, cible et conséquence explicites |
| Profil et mot de passe | Même panneau Compte ; une confirmation si nécessaire |

Les règles d’authentification et d’autorisation ne changent pas dans un lot cosmétique. L’explication des permissions doit respecter la priorité réelle des règles utilisateur/groupes et distinguer SMB de NFS.

## 6. Revue de toutes les familles d’écrans

| Écran et couverture | Recommandation |
|---|---|
| Bureau, fenêtres, dock, lanceur — V | Conserver la métaphore de bureau ; harmoniser les icônes, la sélection, les ombres et les dimensions. Prévoir des tailles de fenêtre adaptées au contenu et des minimums qui évitent les débordements. |
| Barre supérieure, tâches, widgets, recherche et notifications — V/C | Alléger les indicateurs permanents ; présenter tâches et notifications dans des panneaux cohérents. Le dashboard et les widgets doivent partager les mêmes cartes et la même signification des métriques. |
| Connexion — V (aperçu isolé) | Même identité visuelle que l’installation ; fond local cohérent avec PiNAS, meilleure lisibilité des pictogrammes et des erreurs, mêmes champs que le reste du produit. |
| Installation : bienvenue, langue, appareil, compte, mot de passe, SSH, découverte — V | Garder le parcours guidé ; envisager de réunir compte et mot de passe pour réduire les étapes très courtes. Ajouter un vrai résumé avant soumission et conserver les erreurs auprès des champs. |
| Profil, changement de mot de passe, succès de mise à jour — C | Unifier le panneau Compte et les messages de résultat. Garder les formulaires courts, avec validation et état de sauvegarde communs. |
| Tableau de bord — V | Prioriser santé du NAS, capacité utile, services et tâches. Réduire les compteurs décoratifs et ajouter des liens vers les ressources. Employer les mêmes couleurs que Stockage. |
| Panneau : grille d’accueil et navigation — V | Conserver la grille comme entrée ; uniformiser les titres, la largeur de navigation et le niveau des onglets. Reclasser sans ajouter de fonctions : Comptes, Réseau et accès, Système, Apparence. |
| Utilisateurs — V/C | Tableau à colonnes maîtrisées ; clic sur une ligne → fiche Identité / Accès / Services. Création dans la même fenêtre ; actions rares regroupées dans un menu. |
| Groupes — V/C | Même tableau et même fiche ; membres et permissions dans des onglets. Garder visibles les protections des groupes système. |
| Politique de mot de passe — V | Rapprocher cette page de Sécurité ; régler la politique au même endroit que les autres règles d’accès. |
| Permissions et ajout de dossier — V/C | Supprimer les trois couches de modales observées. Utiliser une liste de ressources et une section d’édition, avec un sélecteur de dossier intégré. |
| Services de fichiers : SMB et NFS — V/C | Même en-tête de service, même bloc état/action et même présentation des options. Garder les options compréhensibles même lorsque le service est arrêté. Les exports NFS restent distincts des droits SMB. |
| Connexion appareil — V | Transformer cette page en fiche « Comment accéder à mon NAS » : liens, SMB/NFS, copie, ports et découverte. Donner un accès clair au réglage réseau correspondant. |
| Terminal/SSH dans le panneau — V/C | Renommer la rubrique pour éviter la confusion avec l’app Terminal. État et options dans une page commune aux services ; mot de passe dans une sous-vue courte. |
| Imprimantes — V/C | Même présentation de service que SMB/SSH ; liste d’imprimantes lorsque disponible et sous-vue d’ajout. Un état arrêté doit expliquer le rôle du service et l’action disponible. |
| Matériel et alimentation — V/C | Séparer mesures, réglages et opérations d’alimentation. Calendrier des tâches lisible ; actions immédiates isolées dans une zone dédiée et confirmées. |
| Heure et langue — V | Formulaires alignés ; fuseau choisi dans un champ de recherche unique, sans juxtaposition gênante de deux contrôles. Même barre de sauvegarde que Réseau et Sécurité. |
| Réseau : général et interfaces — V/C | Liste compacte des interfaces, fiche de l’interface pour DHCP/IP/DNS. Conserver une vue claire de la configuration effective et un avertissement avant une modification qui peut couper la connexion. |
| Sécurité : sessions, HTTPS, journal — V/C | Réunir avec la politique de mot de passe ; distinguer réglages et journal. Tableau de connexions avec filtres, sans accumuler de grandes cartes. |
| Personnalisation — V | Aperçu du bureau et thèmes nommés de manière cohérente ; mêmes contrôles de sélection que les autres pages. Le thème sombre complet vient après la consolidation des styles. |
| Mise à jour : état, confirmation, progression, résultat — V/C | Même carte de statut que les autres services ; notes et progression dans la fenêtre. Réserver le plein écran à une phase réellement bloquante. États d’échec et de reconnexion explicites. |
| À propos : général, stockage, services — V | Fiche appareil compacte, sans illustration qui recouvre les informations. Renvois vers Stockage/Réseau/Services pour les actions ; limiter les tableaux dupliqués. |
| Fichiers : liste, vues, opérations, aperçu et propriétés — V/C | Conserver l’explorateur ; simplifier les barres d’outils, améliorer les sélections et mettre les détails dans un volet. Typologie humaine des fichiers plutôt que MIME brut dans la liste. |
| Partages : liste, création, édition et droits — V/C | Remplacer les grandes cartes administratives par une liste : nom, emplacement, protocole, état, accès. Fiche avec onglets et assistant de création intégré. Retirer la seconde grande tuile de création quand une liste existe. |
| Stockage : aperçu, pools/volumes, disques, externe — V/C | Vue maître/détail ; relation Disques → Pool → Volume lisible. Réduire les gros boutons par disque à une action principale et un menu. Corriger le débordement à 900 px. |
| Stockage : création, agrandissement, SMART, alimentation, montage, vérification, snapshots, suppression — C, parcours de création déjà vérifié lors de P1 | Répartir les fonctions dans les fiches disque/pool/volume ; assistants pour les opérations longues. Une confirmation finale pour les opérations destructives. |
| Centre d’applications : catalogue, fiche, installation et désinstallation — V/C | Conserver liste → fiche. Uniformiser les actions et états, améliorer la densité du catalogue et conserver la progression au même endroit. |
| Docker : aperçu, conteneurs, images, volumes et réseaux — V/C | Même tableau et mêmes fiches que les autres ressources. Service indisponible = état explicite ; éviter de le présenter comme un inventaire vide. Indiquer que les jauges actuelles représentent le système hôte, pas des ressources Docker mesurées séparément. |
| Processus — V/C | Tableau compact avec colonnes alignées et unités communes ; fiche de détails et confirmation d’arrêt. Éviter les retours à la ligne qui rendent les lignes irrégulières. |
| Terminal applicatif — V/C | Conserver un terminal sombre ; aligner son cadre, sa typographie et ses commandes de fenêtre sur PiNAS. Garder la lisibilité des sélections et du focus. |
| Affichage externe et Kodi : accueil, télécommande, sources, paramètres, extensions, médiathèque — V/C | Harmoniser le cadre avec les autres apps. Garder les grands contrôles de la télécommande ; sources et extensions deviennent des listes et fiches. Les valeurs renvoyées par Kodi exigent une stratégie de traduction distincte. |
| Kodi autonome — C | Partager les éléments de l’intégration Kodi avec Affichage externe pour éviter deux présentations qui divergent. Ne pas retirer des fonctions selon le rôle pendant une refonte visuelle. |
| ServiceApp : statut, logs, configuration — C | Réutiliser la fiche de service et le journal communs ; ne pas ouvrir des pages de configuration vides. |
| Applications iframe et onglet externe — C | Unifier le cadre PiNAS : titre, état d’accès, chargement et ouverture externe. Le contenu des applications tierces conserve sa propre interface. |

Les formulaires conditionnés à un matériel absent ou à un service activé sont couverts par la lecture du code, sans activer les services de la machine hôte. Cette revue n’est pas une nouvelle validation fonctionnelle ou matérielle.

## 7. Trois parcours de référence à maquetter en premier

### Partages

`Liste → media → onglet Accès → modification → Enregistrer`

En création : `Liste → Nouveau partage → Emplacement (explorateur intégré) → Accès → Options → Résumé → Créer`.

Le nom et l’emplacement du partage restent visibles ; le bouton Retour ramène à la liste et conserve sa sélection/recherche. Les changements non enregistrés sont signalés avant de quitter.

### Stockage

`Liste des disques → Samsung T7 → Vue d’ensemble / SMART / Historique / Alimentation`.

`Pools → Pool1 → Vue d’ensemble / Disques / Volumes / Maintenance`.

La suppression et l’effacement apparaissent dans un menu d’actions ou une zone de danger ; elles ne sont pas de gros boutons permanents mélangés à la consultation.

### Utilisateurs

`Liste → alice → Profil / Groupes / Accès / Services`.

Le même éditeur de règles est présenté depuis Partages ou Utilisateurs, avec le contexte approprié. Une création ne force pas à ouvrir une fenêtre supplémentaire pour choisir un dossier.

## 8. Mise en œuvre proposée

1. **Maquettes et règles** : bureau, une liste, une fiche, un formulaire, un assistant et une confirmation. Valider d’abord Partages, Disques et Utilisateurs à 900 × 600 px et en fenêtre agrandie.
2. **Composants communs** : cadre d’application, en-tête, navigation, barre d’outils, tableau, fiche, section de formulaire, barre de sauvegarde, statuts, états vide/erreur/chargement et confirmation. Les styles locaux de boutons et modales sont remplacés progressivement.
3. **Parcours prioritaires** : Partages et permissions, Utilisateurs/groupes, puis Stockage et ses assistants. C’est là que le gain de cohérence et la réduction des modales sont les plus importants.
4. **Panneau complet** : migrer les douze rubriques, rapprocher les réglages apparentés et uniformiser les sauvegardes explicites/immédiates.
5. **Autres applications et bureau** : Dashboard, Fichiers, Centre d’applications, Docker, processus, Kodi, terminal, connexion et installation ; harmoniser les états transversaux.
6. **Validation** : comparaison des captures, navigation clavier et focus, taille de fenêtre, zoom, traductions FR/EN et vérifications fonctionnelles ciblées. Thème sombre complet dans un lot ultérieur si souhaité.

Les changements d’architecture de navigation doivent être isolés des corrections backend et de l’image LibreELEC. Une branche/PR par lot permet de conserver un produit vérifiable à chaque étape.

## 9. Critères d’acceptation

- Aucun formulaire ou sélecteur de dossier ne crée une seconde modale par-dessus une première.
- Aucune action utile ne sort de la fenêtre de référence à 900 × 600 px ; les tableaux larges défilent dans leur propre zone, sans faire déborder les formulaires.
- Même hauteur et mêmes états pour les boutons, champs, onglets et statuts ; aucun bouton d’aide décoratif sans destination.
- Une action principale identifiable par vue ; actions destructives séparées de la consultation.
- Retour depuis une fiche sans perte de filtre, sélection ou position de liste.
- Changements en attente explicites et protection avant de quitter ; les interactions à effet immédiat sont clairement annoncées.
- États « vide », « chargement », « erreur », « accès refusé » et « service désactivé » distingués.
- Focus visible, ordre clavier cohérent, fermeture et retour prévisibles, contraste vérifié et mouvement réduit respecté.
- Aucun contrôle pour une fonctionnalité absente ; aucune métrique inventée pour améliorer l’esthétique.
- La validation matérielle et la sécurité des opérations destructives restent exigées indépendamment du rendu visuel.
