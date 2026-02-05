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
		searchApplications: 'Rechercher des applications...',
		refresh: 'Actualiser',
		retry: 'Réessayer',
		copy: 'Copier'
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
			email: 'Email',
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
		kodi: 'Kodi'
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

	// Onboarding
	onboarding: {
		welcome: 'Bienvenue !',
		subtitle: 'Configurons votre NAS.',
		steps: {
			language: 'Langue',
			device: 'Appareil',
			account: 'Compte',
			password: 'Mot de passe'
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
		buttons: {
			back: 'Retour',
			next: 'Suivant',
			complete: 'Terminer la configuration'
		},
		validation: {
			machineNameRequired: "Le nom de l'appareil est requis",
			machineNameMinLength: "Le nom doit contenir au moins 2 caractères",
			machineNameInvalid: 'Seuls les lettres, chiffres et tirets sont autorisés',
			usernameRequired: "Le nom d'utilisateur est requis",
			usernameMinLength: "Le nom d'utilisateur doit contenir au moins 3 caractères",
			usernameInvalid: 'Seuls les lettres, chiffres et underscores sont autorisés',
			passwordRequired: 'Le mot de passe est requis',
			passwordMinLength: 'Le mot de passe doit contenir au moins 6 caractères',
			passwordMismatch: 'Les mots de passe ne correspondent pas'
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
			volumes: 'Volumes'
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
			title: 'Disques durs',
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

	// Docker App
	docker: {
		serviceStatus: 'Service en cours',
		status: {
			normal: 'Normal',
			stopped: 'Arrêté',
			error: 'Erreur'
		},
		stats: {
			projects: 'Projets',
			containers: 'Conteneurs',
			local: 'Local',
			data: 'Données'
		},
		cpuUsage: 'Utilisation CPU',
		memoryCapacity: 'Capacité mémoire',
		available: 'Disponible',
		views: {
			overview: 'Aperçu',
			project: 'Projet',
			container: 'Conteneur',
			image: 'Image',
			network: 'Réseau',
			log: 'Journal',
			management: 'Gestion'
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
			size: 'Taille',
			created: 'Créé'
		},
		noContainers: 'Aucun conteneur trouvé',
		noImages: 'Aucune image trouvée',
		underDevelopment: 'Cette section est en cours de développement'
	}
};
