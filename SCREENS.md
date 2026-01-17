# PiNAS - Spécifications des Écrans

> Ce document décrit en détail chaque écran/application de PiNAS.
> Il sert de référence pour le développement frontend et définit les données nécessaires du backend.
> Design inspiré de UGOS (Ugreen NAS OS).

---

## 1. User Manager (Gestion des Utilisateurs)

### 1.1 Vue d'ensemble
Application de gestion complète des utilisateurs et groupes du système NAS.
Design basé sur l'interface UGOS Control Panel > User Management.

### 1.2 Structure générale

```
+------------------------------------------------------------------------+
|  User Management                                    [?] [_] [□] [×]    |
+------------------------------------------------------------------------+
|  [User]  [User Group]  [Advanced Settings]                             |
+------------------------------------------------------------------------+
|                         CONTENU DE L'ONGLET                            |
+------------------------------------------------------------------------+
```

---

### 1.3 Onglet "User" (Liste des utilisateurs)

#### 1.3.1 Layout

```
+------------------------------------------------------------------------+
|  [Add ▼]                                         [⚙] [🔍 Filter    ]   |
+------------------------------------------------------------------------+
|  Username ▼         | Description | Role ▼      | Status ▼  | Edit    |
+------------------------------------------------------------------------+
|  (avatar) john_doe  | -           | Administrator| Normal   | [...]   |
|           [Me]      |             |              |          |         |
+------------------------------------------------------------------------+
|  (avatar) jane      | -           | User         | Normal   | [...]   |
+------------------------------------------------------------------------+
|  (avatar) guest     | Guest user  | Guest        | Disabled | [...]   |
+------------------------------------------------------------------------+
```

#### 1.3.2 Barre d'actions
| Élément | Description |
|---------|-------------|
| **Bouton "Add"** | Bleu (#3B82F6), avec dropdown arrow. Menu: "Add User" |
| **Icône ⚙** | Ouvre le modal "Permission viewer" |
| **Champ Filter** | Input avec icône filtre, recherche dans le tableau |

#### 1.3.3 Colonnes du tableau
| Colonne | Triable | Contenu |
|---------|---------|---------|
| Username | Oui (▼) | Avatar circulaire gris + username + badge bleu "Me" si current user |
| Description | Non | Texte ou "-" si vide |
| Role | Oui (▼) | "Administrator" \| "User" \| "Guest" |
| Status | Oui (▼) | "Normal" (actif) \| "Disabled" (désactivé, texte grisé) |
| Edit | Non | Bouton "..." ouvrant menu dropdown |

#### 1.3.4 Menu contextuel (bouton "...")
| Action | Style | Description |
|--------|-------|-------------|
| Edit | Texte noir | Ouvre modal d'édition utilisateur |
| Delete | Texte rouge (#EF4444) | Supprime après confirmation |

#### 1.3.5 Modal "Permission viewer"
```
+------------------------------------------------------------------+
|  Permission viewer                                         [×]    |
+------------------------------------------------------------------+
|  User [Select ▼]    Shared folder [Select ▼]    [Reset]          |
+------------------------------------------------------------------+
|  Username          | folder1      | folder2      | folder3       |
+------------------------------------------------------------------+
|  (avatar) user1    | Read/Write   | Read/Write   | Read/Write    |
|  (avatar) user2    | Read/Write   | Read/Write   | Read/Write    |
+------------------------------------------------------------------+
|  User data currently shown: 1-12              [<] [1] [>]        |
+------------------------------------------------------------------+
```

---

### 1.4 Onglet "User Group" (Groupes)

#### 1.4.1 Layout
```
+------------------------------------------------------------------------+
|  [Add ▼]                                         [⚙] [🔍 Filter    ]   |
+------------------------------------------------------------------------+
|  Group Name ▼       | Description | Members ▼   | Edit                 |
+------------------------------------------------------------------------+
|  administrators     | System group| 3           | [...]                |
+------------------------------------------------------------------------+
|  users              | Default     | 5           | [...]                |
+------------------------------------------------------------------------+
```

#### 1.4.2 Colonnes du tableau
| Colonne | Triable | Contenu |
|---------|---------|---------|
| Group Name | Oui | Nom du groupe |
| Description | Non | Description ou "-" |
| Members | Oui | Nombre de membres |
| Edit | Non | Menu "..." (Edit, Delete) |

---

### 1.5 Onglet "Advanced Settings"

#### 1.5.1 Section "Password strength rules"
| Option | Type | Défaut | Description |
|--------|------|--------|-------------|
| Do not use user names | Checkbox | Off | Interdit username dans password |
| Do not use common passwords | Checkbox | Off | Interdit passwords courants |
| Must contain uppercase and lowercase | Checkbox | On | Majuscules + minuscules |
| At least 1 number | Checkbox | Off | Au moins un chiffre |
| At least 1 special character | Checkbox | Off | Au moins un caractère spécial |
| Minimum password length | Input number | 6 | Longueur minimale (suffixe "digits") |

#### 1.5.2 Section "Password expiry rules"
| Option | Type | Défaut | Description |
|--------|------|--------|-------------|
| Enable password expiry rules | Toggle | Off | Active/désactive l'expiration |
| Password validity period | Input number | 90 | Durée validité (suffixe "days") |
| Reminder days in advance | Input number | 7 | Rappel avant expiration (suffixe "days") |
| Password change required after expiry | Toggle | Off | Forcer changement |
| Permanent password users | Table + Add/Remove | - | Utilisateurs exemptés |

#### 1.5.3 Layout
```
+------------------------------------------------------------------------+
|  Password strength rules                                               |
+------------------------------------------------------------------------+
|  [ ] Do not use user names                                             |
|  [ ] Do not use common passwords                                       |
|  [✓] Must contain both uppercase and lowercase letters                 |
|  [ ] At least 1 number                                                 |
|  [ ] At least 1 special character                                      |
|  [ ] Minimum password length                        [6      ] digits   |
+------------------------------------------------------------------------+
|  Password expiry rules                                                 |
+------------------------------------------------------------------------+
|  Enable password expiry rules                              [====○]     |
|  Password validity period                          [90     ] days      |
|  Reminder days in advance (before password expires)[7      ] days      |
|  Password change required after expiry                     [○====]     |
|  Permanent password users                    [Add]  [Remove]           |
|  +------------------------------------------------------------------+  |
|  | [ ] Username        | Description | Role ▼    | Status ▼        |  |
|  +------------------------------------------------------------------+  |
+------------------------------------------------------------------------+
|                                                          [Apply]       |
+------------------------------------------------------------------------+
```

---

### 1.6 Modèles de données

#### 1.6.1 User
```typescript
interface User {
  id: string;           // UUID
  username: string;     // Nom d'utilisateur unique
  description: string;  // Description (peut être vide)
  role: 'Administrator' | 'User' | 'Guest';
  status: 'Normal' | 'Disabled';
  avatar?: string;      // URL ou null (afficher avatar par défaut)
  isCurrentUser: boolean; // True si c'est l'utilisateur connecté
  createdAt: string;    // ISO date
  lastLogin?: string;   // ISO date, null si jamais connecté
}
```

#### 1.6.2 UserGroup
```typescript
interface UserGroup {
  id: string;           // UUID
  name: string;         // Nom du groupe
  description: string;  // Description
  memberCount: number;  // Nombre de membres
  isSystem: boolean;    // True = non supprimable
  members?: User[];     // Liste des membres (optionnel, chargé à la demande)
}
```

#### 1.6.3 SharePermission
```typescript
interface SharePermission {
  userId: string;
  username: string;
  avatar?: string;
  permissions: {
    shareId: string;
    shareName: string;
    permission: 'No Access' | 'Read Only' | 'Read/Write';
  }[];
}
```

#### 1.6.4 PasswordSettings
```typescript
interface PasswordSettings {
  strengthRules: {
    noUserNames: boolean;
    noCommonPasswords: boolean;
    requireUpperLower: boolean;
    requireNumber: boolean;
    requireSpecialChar: boolean;
    minLength: number;
  };
  expiryRules: {
    enabled: boolean;
    validityDays: number;
    reminderDays: number;
    forceChangeAfterExpiry: boolean;
    permanentUsers: string[]; // User IDs exemptés
  };
}
```

---

### 1.7 API Endpoints

```
# Users
GET    /api/users                    - Liste tous les utilisateurs
GET    /api/users/:id                - Détails d'un utilisateur
POST   /api/users                    - Créer un utilisateur
PUT    /api/users/:id                - Modifier un utilisateur
DELETE /api/users/:id                - Supprimer un utilisateur
GET    /api/users/me                 - Utilisateur courant

# Groups
GET    /api/groups                   - Liste tous les groupes
GET    /api/groups/:id               - Détails d'un groupe (avec membres)
POST   /api/groups                   - Créer un groupe
PUT    /api/groups/:id               - Modifier un groupe
DELETE /api/groups/:id               - Supprimer un groupe
POST   /api/groups/:id/members       - Ajouter membre(s)
DELETE /api/groups/:id/members/:uid  - Retirer un membre

# Permissions
GET    /api/permissions              - Liste permissions par utilisateur/partage
GET    /api/permissions/user/:id     - Permissions d'un utilisateur
GET    /api/permissions/share/:id    - Permissions sur un partage

# Settings
GET    /api/settings/password        - Paramètres mot de passe
PUT    /api/settings/password        - Modifier paramètres mot de passe
```

---

### 1.8 Actions et modals

#### 1.8.1 Modal "Add User"
```
+------------------------------------------------------------------+
|  Add User                                                  [×]    |
+------------------------------------------------------------------+
|  Username *         [                                    ]        |
|  Description        [                                    ]        |
|  Password *         [                                    ]        |
|  Confirm Password * [                                    ]        |
|  Role *             [User                              ▼]        |
|                                                                   |
|                                      [Cancel]  [Create]           |
+------------------------------------------------------------------+
```

#### 1.8.2 Modal "Edit User"
```
+------------------------------------------------------------------+
|  Edit User                                                 [×]    |
+------------------------------------------------------------------+
|  Username           john_doe (non modifiable)                     |
|  Description        [                                    ]        |
|  Role               [Administrator                     ▼]        |
|  Status             [Normal                            ▼]        |
|                                                                   |
|  [ ] Change password                                              |
|  New Password       [                                    ]        |
|  Confirm Password   [                                    ]        |
|                                                                   |
|                                      [Cancel]  [Save]             |
+------------------------------------------------------------------+
```

#### 1.8.3 Modal "Delete User" (Confirmation)
```
+------------------------------------------------------------------+
|  Delete User                                               [×]    |
+------------------------------------------------------------------+
|  Are you sure you want to delete user "john_doe"?                 |
|  This action cannot be undone.                                    |
|                                                                   |
|                                      [Cancel]  [Delete]           |
+------------------------------------------------------------------+
```

---

### 1.9 États et comportements

#### 1.9.1 États de chargement
- **Spinner** : Affiché au centre pendant le chargement initial
- **Skeleton** : Lignes de tableau grisées pendant refresh
- **Boutons disabled** : Pendant les opérations async

#### 1.9.2 Validation
- Username : 3-32 caractères, alphanumériques + underscore
- Password : Selon règles définies dans Advanced Settings
- Description : Max 255 caractères

#### 1.9.3 Erreurs
- Toast rouge en bas à droite pour erreurs API
- Messages inline sous les champs invalides
- Highlight rouge des champs en erreur

#### 1.9.4 Protections
- L'utilisateur courant ne peut pas se supprimer
- L'admin principal ne peut pas être supprimé
- Les groupes système ne peuvent pas être supprimés

---

### 1.10 Style CSS

#### 1.10.1 Couleurs
```css
--primary: #3B82F6;        /* Bleu - boutons, onglet actif */
--danger: #EF4444;         /* Rouge - delete, erreurs */
--success: #22C55E;        /* Vert - confirmations */
--text-primary: #1F2937;   /* Texte principal */
--text-secondary: #6B7280; /* Texte secondaire */
--border: #E5E7EB;         /* Bordures */
--background: #FFFFFF;     /* Fond */
--background-hover: #F9FAFB; /* Hover ligne tableau */
```

#### 1.10.2 Composants
- **Onglets** : Texte gris, onglet actif bleu avec underline 2px
- **Bouton Add** : Bleu, rounded-lg, padding 8px 16px
- **Tableau** : Header gris clair, lignes alternées blanches, hover gris
- **Avatar** : Cercle 40px, fond gris, icône utilisateur ou initiales
- **Badge "Me"** : Bleu, texte blanc, rounded-full, petit

---

<!--
## 2. Storage Manager (Gestion du Stockage)
À définir...

## 3. Share Manager (Gestion des Partages)
À définir...

## 4. File Manager (Gestionnaire de Fichiers)
À définir...

## 5. Settings (Paramètres Système)
À définir...

## 6. Dashboard (Tableau de Bord)
À définir...

## 7. Network Settings (Paramètres Réseau)
À définir...

## 8. Task Manager (Gestionnaire de Tâches)
À définir...

## 9. Docker Manager (Gestion Docker)
À définir...

## 10. Terminal
À définir...

## 11. Logs Viewer (Visualiseur de Logs)
À définir...

## 12. Security (Sécurité)
À définir...
-->

---

*Dernière mise à jour : Janvier 2025*
