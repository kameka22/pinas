export default {
	// Common
	common: {
		save: 'Enregistrer',
		cancel: 'Annuler',
		delete: 'Supprimer',
		edit: 'Modifier',
		add: 'Ajouter',
		create: 'Créer',
		close: 'Fermer',
		apply: 'Appliquer',
		reset: 'Réinitialiser',
		search: 'Rechercher',
		filter: 'Filtrer',
		actions: 'Actions',
		status: 'Statut',
		enabled: 'Activé',
		disabled: 'Désactivé',
		yes: 'Oui',
		no: 'Non',
		loading: 'Chargement...',
		noData: 'Aucune donnée',
		confirm: 'Confirmer',
		back: 'Retour',
		next: 'Suivant',
		previous: 'Précédent',
		open: 'Ouvrir',
		removeFromDesktop: 'Retirer du bureau',
		addToDesktop: 'Ajouter au bureau',
		noApplicationsFound: 'Aucune application trouvée',
		searchApplications: 'Rechercher des applications...'
	},

	// Control Panel
	controlPanel: {
		title: 'Panneau de configuration',
		backToGrid: 'Retour à la vue grille',
		categories: {
			connectionAccess: 'Connexion et accès',
			general: 'Général',
			service: 'Service'
		},
		items: {
			userManagement: 'Gestion des utilisateurs',
			fileService: 'Service de fichiers',
			deviceConnection: 'Connexion appareil',
			domainLdap: 'Domaine/LDAP',
			terminal: 'Terminal',
			hardwarePower: 'Matériel et alimentation',
			timeLanguage: 'Heure et langue',
			network: 'Réseau',
			security: 'Sécurité',
			indexingService: "Service d'indexation",
			about: 'À propos'
		},
		tabs: {
			general: 'Général',
			storage: 'Stockage',
			service: 'Service',
			deviceAnalysis: 'Analyse appareil'
		},
		about: {
			deviceName: "Nom de l'appareil",
			systemVersion: 'Version système',
			deviceOwner: 'Propriétaire',
			device: 'Appareil',
			model: 'Modèle',
			serialNumber: 'N° série',
			lastStartup: 'Dernier démarrage',
			powerTime: "Temps d'activité",
			hardware: 'Matériel',
			cpu: 'Processeur',
			memory: 'Mémoire',
			network: 'Réseau',
			macAddress: 'Adresse MAC/Masque sous-réseau',
			cores: 'Cœurs',
			threads: 'Threads'
		},
		underDevelopment: 'Cette section est en cours de développement',
		contentFor: "Contenu de l'onglet {tab}"
	},

	// Time & Language
	timeLanguage: {
		title: 'Heure et langue',
		tabs: {
			time: 'Heure',
			language: 'Langue'
		},
		time: {
			title: 'Date et heure',
			timezone: 'Fuseau horaire',
			selectTimezone: 'Sélectionner le fuseau horaire',
			currentTime: 'Heure actuelle',
			dateFormat: 'Format de date',
			timeFormat: "Format d'heure",
			format24h: '24 heures',
			format12h: '12 heures',
			ntpServer: 'Serveur NTP',
			syncWithNtp: 'Synchroniser avec le serveur NTP',
			syncNow: 'Synchroniser maintenant',
			lastSync: 'Dernière synchronisation'
		},
		language: {
			title: "Langue d'affichage",
			selectLanguage: 'Sélectionner la langue',
			currentLanguage: 'Langue actuelle',
			restart: 'Certains changements peuvent nécessiter un rafraîchissement pour prendre effet'
		}
	},

	// User Manager
	userManager: {
		title: 'Gestion des utilisateurs',
		tabs: {
			user: 'Utilisateur',
			userGroup: "Groupe d'utilisateurs",
			advancedSettings: 'Paramètres avancés'
		},
		table: {
			username: "Nom d'utilisateur",
			description: 'Description',
			role: 'Rôle',
			status: 'Statut',
			edit: 'Modifier',
			groupName: 'Nom du groupe',
			members: 'Membres'
		},
		roles: {
			administrator: 'Administrateur',
			user: 'Utilisateur',
			guest: 'Invité'
		},
		statuses: {
			normal: 'Normal',
			disabled: 'Désactivé'
		},
		actions: {
			addUser: 'Ajouter utilisateur',
			addGroup: 'Ajouter groupe',
			editUser: "Modifier l'utilisateur",
			editGroup: 'Modifier le groupe',
			deleteUser: "Supprimer l'utilisateur",
			deleteGroup: 'Supprimer le groupe'
		},
		modals: {
			addUserTitle: 'Ajouter un utilisateur',
			editUserTitle: "Modifier l'utilisateur",
			deleteUserTitle: "Supprimer l'utilisateur",
			addGroupTitle: 'Ajouter un groupe',
			editGroupTitle: 'Modifier le groupe',
			deleteGroupTitle: 'Supprimer le groupe',
			permissionViewer: 'Visualiseur de permissions'
		},
		fields: {
			username: "Nom d'utilisateur",
			password: 'Mot de passe',
			confirmPassword: 'Confirmer le mot de passe',
			description: 'Description',
			role: 'Rôle',
			status: 'Statut',
			changePassword: 'Changer le mot de passe',
			newPassword: 'Nouveau mot de passe',
			groupName: 'Nom du groupe',
			selectMembers: 'Sélectionner les membres'
		},
		permissions: {
			title: 'Permissions',
			user: 'Utilisateur',
			sharedFolder: 'Dossier partagé',
			noAccess: 'Pas d\'accès',
			readOnly: 'Lecture seule',
			readWrite: 'Lecture/Écriture'
		},
		messages: {
			deleteConfirm: 'Êtes-vous sûr de vouloir supprimer l\'utilisateur "{username}" ?',
			deleteGroupConfirm: 'Êtes-vous sûr de vouloir supprimer le groupe "{groupName}" ?',
			cannotBeUndone: 'Cette action est irréversible.',
			userDataShown: 'Données utilisateur affichées',
			me: 'Moi'
		},
		advancedSettings: {
			passwordStrength: {
				title: 'Règles de complexité du mot de passe',
				noUserNames: "Ne pas utiliser le nom d'utilisateur",
				noCommonPasswords: 'Ne pas utiliser de mots de passe courants',
				requireUpperLower: 'Doit contenir des majuscules et des minuscules',
				requireNumber: 'Au moins 1 chiffre',
				requireSpecialChar: 'Au moins 1 caractère spécial',
				minLength: 'Longueur minimale du mot de passe',
				digits: 'caractères'
			},
			passwordExpiry: {
				title: "Règles d'expiration du mot de passe",
				enabled: "Activer les règles d'expiration",
				validityPeriod: 'Période de validité du mot de passe',
				reminderDays: "Rappel (jours avant l'expiration)",
				forceChange: "Changement obligatoire après expiration",
				permanentUsers: 'Utilisateurs avec mot de passe permanent',
				days: 'jours'
			}
		}
	},

	// Desktop & App Launcher
	desktop: {
		appLauncher: {
			title: 'Applications',
			categories: {
				system: 'Système',
				storageFiles: 'Stockage et fichiers',
				services: 'Services'
			}
		}
	},

	// Top Bar
	topBar: {
		search: 'Rechercher...',
		notifications: 'Notifications',
		noNotifications: 'Aucune notification',
		clearAll: 'Tout effacer',
		userMenu: {
			profile: 'Profil',
			settings: 'Paramètres',
			logout: 'Déconnexion'
		}
	},

	// System Widgets
	widgets: {
		cpu: 'CPU',
		memory: 'Mémoire',
		network: 'Réseau',
		storage: 'Stockage',
		upload: 'Envoi',
		download: 'Téléchargement'
	},

	// Window Controls
	window: {
		minimize: 'Réduire',
		maximize: 'Agrandir',
		restore: 'Restaurer',
		close: 'Fermer',
		help: 'Aide'
	},

	// Notifications
	notifications: {
		title: 'Notifications',
		markAsRead: 'Marquer comme lu',
		clearAll: 'Tout effacer',
		empty: 'Aucune notification'
	},

	// File Manager
	fileManager: {
		title: 'Fichiers',
		sidebar: {
			personalFolder: 'Dossier personnel',
			sharedFolder: 'Dossier partagé',
			userFolder: 'Dossier utilisateur'
		},
		toolbar: {
			refresh: 'Actualiser',
			search: 'Rechercher...',
			newFolder: 'Nouveau dossier',
			upload: 'Téléverser',
			copy: 'Copier',
			paste: 'Coller',
			cut: 'Couper',
			duplicate: 'Dupliquer',
			archive: 'Archiver',
			settings: 'Paramètres',
			sort: 'Trier'
		},
		columns: {
			name: 'Nom',
			size: 'Taille',
			type: 'Type',
			modified: 'Date de modification'
		},
		types: {
			folder: 'Dossier',
			file: 'Fichier',
			trash: 'Corbeille'
		},
		viewModes: {
			list: 'Liste',
			grid: 'Grille',
			compact: 'Compact'
		},
		contextMenu: {
			download: 'Télécharger',
			rename: 'Renommer',
			properties: 'Propriétés'
		},
		statusBar: {
			item: 'élément au total',
			items: 'éléments au total',
			selected: 'sélectionné(s)'
		}
	}
};
