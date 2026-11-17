# Maquettes PiNAS — direction 01

Ouvrir `index.html` dans un navigateur ; page autonome, sans installation ni connexion réseau. Affichage également fourni dans la conversation OpenMausBot.

Trois applications : Partages, Utilisateurs, Disques. Recherche, navigation liste/fiche, onglets et assistant de partage simulés. Les données et résultats sont fictifs ; aucune requête à PiNAS. Le sélecteur de dossier et les options SMB/NFS restent schématiques. Les valeurs ne persistent pas après rechargement.

Validation effectuée : syntaxe JavaScript, navigation des trois listes/fiches/onglets et quatre étapes de l’assistant dans un environnement DOM simulé ; diff sans erreur d’espacement. Vérification dans le navigateur intégré : rendu clair de la fiche partage, navigation réelle des trois applications et de leurs fiches/onglets, assistant jusqu’au résumé ; aucun débordement horizontal à la largeur du navigateur. Les vérifications visuelles sombre et mobile restent à compléter. Aucun build Rust/Svelte relancé : changement limité aux documents et à ce prototype isolé.

Plan actif : `../../../REMEDIATION_PLAN.md`. Cette proposition attend la validation de Laurent avant implémentation dans Svelte.
