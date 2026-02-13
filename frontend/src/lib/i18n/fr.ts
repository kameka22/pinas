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
		addToDock: 'Ajouter au dock',
		removeFromDock: 'Retirer du dock',
		pinToDock: 'Épingler au dock',
		noApplicationsFound: 'Aucune application trouvée',
		searchApplications: 'Rechercher des applications...',
		refresh: 'Actualiser',
		retry: 'Réessayer',
		copy: 'Copier'
	},

	// Password validation rules
	passwordRules: {
		minLength: 'Au moins 8 caractères',
		passwordsMatch: 'Les mots de passe correspondent'
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
			printer: 'Imprimante',
			hardwarePower: 'Matériel et alimentation',
			timeLanguage: 'Heure et langue',
			network: 'Réseau',
			security: 'Sécurité',
			indexingService: "Service d'indexation",
			systemUpdate: 'Mise à jour système',
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
			email: 'Email',
			description: 'Description',
			role: 'Rôle',
			status: 'Statut',
			services: 'Services',
			edit: 'Modifier',
			groupName: 'Nom du groupe',
			members: 'Membres'
		},
		roles: {
			administrator: 'Administrateur',
			user: 'Utilisateur',
			guest: 'Invité'
		},
		badges: {
			system: 'Système'
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
			email: 'Email',
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
		serviceAccess: {
			title: 'Accès services',
			smb: 'SMB (Windows)',
			nfs: 'NFS (Linux)',
			ftp: 'FTP',
			noAccess: 'Aucun accès'
		},
		permissions: {
			title: 'Permissions',
			user: 'Utilisateur',
			users: 'Utilisateurs',
			groups: 'Groupes',
			sharedFolder: 'Dossier partagé',
			noAccess: 'Pas d\'accès',
			readOnly: 'Lecture seule',
			readWrite: 'Lecture/Écriture',
			addFolder: 'Ajouter un dossier',
			folderPath: 'Chemin du dossier',
			folderPathHint: 'Entrez le chemin complet du dossier (ex: /storage/shares/documents)',
			noPermissions: 'Aucune permission de dossier configurée',
			noPermissionsHint: 'Ajoutez un dossier pour commencer à gérer les permissions',
			confirmRemoveFolder: 'Supprimer toutes les permissions pour {path} ?'
		},
		messages: {
			deleteConfirm: 'Êtes-vous sûr de vouloir supprimer l\'utilisateur "{username}" ?',
			deleteGroupConfirm: 'Êtes-vous sûr de vouloir supprimer le groupe "{groupName}" ?',
			cannotBeUndone: 'Cette action est irréversible.',
			userDataShown: 'Données utilisateur affichées',
			me: 'Moi',
			noUsersFound: 'Aucun utilisateur trouvé',
			noGroupsFound: 'Aucun groupe trouvé',
			cannotChangeOwnRole: 'Vous ne pouvez pas modifier votre propre rôle',
			systemGroupNameReadonly: 'Les noms des groupes système ne peuvent pas être modifiés'
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
				all: 'Tout',
				system: 'Système',
				storageFiles: 'Stockage et fichiers',
				services: 'Services'
			}
		}
	},

	// Application names
	apps: {
		controlPanel: 'Panneau de configuration',
		files: 'Fichiers',
		appCenter: "Centre d'applications",
		storage: 'Stockage',
		shares: 'Partages',
		docker: 'Docker',
		terminal: 'Terminal',
		users: 'Utilisateurs',
		processManager: 'Gestionnaire de processus',
		kodi: 'Kodi',
		display: 'Affichage Externe'
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
			logout: 'Déconnexion',
			restart: 'Redémarrer',
			shutdown: 'Éteindre',
			confirmRestart: 'Êtes-vous sûr de vouloir redémarrer le système ?',
			confirmShutdown: 'Êtes-vous sûr de vouloir éteindre le système ?',
			restartingMessage: 'Le système redémarre...',
			shuttingDownMessage: 'Le système s\'éteint...',
			restartingSubtitle: 'Veuillez patienter, cela peut prendre un moment',
			shuttingDownSubtitle: 'Vous pouvez débrancher l\'appareil en toute sécurité',
			systemRestarted: 'Système redémarré avec succès',
			systemShutDown: 'Le système s\'éteint',
			reloadDesktop: 'Recharger le bureau',
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

	// Onboarding
	onboarding: {
		welcome: 'Bienvenue !',
		subtitle: 'Configurons votre NAS.',
		steps: {
			welcome: 'Bienvenue',
			language: 'Langue',
			device: 'Appareil',
			account: 'Compte',
			password: 'Mot de passe',
			ssh: 'SSH',
			features: 'Découvrir'
		},
		welcomeScreen: {
			title: 'Bienvenue sur PiNAS !',
			thankYou: 'Merci d\'avoir installé PiNAS sur votre Raspberry Pi.',
			description: 'En quelques étapes, nous allons configurer votre NAS personnel et vous serez prêt à stocker, partager et gérer vos fichiers.',
			letsGo: 'C\'est parti !'
		},
		language: {
			title: 'Choisissez votre langue',
			description: "Sélectionnez la langue de l'interface.",
			selectLanguage: 'Sélectionner la langue'
		},
		device: {
			title: 'Nommez votre appareil',
			description: 'Ce nom identifiera votre NAS sur le réseau.',
			fieldLabel: "Nom de l'appareil",
			placeholder: 'ex: pinas-maison'
		},
		account: {
			title: 'Créer le compte administrateur',
			description: 'Ce sera le compte administrateur de votre NAS.',
			fieldLabel: "Nom d'utilisateur",
			placeholder: 'ex: admin'
		},
		password: {
			title: 'Définir votre mot de passe',
			description: 'Choisissez un mot de passe sécurisé pour votre compte admin.',
			fieldLabel: 'Mot de passe',
			confirmLabel: 'Confirmer le mot de passe',
			placeholder: 'Entrez le mot de passe',
			confirmPlaceholder: 'Confirmez le mot de passe'
		},
		ssh: {
			title: 'Accès à distance (SSH)',
			description: 'SSH vous permet de vous connecter à votre NAS à distance via un terminal.',
			enableSsh: 'Activer SSH',
			enableHint: 'Autoriser l\'accès terminal à distance à votre appareil',
			port: 'Port',
			passwordLabel: 'Mot de passe SSH',
			passwordPlaceholder: 'Entrez le mot de passe SSH',
			confirmPasswordLabel: 'Confirmer le mot de passe',
			confirmPasswordPlaceholder: 'Confirmez le mot de passe SSH',
			passwordHint: 'Ce mot de passe sera utilisé pour se connecter en tant que root via SSH.',
			skipHint: 'Vous pourrez toujours activer SSH plus tard dans les Paramètres.'
		},
		features: {
			title: 'Votre NAS est prêt !',
			subtitle: 'Voici ce que vous pouvez faire avec PiNAS :',
			items: {
				files: {
					title: 'Gestionnaire de fichiers',
					description: 'Parcourez, téléversez et organisez tous vos fichiers.'
				},
				storage: {
					title: 'Stockage',
					description: 'Gérez vos disques, créez des pools RAID et des volumes.'
				},
				shares: {
					title: 'Partage de fichiers',
					description: 'Partagez des dossiers sur votre réseau via SMB.'
				},
				docker: {
					title: 'Docker & Apps',
					description: 'Installez des apps comme Plex, Pi-hole et bien plus.'
				},
				kodi: {
					title: 'Media Center',
					description: 'Kodi est intégré pour vos films et votre musique.'
				},
				terminal: {
					title: 'Terminal',
					description: 'Accès shell complet directement depuis votre navigateur.'
				}
			}
		},
		buttons: {
			back: 'Retour',
			next: 'Suivant',
			complete: 'Commencer à utiliser PiNAS',
			getStarted: 'C\'est parti !'
		},
		validation: {
			machineNameRequired: "Le nom de l'appareil est requis",
			machineNameMinLength: "Le nom doit contenir au moins 2 caractères",
			machineNameInvalid: 'Seuls les lettres, chiffres et tirets sont autorisés',
			usernameRequired: "Le nom d'utilisateur est requis",
			usernameMinLength: "Le nom d'utilisateur doit contenir au moins 3 caractères",
			usernameInvalid: 'Seuls les lettres, chiffres et underscores sont autorisés',
			passwordRequired: 'Le mot de passe est requis',
			passwordMinLength: 'Le mot de passe doit contenir au moins 8 caractères',
			passwordMismatch: 'Les mots de passe ne correspondent pas',
			sshPasswordMinLength: 'Le mot de passe SSH doit contenir au moins 8 caractères',
			sshPasswordMismatch: 'Les mots de passe SSH ne correspondent pas'
		}
	},

	// File Manager
	fileManager: {
		title: 'Fichiers',
		sidebar: {
			personalFolder: 'Dossier personnel',
			sharedFolder: 'Dossier partagé',
			userFolder: 'Dossier utilisateur'
		},
		sections: {
			personal: 'Personnel',
			shares: 'Dossiers partagés',
			volumes: 'Volumes',
			media: 'Périphériques amovibles'
		},
		statuses: {
			mounted: 'Monté',
			unmounted: 'Non monté',
			disabled: 'Désactivé'
		},
		noLocations: 'Aucun emplacement disponible',
		toolbar: {
			refresh: 'Actualiser',
			search: 'Rechercher...',
			newFolder: 'Nouveau dossier',
			newFile: 'Nouveau fichier',
			upload: 'Téléverser',
			download: 'Télécharger',
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
			open: 'Ouvrir',
			download: 'Télécharger',
			rename: 'Renommer',
			copy: 'Copier',
			cut: 'Couper',
			delete: 'Supprimer',
			newFolder: 'Nouveau dossier',
			newFile: 'Nouveau fichier',
			upload: 'Téléverser',
			paste: 'Coller',
			selectAll: 'Tout sélectionner',
			properties: 'Propriétés'
		},
		modals: {
			newFolderTitle: 'Nouveau dossier',
			newFileTitle: 'Nouveau fichier',
			folderNamePlaceholder: 'Nom du dossier',
			fileNamePlaceholder: 'Nom du fichier',
			deleteTitle: 'Supprimer',
			deleteFolderMessage: 'Supprimer ce dossier et tout son contenu ?',
			deleteFileMessage: 'Supprimer ce fichier ?',
			emptyFolder: 'Ce dossier est vide'
		},
		statusBar: {
			item: 'élément au total',
			items: 'éléments au total',
			selected: 'sélectionné(s)'
		}
	},

	// Task Manager
	taskManager: {
		title: 'Tâches',
		clear: 'Effacer les terminées',
		empty: 'Aucune tâche',
		types: {
			upload: 'Téléversement',
			copy: 'Copie',
			move: 'Déplacement',
			delete: 'Suppression',
			createFolder: 'Création dossier',
			createFile: 'Création fichier'
		}
	},

	// App Center
	appCenter: {
		title: 'Centre d\'applications',
		searchPlaceholder: 'Rechercher des applications...',
		installedCount: 'installée(s)',
		noPackages: 'Aucune application trouvée',
		version: 'Version',
		description: 'Description',
		features: 'Fonctionnalités',
		categories: {
			all: 'Toutes',
			containers: 'Conteneurs',
			media: 'Multimédia',
			network: 'Réseau',
			utilities: 'Utilitaires'
		},
		status: {
			installed: 'Installé',
			installing: 'Installation...',
			updateAvailable: 'Mise à jour disponible',
			notInstalled: 'Non installé'
		},
		actions: {
			install: 'Installer',
			installing: 'Installation...',
			uninstall: 'Désinstaller',
			uninstalling: 'Désinstallation en cours...',
			open: 'Ouvrir',
			update: 'Mettre à jour'
		},
		uninstallModal: {
			title: 'Désinstaller l\'application',
			message: 'Voulez-vous désinstaller',
			deleteData: 'Supprimer les données de l\'application'
		},
		missingDependencies: 'Dépendances manquantes',
		requiresInstall: 'Nécessite',
		dependencies: 'Dépendances',
		packages: {
			docker: {
				description: 'Plateforme de conteneurs pour déployer et gérer des applications',
				feature1: 'Exécuter des conteneurs isolés',
				feature2: 'Déploiement facile d\'applications',
				feature3: 'Support Docker Compose'
			}
		}
	},

	// Generic App Components
	iframeApp: {
		connectionError: 'Impossible de se connecter à l\'application',
		timeout: 'Délai de connexion dépassé - l\'application n\'est peut-être pas en cours d\'exécution',
		openExternal: 'Ouvrir dans un nouvel onglet',
		errorTitle: 'Erreur de connexion'
	},

	webviewApp: {
		description: 'Cette application s\'ouvre dans un nouvel onglet du navigateur',
		openApp: 'Ouvrir l\'application',
		checking: 'Vérification du statut...',
		online: 'En ligne',
		offline: 'Hors ligne',
		tip: 'Vous pouvez également accéder à cette application directement via l\'URL ci-dessus'
	},

	serviceApp: {
		running: 'En cours',
		stopped: 'Arrêté',
		status: 'Statut',
		logs: 'Journaux',
		config: 'Configuration',
		actions: 'Actions',
		start: 'Démarrer',
		stop: 'Arrêter',
		restart: 'Redémarrer',
		uptime: 'Temps de fonctionnement',
		memory: 'Mémoire',
		cpu: 'CPU',
		autostart: 'Démarrage auto',
		recentLogs: 'Journaux récents',
		noLogs: 'Aucun journal disponible',
		configPlaceholder: 'Options de configuration bientôt disponibles'
	},

	// Storage Manager
	storageManager: {
		title: 'Gestionnaire de stockage',
		sidebar: {
			overview: 'Aperçu',
			storage: 'Stockage',
			hardDisk: 'Disque dur',
			externalStorage: 'Stockage externe'
		},
		tabs: {
			poolsVolumes: 'Pool de stockage & volume',
			dataOrganizing: 'Organisation des données',
			advancedSettings: 'Paramètres avancés'
		},
		overview: {
			title: 'Aperçu du stockage',
			disks: 'Disques',
			pools: 'Pools',
			volumes: 'Volumes',
			totalCapacity: 'Capacité totale'
		},
		pools: {
			title: 'Pools de stockage',
			create: 'Créer',
			createPool: 'Pool de stockage',
			createVolume: 'Volume',
			noPoolsConfigured: 'Aucun pool de stockage configuré',
			noPools: 'Aucun pool',
			pool: 'pool',
			pools: 'pools',
			disk: 'disque',
			disks: 'disques',
			noVolumes: 'Aucun volume dans ce pool',
			createVolumeLink: 'Créer un volume'
		},
		volumes: {
			title: 'Volumes',
			used: 'Utilisé',
			mount: 'Monter',
			unmount: 'Démonter'
		},
		disks: {
			title: 'Disques de stockage',
			systemDisk: 'Disque système',
			system: 'Système',
			smart: 'S.M.A.R.T.',
			details: 'Détails',
			wipe: 'Effacer',
			noExternalDevices: 'Aucun périphérique de stockage externe connecté'
		},
		status: {
			normal: 'Normal',
			degraded: 'Dégradé',
			rebuilding: 'Reconstruction',
			error: 'Erreur',
			creating: 'Création',
			mounted: 'Monté',
			unmounted: 'Démonté'
		},
		raidTypes: {
			basic: 'Basique',
			jbod: 'JBOD',
			raid0: 'RAID 0',
			raid1: 'RAID 1',
			raid5: 'RAID 5',
			raid10: 'RAID 10',
			btrfsSingle: 'Btrfs Simple',
			btrfsRaid1: 'Btrfs RAID1',
			basicDesc: 'Disque unique, pas de redondance. Données perdues si le disque tombe en panne.',
			jbodDesc: 'Ensemble de disques. Pas de redondance, combine la capacité.',
			raid0Desc: 'Striping. Performance maximale, pas de redondance.',
			raid1Desc: 'Miroir. 50% de capacité, survit à 1 panne de disque.',
			raid5Desc: 'Striping avec parité. Survit à 1 panne de disque.',
			raid10Desc: 'Striping + Miroir. Haute performance, survit aux pannes.',
			btrfsSingleDesc: 'Système de fichiers Btrfs, pas de redondance.',
			btrfsRaid1Desc: 'Miroir Btrfs. Survit à 1 panne de disque.'
		},
		modals: {
			createPool: {
				title: 'Créer un pool de stockage',
				preview: 'Aperçu',
				poolName: 'Nom du pool',
				selectHardDisk: 'Sélectionner le disque dur',
				noDisksAvailable: 'Aucun disque disponible trouvé',
				selectRaidType: 'Sélectionner le type RAID',
				available: 'Disponible',
				wipeDisks: 'Effacer les disques sélectionnés avant de créer le pool',
				total: 'Total',
				raidType: 'Type RAID',
				useHardDisk: 'Utiliser le disque dur',
				recommended: 'Rec.'
			},
			createVolume: {
				title: 'Créer un volume',
				storagePool: 'Pool de stockage',
				selectPool: 'Sélectionner un pool...',
				volumeName: 'Nom du volume',
				fileSystem: 'Système de fichiers'
			},
			deletePool: {
				title: 'Supprimer le pool',
				confirmMessage: 'Êtes-vous sûr de vouloir supprimer',
				willDeleteVolumes: 'Cela supprimera également',
				volume: 'volume',
				volumes: 'volumes',
				cannotBeUndone: 'Cette action est irréversible.'
			},
			deleteVolume: {
				title: 'Supprimer le volume',
				confirmMessage: 'Êtes-vous sûr de vouloir supprimer',
				dataLost: 'Toutes les données sur ce volume seront perdues.',
				cannotBeUndone: 'Cette action est irréversible.'
			},
			smartInfo: {
				title: 'Informations S.M.A.R.T.',
				temperature: 'Température',
				powerOnHours: 'Heures de fonctionnement',
				powerCycles: 'Cycles de démarrage',
				serial: 'N° de série',
				attributes: 'Attributs',
				loadingData: 'Chargement des données S.M.A.R.T....',
				loadFailed: 'Échec du chargement des informations S.M.A.R.T.'
			},
			editPool: {
				title: 'Modifier le pool',
				poolName: 'Nom du pool',
				description: 'Description'
			},
			wipeDisk: {
				title: 'Effacer le disque',
				warning: 'Attention : Cette opération est destructive !',
				confirmMessage: 'Êtes-vous sûr de vouloir effacer',
				allDataLost: 'Toutes les données sur ce disque seront définitivement supprimées.',
				cannotBeUndone: 'Cette action est irréversible.',
				wiping: 'Effacement...'
			},
			diskDetails: {
				title: 'Détails du disque',
				totalSize: 'Taille totale',
				type: 'Type',
				serial: 'Numéro de série',
				deviceId: 'ID du périphérique',
				partitions: 'Partitions',
				noPartitions: 'Aucune partition trouvée sur ce disque'
			}
		},
		health: {
			rebuilding: 'Reconstruction',
			lastScrub: 'Dernière vérification',
			errorsFound: 'erreurs',
			deviceErrors: 'Erreurs de périphérique détectées',
			today: "Aujourd'hui",
			yesterday: 'Hier',
			daysAgo: 'jours',
			noScrub: 'Jamais vérifié'
		},
		contextMenu: {
			createVolume: 'Créer un volume',
			edit: 'Modifier',
			scrub: 'Vérifier',
			scrubbing: 'Vérification...',
			changeRaidType: 'Changer le type RAID',
			delete: 'Supprimer'
		},
		messages: {
			loading: 'Chargement des informations de stockage...',
			comingSoon: 'bientôt disponible',
			dataOrganizingComingSoon: 'Fonctionnalités d\'organisation des données bientôt disponibles',
			advancedSettingsComingSoon: 'Paramètres avancés bientôt disponibles',
			empty: 'Vide'
		},
		errors: {
			loadFailed: 'Échec du chargement des données de stockage',
			createPoolFailed: 'Échec de la création du pool',
			createVolumeFailed: 'Échec de la création du volume',
			deletePoolFailed: 'Échec de la suppression du pool',
			deleteVolumeFailed: 'Échec de la suppression du volume',
			toggleMountFailed: 'Échec du montage/démontage du volume',
			scrubFailed: 'Échec du lancement de la vérification',
			editPoolFailed: 'Échec de la modification du pool',
			wipeDiskFailed: 'Échec de l\'effacement du disque'
		}
	},

	// Process Manager
	processManager: {
		title: 'Gestionnaire de processus',
		processes: 'Processus',
		running: 'En cours',
		endProcess: 'Terminer le processus',
		confirmKill: 'Êtes-vous sûr de vouloir terminer ce processus ?',
		noProcesses: 'Aucun processus trouvé',
		columns: {
			name: 'Nom',
			user: 'Utilisateur',
			memory: 'Mémoire',
			status: 'Statut'
		},
		status: {
			running: 'En cours',
			sleeping: 'En veille',
			stopped: 'Arrêté',
			zombie: 'Zombie',
			idle: 'Inactif',
			unknown: 'Inconnu'
		},
		errors: {
			loadFailed: 'Échec du chargement des processus',
			killFailed: 'Échec de la terminaison du processus'
		}
	},

	// Terminal App
	terminalApp: {
		welcome: 'PiNAS Terminal v1.0',
		helpHint: 'Tapez "help" pour les commandes disponibles.',
		tabHint: 'Utilisez Tab pour l\'auto-complétion des chemins.',
		errors: {
			authRequired: 'Authentification requise. Veuillez vous reconnecter.',
			adminRequired: "Accès administrateur requis pour utiliser le terminal.",
			rateLimit: 'Trop de commandes. Veuillez patienter avant de continuer.',
			connectionFailed: 'Erreur de connexion'
		}
	},

	// Display App
	display: {
		title: 'Affichage Externe',
		description: "Gérer les services d'affichage HDMI",
		services: 'Services disponibles',
		noActiveService: 'Aucun service actif',
		splashActive: "L'écran d'accueil PiNAS est affiché sur la sortie HDMI",
		splashDescription: 'Accédez à PiNAS sur :',
		activeService: 'Service actif',
		configure: 'Configurer',
		back: 'Retour',
		activating: 'Démarrage...',
		deactivating: 'Arrêt...',
		hdmiOutput: 'Sortie HDMI'
	},

	// Kodi App
	kodi: {
		title: 'Kodi',
		tabs: {
			remote: 'Télécommande',
			sources: 'Sources',
			settings: 'Paramètres',
			addons: 'Extensions',
			library: 'Médiathèque'
		},
		status: {
			connected: 'Connecté',
			disconnected: 'Déconnecté',
			playing: 'Lecture',
			paused: 'Pause',
			stopped: 'Arrêté'
		},
		remote: {
			nowPlaying: 'En cours de lecture',
			nothingPlaying: 'Rien en lecture',
			volume: 'Volume',
			mute: 'Muet',
			playback: 'Lecture',
			previous: 'Précédent',
			next: 'Suivant',
			stop: 'Stop',
			navigation: 'Navigation',
			back: 'Retour',
			home: 'Accueil',
			menu: 'Menu',
			info: 'Info'
		},
		sources: {
			title: 'Sources multimédia',
			addSource: 'Ajouter une source',
			editSource: 'Modifier la source',
			deleteSource: 'Supprimer la source',
			noSources: 'Aucune source multimédia configurée',
			type: 'Type',
			path: 'Chemin',
			name: 'Nom',
			types: {
				smb: 'SMB/CIFS',
				nfs: 'NFS',
				local: 'Local'
			},
			mediaTypes: {
				video: 'Vidéo',
				music: 'Musique',
				pictures: 'Images',
				files: 'Fichiers'
			},
			allTypes: 'Tous les types',
			pathFormats: 'Formats: smb://server/share, nfs://server/path, /storage/path',
			fields: {
				sourceName: 'Nom de la source',
				sourceType: 'Type de source',
				mediaType: 'Type de média',
				serverPath: 'Serveur/Chemin',
				username: "Nom d'utilisateur",
				password: 'Mot de passe'
			},
			deleteConfirm: 'Êtes-vous sûr de vouloir supprimer cette source ?'
		},
		settings: {
			title: 'Paramètres Kodi',
			categories: {
				player: 'Lecteur',
				media: 'Médias',
				interface: 'Interface',
				services: 'Services',
				system: 'Système'
			},
			player: {
				skipSteps: 'Pas de saut (secondes)',
				defaultPlayer: 'Lecteur par défaut'
			},
			media: {
				showHiddenFiles: 'Afficher les fichiers cachés',
				autoScanLibrary: 'Analyse auto de la médiathèque'
			},
			interface: {
				language: 'Langue',
				skin: 'Thème',
				soundsEnabled: 'Sons de l\'interface'
			},
			services: {
				webServer: 'Serveur web',
				airplay: 'AirPlay',
				upnp: 'UPnP/DLNA'
			},
			system: {
				powerSaving: 'Économie d\'énergie',
				debugLogging: 'Journalisation debug'
			}
		},
		addons: {
			title: 'Extensions',
			installed: 'Installées',
			available: 'Disponibles',
			enable: 'Activer',
			disable: 'Désactiver',
			noAddons: 'Aucune extension trouvée'
		},
		library: {
			title: 'Médiathèque',
			videoLibrary: 'Vidéothèque',
			musicLibrary: 'Musicothèque',
			scan: 'Analyser',
			clean: 'Nettoyer',
			scanVideo: 'Analyser la vidéothèque',
			scanMusic: 'Analyser la musicothèque',
			cleanVideo: 'Nettoyer la vidéothèque',
			cleanMusic: 'Nettoyer la musicothèque',
			scanning: 'Analyse en cours...',
			cleaning: 'Nettoyage en cours...',
			scanStarted: 'Analyse de la médiathèque lancée',
			lastScanned: 'Dernière analyse',
			movies: 'Films',
			tvShows: 'Séries TV',
			albums: 'Albums',
			artists: 'Artistes'
		},
		actions: {
			reboot: 'Redémarrer Kodi',
			shutdown: 'Éteindre Kodi',
			sendNotification: 'Envoyer une notification'
		},
		errors: {
			connectionFailed: 'Échec de la connexion à Kodi',
			actionFailed: 'Action échouée',
			loadFailed: 'Échec du chargement des données Kodi',
			addSourceFailed: 'Échec de l\'ajout de la source',
			removeSourceFailed: 'Échec de la suppression de la source',
			updateSettingFailed: 'Échec de la mise à jour du paramètre',
			toggleAddonFailed: 'Échec de l\'activation/désactivation de l\'addon',
			scanFailed: 'Échec de l\'analyse de la médiathèque'
		}
	},

	// Network Settings
	networkSettings: {
		title: 'Réseau',
		description: 'Configurer les interfaces réseau et le DNS',
		tabs: {
			general: 'Général',
			interfaces: 'Interface réseau'
		},
		hostname: "Nom d'hôte",
		hostnameHint: 'Nom de votre appareil sur le réseau',
		defaultGateway: 'Passerelle par défaut',
		dnsServer: 'Serveur DNS',
		configureDnsManually: 'Configurer le serveur DNS manuellement',
		primaryDns: 'DNS primaire',
		secondaryDns: 'DNS secondaire',
		dnsAuto: 'DNS fourni par DHCP',
		ipAddress: 'Adresse IP',
		subnetMask: 'Masque de sous-réseau',
		gateway: 'Passerelle',
		macAddress: 'Adresse MAC',
		speed: 'Vitesse',
		method: 'Méthode',
		manual: 'Manuel',
		connected: 'Connecté',
		disconnected: 'Déconnecté',
		editInterface: "Modifier l'interface",
		noInterfaces: 'Aucune interface réseau détectée'
	},

	// File Service
	fileService: {
		title: 'Services de fichiers',
		ssh: {
			description: 'Accès Shell sécurisé pour l\'administration à distance',
			enable: 'Activer SSH',
			enableHint: 'Autoriser l\'accès terminal à distance',
			status: 'État',
			running: 'En cours',
			stopped: 'Arrêté',
			port: 'Port',
			password: 'Mot de passe',
			passwordHint: 'Utilisé pour la connexion SSH en tant que root',
			changePassword: 'Changer le mot de passe',
			newPassword: 'Nouveau mot de passe',
			confirmPassword: 'Confirmer le mot de passe',
			passwordDescription: 'Ce mot de passe est utilisé pour se connecter via SSH en tant qu\'utilisateur root.',
			passwordTooShort: 'Le mot de passe doit contenir au moins 8 caractères',
			passwordMismatch: 'Les mots de passe ne correspondent pas',
			connectionInfo: 'Se connecter avec :'
		},
		smb: {
			title: 'SMB/CIFS',
			description: 'Protocole de partage de fichiers Windows',
			enable: 'Activer SMB',
			disable: 'Désactiver SMB',
			status: 'Statut',
			running: 'En cours d\'exécution',
			stopped: 'Arrêté',
			workgroup: 'Groupe de travail',
			serverDescription: 'Description du serveur',
			minProtocol: 'Protocole minimum',
			maxProtocol: 'Protocole maximum',
			apply: 'Appliquer',
			applySuccess: 'Configuration SMB mise à jour',
			applyError: 'Erreur lors de la mise à jour de la configuration SMB',
			activeShares: 'Partages actifs',
			manageShares: 'Gérer les partages',
			connectedUsers: 'Utilisateurs connectés',
			version: 'Version',
			globalConfig: 'Configuration globale'
		},
		nfs: {
			description: 'Système de fichiers réseau pour Unix/Linux'
		},
		ftp: {
			description: 'Protocole de transfert de fichiers'
		},
		cups: {
			title: 'Partage d\'imprimante',
			description: 'Partagez vos imprimantes USB sur le réseau',
			enable: 'Activer le partage d\'imprimante',
			enableHint: 'Démarrer le service CUPS pour partager les imprimantes via IPP/AirPrint',
			status: 'État',
			running: 'En cours',
			stopped: 'Arrêté',
			printers: 'Imprimantes',
			noPrinters: 'Aucune imprimante configurée',
			noPrintersHint: 'Connectez une imprimante USB et cliquez sur « Scanner » pour la détecter',
			scan: 'Scanner les imprimantes',
			scanning: 'Recherche en cours...',
			addPrinter: 'Ajouter une imprimante',
			removePrinter: 'Supprimer',
			printerName: 'Nom de l\'imprimante',
			printerUri: 'Connexion',
			selectDriver: 'Sélectionner le pilote',
			loadingDrivers: 'Chargement des pilotes...',
			location: 'Emplacement',
			shared: 'Partagée',
			notShared: 'Non partagée',
			setDefault: 'Définir par défaut',
			default: 'Par défaut',
			testPage: 'Page de test',
			testPageSent: 'Page de test envoyée',
			protocols: 'Protocoles de partage',
			protocolsHint: 'Les imprimantes sont partagées via IPP (macOS/Linux/iOS) et découvrables via AirPrint. Les clients Windows peuvent se connecter via SMB si Samba est activé.',
			queue: 'File d\'impression',
			noJobs: 'Aucun travail d\'impression',
			cancelJob: 'Annuler',
			state: {
				idle: 'Inactif',
				processing: 'Impression',
				stopped: 'Arrêté',
				error: 'Erreur'
			},
			detected: 'Imprimantes détectées',
			noDetected: 'Aucune imprimante USB détectée',
			connectUsb: 'Connectez une imprimante USB à votre appareil',
			add: 'Ajouter',
			adding: 'Ajout en cours...',
			confirmRemove: 'Supprimer cette imprimante ?',
			confirmRemoveMessage: 'L\'imprimante ne sera plus partagée sur le réseau.'
		}
	},

	// Share Manager
	shareManager: {
		title: 'Dossiers partagés',
		createShare: 'Créer un partage',
		editShare: 'Modifier le partage',
		deleteShare: 'Supprimer le partage',
		description: 'Description',
		descriptionPlaceholder: 'Entrer la description du partage',
		advancedOptions: 'Options avancées',
		guestAccess: 'Accès invité',
		browseable: 'Navigable',
		readOnly: 'Lecture seule',
		createMask: 'Masque de permissions fichiers',
		directoryMask: 'Masque de permissions dossiers',
		recycleBin: 'Corbeille',
		timeMachine: 'Support Time Machine',
		timeMachineDesc: 'Activer la sauvegarde macOS Time Machine sur ce partage',
		smbEncrypt: 'Chiffrement',
		encryptOff: 'Désactivé',
		encryptDesired: 'Si supporté',
		encryptRequired: 'Obligatoire',
		hostsAllow: 'Hôtes autorisés',
		hostsDeny: 'Hôtes bloqués',
		hostsHint: 'IPs ou sous-réseaux séparés par des espaces (ex: 192.168.1.0/24)',
		auditLogging: 'Journal d\'audit',
		auditLoggingDesc: 'Journaliser les opérations fichiers (création, renommage, suppression)',
		vetoFiles: 'Fichiers interdits',
		vetoFilesHint: 'Pattern Samba veto (ex: /._*/.DS_Store/)',
		extraOptions: 'Options avancées',
		extraOptionsHint: 'Directives Samba brutes (une par ligne)',
		sambaNotRunning: 'Le service Samba n\'est pas actif. Activez-le dans les paramètres du Service de fichiers.',
		permissions: 'Permissions',
		toggleEnabled: 'Activer',
		toggleDisabled: 'Désactiver',
		fields: {
			name: 'Nom du partage',
			namePlaceholder: 'Entrez le nom du partage',
			path: 'Chemin',
			protocol: 'Protocole',
			enabled: 'Activé',
			users: 'Utilisateurs'
		},
		protocols: {
			smb: 'SMB/CIFS',
			nfs: 'NFS',
			ftp: 'FTP'
		},
		messages: {
			noShares: 'Aucun dossier partagé configuré',
			deleteConfirm: 'Êtes-vous sûr de vouloir supprimer ce partage ?',
			createSuccess: 'Partage créé avec succès',
			updateSuccess: 'Partage mis à jour avec succès',
			deleteSuccess: 'Partage supprimé avec succès',
			createError: 'Erreur lors de la création du partage',
			updateError: 'Erreur lors de la mise à jour du partage',
			deleteError: 'Erreur lors de la suppression du partage'
		}
	},

	// Folder Picker
	folderPicker: {
		title: 'Sélectionner un dossier',
		browse: 'Parcourir',
		select: 'Sélectionner',
		selected: 'Sélectionné',
		location: 'Emplacement',
		noSubfolders: 'Aucun sous-dossier',
		emptyFolder: 'Ce dossier est vide',
		noLocations: 'Aucun emplacement disponible',
		loading: 'Chargement...',
		error: 'Échec du chargement des dossiers'
	},

	// Docker App
	docker: {
		search: 'Rechercher...',
		noResults: 'Aucun résultat',
		serviceStatus: 'Service en cours',
		status: {
			normal: 'Normal',
			stopped: 'Arrêté',
			error: 'Erreur'
		},
		stats: {
			containers: 'Conteneurs',
			images: 'Images',
			volumes: 'Volumes',
			networks: 'Réseaux'
		},
		cpuUsage: 'Utilisation CPU',
		memoryCapacity: 'Capacité mémoire',
		available: 'Disponible',
		views: {
			overview: 'Aperçu',
			container: 'Conteneurs',
			image: 'Images',
			volume: 'Volumes',
			network: 'Réseaux'
		},
		table: {
			name: 'Nom',
			image: 'Image',
			status: 'Statut',
			ports: 'Ports',
			actions: 'Actions',
			repository: 'Dépôt',
			tag: 'Tag',
			imageId: 'ID Image',
			id: 'ID',
			size: 'Taille',
			created: 'Créé',
			driver: 'Pilote',
			mountPoint: 'Point de montage',
			scope: 'Portée'
		},
		noContainers: 'Aucun conteneur trouvé',
		noImages: 'Aucune image trouvée',
		confirm: {
			stopTitle: 'Arrêter le conteneur',
			stopMessage: 'Êtes-vous sûr de vouloir arrêter le conteneur "{name}" ?',
			stopBtn: 'Arrêter',
			startTitle: 'Démarrer le conteneur',
			startMessage: 'Êtes-vous sûr de vouloir démarrer le conteneur "{name}" ?',
			startBtn: 'Démarrer',
			restartTitle: 'Redémarrer le conteneur',
			restartMessage: 'Êtes-vous sûr de vouloir redémarrer le conteneur "{name}" ?',
			restartBtn: 'Redémarrer',
			removeContainerTitle: 'Supprimer le conteneur',
			removeContainerMessage: 'Êtes-vous sûr de vouloir supprimer le conteneur "{name}" ? Cette action est irréversible.',
			removeImageTitle: 'Supprimer l\'image',
			removeImageMessage: 'Êtes-vous sûr de vouloir supprimer l\'image "{name}" ? Cette action est irréversible.',
			removeVolumeTitle: 'Supprimer le volume',
			removeVolumeMessage: 'Êtes-vous sûr de vouloir supprimer le volume "{name}" ? Toutes les données seront perdues.',
			removeNetworkTitle: 'Supprimer le réseau',
			removeNetworkMessage: 'Êtes-vous sûr de vouloir supprimer le réseau "{name}" ?',
			removeBtn: 'Supprimer',
			pruneImagesTitle: 'Nettoyer les images',
			pruneImagesMessage: 'Cela supprimera toutes les images inutilisées. Cette action est irréversible.',
			pruneImagesBtn: 'Nettoyer',
			pruneVolumesTitle: 'Nettoyer les volumes',
			pruneVolumesMessage: 'Cela supprimera tous les volumes inutilisés. Toutes les données de ces volumes seront perdues.',
			pruneVolumesBtn: 'Nettoyer'
		},
		logs: {
			title: 'Journaux',
			noLogs: 'Aucun journal disponible',
			refresh: 'Actualiser',
			lines: 'lignes'
		},
		volumes: {
			title: 'Volumes',
			noVolumes: 'Aucun volume trouvé'
		},
		networks: {
			title: 'Réseaux',
			noNetworks: 'Aucun réseau trouvé',
			builtIn: 'intégré'
		},
		pull: {
			placeholder: 'Nom de l\'image (ex: nginx:latest)',
			button: 'Télécharger',
			pulling: 'Téléchargement...'
		}
	},

	// Mise à jour système
	systemUpdate: {
		title: 'Mise à jour système',
		currentVersion: 'Version actuelle',
		checkForUpdates: 'Vérifier les mises à jour',
		checking: 'Vérification...',
		upToDate: 'Votre système est à jour',
		upToDateDesc: 'Vous utilisez la dernière version de PiNAS.',
		updateAvailable: 'Mise à jour disponible',
		downloadSize: 'Taille du téléchargement',
		noRebootRequired: 'Aucun redémarrage nécessaire',
		rebootRequired: 'Redémarrage nécessaire après la mise à jour',
		installUpdate: 'Installer la mise à jour',
		installing: 'Installation en cours...',
		downloading: 'Téléchargement en cours...',
		applying: 'Application de la mise à jour...',
		restarting: 'Redémarrage du service...',
		updateHistory: 'Historique des mises à jour',
		noHistory: 'Aucun historique de mise à jour.',
		failedToCheck: 'Échec de la vérification des mises à jour',
		failedToInstall: 'Échec de l\'installation de la mise à jour',
		modal: {
			title: 'Mise à jour réussie',
			subtitle: 'PiNAS a été mis à jour avec succès',
			updatedTo: 'Mis à jour vers la version',
			changelog: 'Nouveautés',
			dismiss: 'Parfait !',
			thanks: 'Merci d\'utiliser PiNAS.'
		},
		screen: {
			starting: 'La mise à jour va démarrer...',
			doNotTurnOff: 'Veuillez ne pas éteindre l\'appareil',
			inProgress: 'Mise à jour en cours',
			completed: 'Mise à jour terminée avec succès !',
			installed: 'PiNAS {version} installé',
			failed: 'La mise à jour a échoué',
			reboot: 'Redémarrer',
			reloadDesktop: 'Recharger le bureau',
			restarting: 'Redémarrage du service...',
			close: 'Fermer',
			confirm: {
				title: 'Installer la mise à jour v{version} ?',
				description: 'Cette opération va mettre à jour votre système. Ne débranchez pas l\'appareil.',
				cancel: 'Annuler',
				confirm: 'Confirmer'
			},
			devTest: 'Tester l\'écran de mise à jour'
		}
	}
};
