export default {
	// Common
	common: {
		save: 'Save',
		cancel: 'Cancel',
		delete: 'Delete',
		edit: 'Edit',
		add: 'Add',
		create: 'Create',
		close: 'Close',
		apply: 'Apply',
		reset: 'Reset',
		search: 'Search',
		filter: 'Filter',
		actions: 'Actions',
		status: 'Status',
		enabled: 'Enabled',
		disabled: 'Disabled',
		yes: 'Yes',
		no: 'No',
		loading: 'Loading...',
		noData: 'No data',
		confirm: 'Confirm',
		back: 'Back',
		next: 'Next',
		previous: 'Previous',
		open: 'Open',
		removeFromDesktop: 'Remove from Desktop',
		addToDesktop: 'Add to Desktop',
		addToDock: 'Add to Dock',
		removeFromDock: 'Remove from Dock',
		pinToDock: 'Pin to Dock',
		noApplicationsFound: 'No applications found',
		searchApplications: 'Search applications...',
		refresh: 'Refresh',
		retry: 'Retry',
		copy: 'Copy'
	},

	// Password validation rules
	passwordRules: {
		minLength: 'At least 8 characters',
		passwordsMatch: 'Passwords match'
	},

	// Control Panel
	controlPanel: {
		title: 'Control Panel',
		backToGrid: 'Back to grid view',
		categories: {
			connectionAccess: 'Connection & Access',
			general: 'General',
			service: 'Service'
		},
		items: {
			userManagement: 'User Management',
			fileService: 'File Service',
			deviceConnection: 'Device Connection',
			domainLdap: 'Domain/LDAP',
			terminal: 'Terminal',
			printer: 'Printer',
			hardwarePower: 'Hardware & Power',
			timeLanguage: 'Time & Language',
			network: 'Network',
			security: 'Security',
			indexingService: 'Indexing Service',
			systemUpdate: 'System Update',
			about: 'About'
		},
		tabs: {
			general: 'General',
			storage: 'Storage',
			service: 'Service',
			deviceAnalysis: 'Device analysis'
		},
		about: {
			deviceName: 'Device Name',
			systemVersion: 'System Version',
			deviceOwner: 'Device Owner',
			device: 'Device',
			model: 'Model',
			serialNumber: 'SN',
			lastStartup: 'Last startup',
			powerTime: 'Power Time',
			hardware: 'Hardware',
			cpu: 'CPU',
			memory: 'Memory',
			network: 'Network',
			macAddress: 'Mac Address/Subnet mask',
			cores: 'Cores',
			threads: 'Threads'
		},
		underDevelopment: 'This section is under development',
		contentFor: 'Content for {tab} tab'
	},

	// Time & Language
	timeLanguage: {
		title: 'Time & Language',
		tabs: {
			time: 'Time',
			language: 'Language'
		},
		time: {
			title: 'Date & Time',
			timezone: 'Time Zone',
			selectTimezone: 'Select time zone',
			currentTime: 'Current Time',
			dateFormat: 'Date Format',
			timeFormat: 'Time Format',
			format24h: '24-hour',
			format12h: '12-hour',
			ntpServer: 'NTP Server',
			syncWithNtp: 'Synchronize with NTP server',
			syncNow: 'Sync Now',
			lastSync: 'Last synchronized'
		},
		language: {
			title: 'Display Language',
			selectLanguage: 'Select Language',
			currentLanguage: 'Current Language',
			restart: 'Some changes may require a refresh to take effect'
		}
	},

	// User Manager
	userManager: {
		title: 'User Management',
		tabs: {
			user: 'User',
			userGroup: 'User Group',
			advancedSettings: 'Advanced Settings'
		},
		table: {
			username: 'Username',
			email: 'Email',
			description: 'Description',
			role: 'Role',
			status: 'Status',
			edit: 'Edit',
			groupName: 'Group Name',
			members: 'Members'
		},
		roles: {
			administrator: 'Administrator',
			user: 'User',
			guest: 'Guest'
		},
		badges: {
			system: 'System'
		},
		statuses: {
			normal: 'Normal',
			disabled: 'Disabled'
		},
		actions: {
			addUser: 'Add User',
			addGroup: 'Add Group',
			editUser: 'Edit User',
			editGroup: 'Edit Group',
			deleteUser: 'Delete User',
			deleteGroup: 'Delete Group'
		},
		modals: {
			addUserTitle: 'Add User',
			editUserTitle: 'Edit User',
			deleteUserTitle: 'Delete User',
			addGroupTitle: 'Add Group',
			editGroupTitle: 'Edit Group',
			deleteGroupTitle: 'Delete Group',
			permissionViewer: 'Permission viewer'
		},
		fields: {
			username: 'Username',
			email: 'Email',
			password: 'Password',
			confirmPassword: 'Confirm Password',
			description: 'Description',
			role: 'Role',
			status: 'Status',
			changePassword: 'Change password',
			newPassword: 'New Password',
			groupName: 'Group Name',
			selectMembers: 'Select Members'
		},
		permissions: {
			title: 'Permissions',
			user: 'User',
			users: 'Users',
			groups: 'Groups',
			sharedFolder: 'Shared folder',
			noAccess: 'No Access',
			readOnly: 'Read Only',
			readWrite: 'Read/Write',
			addFolder: 'Add Folder',
			folderPath: 'Folder Path',
			folderPathHint: 'Enter the full path to the folder (e.g., /storage/shares/documents)',
			noPermissions: 'No folder permissions configured',
			noPermissionsHint: 'Add a folder to start managing permissions',
			confirmRemoveFolder: 'Remove all permissions for {path}?'
		},
		messages: {
			deleteConfirm: 'Are you sure you want to delete user "{username}"?',
			deleteGroupConfirm: 'Are you sure you want to delete group "{groupName}"?',
			cannotBeUndone: 'This action cannot be undone.',
			userDataShown: 'User data currently shown',
			me: 'Me',
			noUsersFound: 'No users found',
			noGroupsFound: 'No groups found',
			cannotChangeOwnRole: 'You cannot change your own role',
			systemGroupNameReadonly: 'System group names cannot be changed'
		},
		advancedSettings: {
			passwordStrength: {
				title: 'Password strength rules',
				noUserNames: 'Do not use user names',
				noCommonPasswords: 'Do not use common passwords',
				requireUpperLower: 'Must contain both uppercase and lowercase letters',
				requireNumber: 'At least 1 number',
				requireSpecialChar: 'At least 1 special character',
				minLength: 'Minimum password length',
				digits: 'digits'
			},
			passwordExpiry: {
				title: 'Password expiry rules',
				enabled: 'Enable password expiry rules',
				validityPeriod: 'Password validity period',
				reminderDays: 'Reminder days in advance (before password expires)',
				forceChange: 'Password change required after expiry',
				permanentUsers: 'Permanent password users',
				days: 'days'
			}
		}
	},

	// Desktop & App Launcher
	desktop: {
		appLauncher: {
			title: 'Applications',
			categories: {
				all: 'All',
				system: 'System',
				storageFiles: 'Storage & Files',
				services: 'Services'
			}
		}
	},

	// Application names
	apps: {
		controlPanel: 'Control Panel',
		files: 'Files',
		appCenter: 'App Center',
		storage: 'Storage',
		shares: 'Shares',
		docker: 'Docker',
		terminal: 'Terminal',
		users: 'Users',
		processManager: 'Process Manager',
		kodi: 'Kodi',
		display: 'Display'
	},

	// Top Bar
	topBar: {
		search: 'Search...',
		notifications: 'Notifications',
		noNotifications: 'No notifications',
		clearAll: 'Clear all',
		userMenu: {
			profile: 'Profile',
			settings: 'Settings',
			logout: 'Logout',
			restart: 'Restart',
			shutdown: 'Shut Down',
			confirmRestart: 'Are you sure you want to restart the system?',
			confirmShutdown: 'Are you sure you want to shut down the system?',
			restartingMessage: 'The system is restarting...',
			shuttingDownMessage: 'The system is shutting down...',
			restartingSubtitle: 'Please wait, this may take a moment',
			shuttingDownSubtitle: 'You can safely unplug the device',
			systemRestarted: 'System restarted successfully',
			systemShutDown: 'System is shutting down',
			reloadDesktop: 'Reload desktop',
		}
	},

	// System Widgets
	widgets: {
		cpu: 'CPU',
		memory: 'Memory',
		network: 'Network',
		storage: 'Storage',
		upload: 'Upload',
		download: 'Download'
	},

	// Window Controls
	window: {
		minimize: 'Minimize',
		maximize: 'Maximize',
		restore: 'Restore',
		close: 'Close',
		help: 'Help'
	},

	// Notifications
	notifications: {
		title: 'Notifications',
		markAsRead: 'Mark as read',
		clearAll: 'Clear all',
		empty: 'No notifications'
	},

	// Onboarding
	onboarding: {
		welcome: 'Welcome!',
		subtitle: "Let's set up your NAS.",
		steps: {
			welcome: 'Welcome',
			language: 'Language',
			device: 'Device',
			account: 'Account',
			password: 'Password',
			ssh: 'SSH',
			features: 'Discover'
		},
		welcomeScreen: {
			title: 'Welcome to PiNAS!',
			thankYou: 'Thank you for installing PiNAS on your Raspberry Pi.',
			description: 'In just a few steps, we\'ll set up your personal NAS and you\'ll be ready to store, share and manage your files.',
			letsGo: 'Let\'s get started!'
		},
		language: {
			title: 'Choose your language',
			description: 'Select the language for the interface.',
			selectLanguage: 'Select Language'
		},
		device: {
			title: 'Name your device',
			description: 'This name will identify your NAS on the network.',
			fieldLabel: 'Device Name',
			placeholder: 'e.g., pinas-home'
		},
		account: {
			title: 'Create admin account',
			description: 'This will be the administrator account for your NAS.',
			fieldLabel: 'Username',
			placeholder: 'e.g., admin'
		},
		password: {
			title: 'Set your password',
			description: 'Choose a secure password for your admin account.',
			fieldLabel: 'Password',
			confirmLabel: 'Confirm Password',
			placeholder: 'Enter password',
			confirmPlaceholder: 'Confirm password'
		},
		ssh: {
			title: 'Remote access (SSH)',
			description: 'SSH lets you connect to your NAS remotely via terminal.',
			enableSsh: 'Enable SSH',
			enableHint: 'Allow remote terminal access to your device',
			port: 'Port',
			passwordLabel: 'SSH Password',
			passwordPlaceholder: 'Enter SSH password',
			confirmPasswordLabel: 'Confirm Password',
			confirmPasswordPlaceholder: 'Confirm SSH password',
			passwordHint: 'This password will be used to connect as root via SSH.',
			skipHint: 'You can always enable SSH later in Settings.'
		},
		features: {
			title: 'Your NAS is ready!',
			subtitle: 'Here\'s what you can do with PiNAS:',
			items: {
				files: {
					title: 'File Manager',
					description: 'Browse, upload and organize all your files.'
				},
				storage: {
					title: 'Storage',
					description: 'Manage your disks, create RAID pools and volumes.'
				},
				shares: {
					title: 'File Sharing',
					description: 'Share folders on your network via SMB.'
				},
				docker: {
					title: 'Docker & Apps',
					description: 'Install apps like Plex, Pi-hole and more.'
				},
				kodi: {
					title: 'Media Center',
					description: 'Kodi is built-in for your movies and music.'
				},
				terminal: {
					title: 'Terminal',
					description: 'Full shell access right from your browser.'
				}
			}
		},
		buttons: {
			back: 'Back',
			next: 'Next',
			complete: 'Start using PiNAS',
			getStarted: 'Let\'s go!'
		},
		validation: {
			machineNameRequired: 'Machine name is required',
			machineNameMinLength: 'Machine name must be at least 2 characters',
			machineNameInvalid: 'Only letters, numbers and hyphens allowed',
			usernameRequired: 'Username is required',
			usernameMinLength: 'Username must be at least 3 characters',
			usernameInvalid: 'Only letters, numbers and underscores allowed',
			passwordRequired: 'Password is required',
			passwordMinLength: 'Password must be at least 8 characters',
			passwordMismatch: 'Passwords do not match',
			sshPasswordMinLength: 'SSH password must be at least 8 characters',
			sshPasswordMismatch: 'SSH passwords do not match'
		}
	},

	// File Manager
	fileManager: {
		title: 'Files',
		sidebar: {
			personalFolder: 'Personal folder',
			sharedFolder: 'Shared folder',
			userFolder: 'User folder'
		},
		sections: {
			personal: 'Personal',
			shares: 'Shared Folders',
			volumes: 'Volumes',
			media: 'Removable Media'
		},
		statuses: {
			mounted: 'Mounted',
			unmounted: 'Unmounted',
			disabled: 'Disabled'
		},
		noLocations: 'No locations available',
		toolbar: {
			refresh: 'Refresh',
			search: 'Search...',
			newFolder: 'New folder',
			newFile: 'New file',
			upload: 'Upload',
			download: 'Download',
			copy: 'Copy',
			paste: 'Paste',
			cut: 'Cut',
			duplicate: 'Duplicate',
			archive: 'Archive',
			settings: 'Settings',
			sort: 'Sort'
		},
		columns: {
			name: 'Name',
			size: 'Size',
			type: 'Type',
			modified: 'Date modified'
		},
		types: {
			folder: 'Folder',
			file: 'File',
			trash: 'Trash'
		},
		viewModes: {
			list: 'List',
			grid: 'Grid',
			compact: 'Compact'
		},
		contextMenu: {
			open: 'Open',
			download: 'Download',
			rename: 'Rename',
			copy: 'Copy',
			cut: 'Cut',
			delete: 'Delete',
			newFolder: 'New Folder',
			newFile: 'New File',
			upload: 'Upload',
			paste: 'Paste',
			selectAll: 'Select All',
			properties: 'Properties'
		},
		modals: {
			newFolderTitle: 'New Folder',
			newFileTitle: 'New File',
			folderNamePlaceholder: 'Folder name',
			fileNamePlaceholder: 'File name',
			deleteTitle: 'Delete',
			deleteFolderMessage: 'Delete this folder and all its contents?',
			deleteFileMessage: 'Delete this file?',
			emptyFolder: 'This folder is empty'
		},
		statusBar: {
			item: 'item in total',
			items: 'items in total',
			selected: 'selected'
		}
	},

	// Task Manager
	taskManager: {
		title: 'Tasks',
		clear: 'Clear completed',
		empty: 'No tasks',
		types: {
			upload: 'Upload',
			copy: 'Copy',
			move: 'Move',
			delete: 'Delete',
			createFolder: 'Create folder',
			createFile: 'Create file'
		}
	},

	// App Center
	appCenter: {
		title: 'App Center',
		searchPlaceholder: 'Search applications...',
		installedCount: 'installed',
		noPackages: 'No applications found',
		version: 'Version',
		description: 'Description',
		features: 'Features',
		categories: {
			all: 'All',
			containers: 'Containers',
			media: 'Media',
			network: 'Network',
			utilities: 'Utilities'
		},
		status: {
			installed: 'Installed',
			installing: 'Installing...',
			updateAvailable: 'Update available',
			notInstalled: 'Not installed'
		},
		actions: {
			install: 'Install',
			installing: 'Installing...',
			uninstall: 'Uninstall',
			uninstalling: 'Uninstalling...',
			open: 'Open',
			update: 'Update'
		},
		uninstallModal: {
			title: 'Uninstall Application',
			message: 'Do you want to uninstall',
			deleteData: 'Delete application data'
		},
		missingDependencies: 'Missing dependencies',
		requiresInstall: 'Requires',
		dependencies: 'Dependencies',
		packages: {
			docker: {
				description: 'Container platform for deploying and managing applications',
				feature1: 'Run isolated containers',
				feature2: 'Easy application deployment',
				feature3: 'Docker Compose support'
			}
		}
	},

	// Generic App Components
	iframeApp: {
		connectionError: 'Unable to connect to the application',
		timeout: 'Connection timeout - application may not be running',
		openExternal: 'Open in new tab',
		errorTitle: 'Connection Error'
	},

	webviewApp: {
		description: 'This application opens in a new browser tab',
		openApp: 'Open Application',
		checking: 'Checking status...',
		online: 'Online',
		offline: 'Offline',
		tip: 'You can also access this application directly at the URL above'
	},

	serviceApp: {
		running: 'Running',
		stopped: 'Stopped',
		status: 'Status',
		logs: 'Logs',
		config: 'Configuration',
		actions: 'Actions',
		start: 'Start',
		stop: 'Stop',
		restart: 'Restart',
		uptime: 'Uptime',
		memory: 'Memory',
		cpu: 'CPU',
		autostart: 'Auto-start',
		recentLogs: 'Recent Logs',
		noLogs: 'No logs available',
		configPlaceholder: 'Configuration options coming soon'
	},

	// Storage Manager
	storageManager: {
		title: 'Storage Manager',
		sidebar: {
			overview: 'Overview',
			storage: 'Storage',
			hardDisk: 'Hard disk',
			externalStorage: 'External storage'
		},
		tabs: {
			poolsVolumes: 'Storage pool & volume',
			dataOrganizing: 'Data organizing',
			advancedSettings: 'Advanced settings'
		},
		overview: {
			title: 'Storage Overview',
			disks: 'Disks',
			pools: 'Pools',
			volumes: 'Volumes',
			totalCapacity: 'Total Capacity'
		},
		pools: {
			title: 'Storage Pools',
			create: 'Create',
			createPool: 'Storage pool',
			createVolume: 'Volume',
			noPoolsConfigured: 'No storage pools configured',
			noPools: 'No pools',
			pool: 'pool',
			pools: 'pools',
			disk: 'disk',
			disks: 'disks',
			noVolumes: 'No volumes in this pool',
			createVolumeLink: 'Create volume'
		},
		volumes: {
			title: 'Volumes',
			used: 'Used',
			mount: 'Mount',
			unmount: 'Unmount'
		},
		disks: {
			title: 'Storage Disks',
			systemDisk: 'System Disk',
			system: 'System',
			smart: 'S.M.A.R.T.',
			details: 'Details',
			wipe: 'Wipe',
			noExternalDevices: 'No external storage devices connected'
		},
		status: {
			normal: 'Normal',
			degraded: 'Degraded',
			rebuilding: 'Rebuilding',
			error: 'Error',
			creating: 'Creating',
			mounted: 'Mounted',
			unmounted: 'Unmounted'
		},
		raidTypes: {
			basic: 'Basic',
			jbod: 'JBOD',
			raid0: 'RAID 0',
			raid1: 'RAID 1',
			raid5: 'RAID 5',
			raid10: 'RAID 10',
			btrfsSingle: 'Btrfs Single',
			btrfsRaid1: 'Btrfs RAID1',
			basicDesc: 'Single disk, no redundancy. Data lost if disk fails.',
			jbodDesc: 'Just a bunch of disks. No redundancy, combines capacity.',
			raid0Desc: 'Striping. Maximum performance, no redundancy.',
			raid1Desc: 'Mirroring. 50% capacity, survives 1 disk failure.',
			raid5Desc: 'Striping with parity. Survives 1 disk failure.',
			raid10Desc: 'Striping + Mirroring. High performance, survives failures.',
			btrfsSingleDesc: 'Btrfs filesystem, no redundancy.',
			btrfsRaid1Desc: 'Btrfs mirroring. Survives 1 disk failure.'
		},
		modals: {
			createPool: {
				title: 'Create storage pool',
				preview: 'Preview',
				poolName: 'Pool name',
				selectHardDisk: 'Select hard disk',
				noDisksAvailable: 'No available disks found',
				selectRaidType: 'Select RAID type',
				available: 'Available',
				wipeDisks: 'Wipe selected disks before creating pool',
				total: 'Total',
				raidType: 'RAID type',
				useHardDisk: 'Use hard disk',
				recommended: 'Rec.'
			},
			createVolume: {
				title: 'Create Volume',
				storagePool: 'Storage Pool',
				selectPool: 'Select a pool...',
				volumeName: 'Volume Name',
				fileSystem: 'File System'
			},
			deletePool: {
				title: 'Delete Pool',
				confirmMessage: 'Are you sure you want to delete',
				willDeleteVolumes: 'This will also delete',
				volume: 'volume',
				volumes: 'volumes',
				cannotBeUndone: 'This action cannot be undone.'
			},
			deleteVolume: {
				title: 'Delete Volume',
				confirmMessage: 'Are you sure you want to delete',
				dataLost: 'All data on this volume will be lost.',
				cannotBeUndone: 'This action cannot be undone.'
			},
			smartInfo: {
				title: 'S.M.A.R.T. Information',
				temperature: 'Temperature',
				powerOnHours: 'Power On Hours',
				powerCycles: 'Power Cycles',
				serial: 'Serial',
				attributes: 'Attributes',
				loadingData: 'Loading S.M.A.R.T. data...',
				loadFailed: 'Failed to load S.M.A.R.T. information'
			},
			editPool: {
				title: 'Edit Pool',
				poolName: 'Pool Name',
				description: 'Description'
			},
			wipeDisk: {
				title: 'Wipe Disk',
				warning: 'Warning: This operation is destructive!',
				confirmMessage: 'Are you sure you want to wipe',
				allDataLost: 'All data on this disk will be permanently erased.',
				cannotBeUndone: 'This action cannot be undone.',
				wiping: 'Wiping...'
			},
			diskDetails: {
				title: 'Disk Details',
				totalSize: 'Total Size',
				type: 'Type',
				serial: 'Serial Number',
				deviceId: 'Device ID',
				partitions: 'Partitions',
				noPartitions: 'No partitions found on this disk'
			}
		},
		contextMenu: {
			createVolume: 'Create volume',
			edit: 'Edit',
			scrub: 'Scrub',
			scrubbing: 'Scrubbing...',
			changeRaidType: 'Change RAID type',
			delete: 'Delete'
		},
		messages: {
			loading: 'Loading storage information...',
			comingSoon: 'coming soon',
			dataOrganizingComingSoon: 'Data organizing features coming soon',
			advancedSettingsComingSoon: 'Advanced settings coming soon',
			empty: 'Empty'
		},
		errors: {
			loadFailed: 'Failed to load storage data',
			createPoolFailed: 'Failed to create pool',
			createVolumeFailed: 'Failed to create volume',
			deletePoolFailed: 'Failed to delete pool',
			deleteVolumeFailed: 'Failed to delete volume',
			toggleMountFailed: 'Failed to toggle volume mount',
			scrubFailed: 'Failed to start scrub operation',
			editPoolFailed: 'Failed to update pool',
			wipeDiskFailed: 'Failed to wipe disk'
		}
	},

	// Process Manager
	processManager: {
		title: 'Process Manager',
		processes: 'Processes',
		running: 'Running',
		endProcess: 'End process',
		confirmKill: 'Are you sure you want to terminate this process?',
		noProcesses: 'No processes found',
		columns: {
			name: 'Name',
			user: 'User',
			memory: 'Memory',
			status: 'Status'
		},
		status: {
			running: 'Running',
			sleeping: 'Sleeping',
			stopped: 'Stopped',
			zombie: 'Zombie',
			idle: 'Idle',
			unknown: 'Unknown'
		},
		errors: {
			loadFailed: 'Failed to load processes',
			killFailed: 'Failed to terminate process'
		}
	},

	// Terminal App
	terminalApp: {
		welcome: 'PiNAS Terminal v1.0',
		helpHint: 'Type "help" for available commands.',
		tabHint: 'Use Tab for path auto-completion.',
		errors: {
			authRequired: 'Authentication required. Please log in again.',
			adminRequired: 'Admin access required to use the terminal.',
			rateLimit: 'Rate limit exceeded. Please wait before executing more commands.',
			connectionFailed: 'Connection failed'
		}
	},

	// Display App
	display: {
		title: 'Display',
		description: 'Manage HDMI display services',
		services: 'Display Services',
		noActiveService: 'No active display service',
		splashActive: 'PiNAS splash screen is displayed',
		splashDescription: 'Access PiNAS at:',
		activate: 'Activate',
		deactivate: 'Deactivate',
		activating: 'Starting...',
		deactivating: 'Stopping...'
	},

	// Kodi App
	kodi: {
		title: 'Kodi',
		tabs: {
			remote: 'Remote',
			sources: 'Sources',
			settings: 'Settings',
			addons: 'Add-ons',
			library: 'Library'
		},
		status: {
			connected: 'Connected',
			disconnected: 'Disconnected',
			playing: 'Playing',
			paused: 'Paused',
			stopped: 'Stopped'
		},
		remote: {
			nowPlaying: 'Now Playing',
			nothingPlaying: 'Nothing playing',
			volume: 'Volume',
			mute: 'Mute',
			playback: 'Playback',
			previous: 'Previous',
			next: 'Next',
			stop: 'Stop',
			navigation: 'Navigation',
			back: 'Back',
			home: 'Home',
			menu: 'Menu',
			info: 'Info'
		},
		sources: {
			title: 'Media Sources',
			addSource: 'Add Source',
			editSource: 'Edit Source',
			deleteSource: 'Delete Source',
			noSources: 'No media sources configured',
			type: 'Type',
			path: 'Path',
			name: 'Name',
			types: {
				smb: 'SMB/CIFS',
				nfs: 'NFS',
				local: 'Local'
			},
			mediaTypes: {
				video: 'Video',
				music: 'Music',
				pictures: 'Pictures',
				files: 'Files'
			},
			allTypes: 'All types',
			pathFormats: 'Formats: smb://server/share, nfs://server/path, /storage/path',
			fields: {
				sourceName: 'Source Name',
				sourceType: 'Source Type',
				mediaType: 'Media Type',
				serverPath: 'Server/Path',
				username: 'Username',
				password: 'Password'
			},
			deleteConfirm: 'Are you sure you want to delete this source?'
		},
		settings: {
			title: 'Kodi Settings',
			categories: {
				player: 'Player',
				media: 'Media',
				interface: 'Interface',
				services: 'Services',
				system: 'System'
			},
			player: {
				skipSteps: 'Skip steps (seconds)',
				defaultPlayer: 'Default player'
			},
			media: {
				showHiddenFiles: 'Show hidden files',
				autoScanLibrary: 'Auto-scan library'
			},
			interface: {
				language: 'Language',
				skin: 'Skin',
				soundsEnabled: 'Interface sounds'
			},
			services: {
				webServer: 'Web server',
				airplay: 'AirPlay',
				upnp: 'UPnP/DLNA'
			},
			system: {
				powerSaving: 'Power saving',
				debugLogging: 'Debug logging'
			}
		},
		addons: {
			title: 'Add-ons',
			installed: 'Installed',
			available: 'Available',
			enable: 'Enable',
			disable: 'Disable',
			noAddons: 'No add-ons found'
		},
		library: {
			title: 'Library',
			videoLibrary: 'Video Library',
			musicLibrary: 'Music Library',
			scan: 'Scan',
			clean: 'Clean',
			scanVideo: 'Scan Video Library',
			scanMusic: 'Scan Music Library',
			cleanVideo: 'Clean Video Library',
			cleanMusic: 'Clean Music Library',
			scanning: 'Scanning...',
			cleaning: 'Cleaning...',
			scanStarted: 'Library scan started',
			lastScanned: 'Last scanned',
			movies: 'Movies',
			tvShows: 'TV Shows',
			albums: 'Albums',
			artists: 'Artists'
		},
		actions: {
			reboot: 'Reboot Kodi',
			shutdown: 'Shutdown Kodi',
			sendNotification: 'Send Notification'
		},
		errors: {
			connectionFailed: 'Failed to connect to Kodi',
			actionFailed: 'Action failed',
			loadFailed: 'Failed to load Kodi data',
			addSourceFailed: 'Failed to add source',
			removeSourceFailed: 'Failed to remove source',
			updateSettingFailed: 'Failed to update setting',
			toggleAddonFailed: 'Failed to toggle addon',
			scanFailed: 'Failed to scan library'
		}
	},

	// Network Settings
	networkSettings: {
		title: 'Network',
		description: 'Configure network interfaces and DNS',
		tabs: {
			general: 'General',
			interfaces: 'Network Interface'
		},
		hostname: 'Hostname',
		hostnameHint: 'Name of your device on the network',
		defaultGateway: 'Default Gateway',
		dnsServer: 'DNS Server',
		configureDnsManually: 'Configure DNS server manually',
		primaryDns: 'Primary DNS',
		secondaryDns: 'Secondary DNS',
		dnsAuto: 'DNS provided by DHCP',
		ipAddress: 'IP Address',
		subnetMask: 'Subnet Mask',
		gateway: 'Gateway',
		macAddress: 'MAC Address',
		speed: 'Speed',
		method: 'Method',
		manual: 'Manual',
		connected: 'Connected',
		disconnected: 'Disconnected',
		editInterface: 'Edit Interface',
		noInterfaces: 'No network interfaces detected'
	},

	// File Service
	fileService: {
		title: 'File Services',
		ssh: {
			description: 'Secure Shell access for remote administration',
			enable: 'Enable SSH',
			enableHint: 'Allow remote terminal access',
			status: 'Status',
			running: 'Running',
			stopped: 'Stopped',
			port: 'Port',
			password: 'Password',
			passwordHint: 'Used for SSH login as root',
			changePassword: 'Change Password',
			newPassword: 'New Password',
			confirmPassword: 'Confirm Password',
			passwordDescription: 'This password is used to connect via SSH as the root user.',
			passwordTooShort: 'Password must be at least 8 characters',
			passwordMismatch: 'Passwords do not match',
			connectionInfo: 'Connect using:'
		},
		smb: {
			description: 'Windows file sharing protocol',
			enable: 'Enable SMB/CIFS',
			enableHint: 'Start Samba service to share folders on the network',
			status: 'Status',
			running: 'Running',
			stopped: 'Stopped',
			connectedUsers: 'Connected Users',
			version: 'Samba Version',
			workgroup: 'Workgroup',
			serverDescription: 'Server Description',
			minProtocol: 'Minimum Protocol',
			maxProtocol: 'Maximum Protocol',
			apply: 'Apply',
			applying: 'Applying...',
			configUpdated: 'SMB configuration updated',
			manageShares: 'Manage Shared Folders',
			globalSettings: 'Global Settings',
			activeShares: 'Active Shares'
		},
		nfs: {
			description: 'Network File System for Unix/Linux'
		},
		ftp: {
			description: 'File Transfer Protocol'
		},
		cups: {
			title: 'Printer Sharing',
			description: 'Share USB printers across your network',
			enable: 'Enable Printer Sharing',
			enableHint: 'Start CUPS service to share printers over IPP/AirPrint',
			status: 'Status',
			running: 'Running',
			stopped: 'Stopped',
			printers: 'Printers',
			noPrinters: 'No printers configured',
			noPrintersHint: 'Connect a USB printer and click "Scan" to detect it',
			scan: 'Scan for printers',
			scanning: 'Scanning...',
			addPrinter: 'Add Printer',
			removePrinter: 'Remove',
			printerName: 'Printer Name',
			printerUri: 'Connection',
			selectDriver: 'Select Driver',
			loadingDrivers: 'Loading drivers...',
			location: 'Location',
			shared: 'Shared',
			notShared: 'Not shared',
			setDefault: 'Set as default',
			default: 'Default',
			testPage: 'Test Page',
			testPageSent: 'Test page sent',
			protocols: 'Sharing Protocols',
			protocolsHint: 'Printers are shared via IPP (macOS/Linux/iOS) and discoverable via AirPrint. Windows clients can connect via SMB if Samba is enabled.',
			queue: 'Print Queue',
			noJobs: 'No print jobs',
			cancelJob: 'Cancel',
			state: {
				idle: 'Idle',
				processing: 'Printing',
				stopped: 'Stopped',
				error: 'Error'
			},
			detected: 'Detected Printers',
			noDetected: 'No USB printers detected',
			connectUsb: 'Connect a USB printer to your device',
			add: 'Add',
			adding: 'Adding...',
			confirmRemove: 'Remove this printer?',
			confirmRemoveMessage: 'The printer will no longer be shared on the network.'
		}
	},

	// Share Manager
	shareManager: {
		title: 'Shared Folders',
		createShare: 'Create Share',
		editShare: 'Edit Share',
		deleteShare: 'Delete Share',
		permissions: 'Permissions',
		fields: {
			name: 'Share Name',
			namePlaceholder: 'Enter share name',
			path: 'Path',
			protocol: 'Protocol',
			enabled: 'Enabled',
			users: 'Users',
			description: 'Description',
			descriptionPlaceholder: 'Optional description',
			guestAccess: 'Allow Guest Access',
			browseable: 'Browseable',
			readOnly: 'Read Only',
			advanced: 'Advanced Options'
		},
		protocols: {
			smb: 'SMB/CIFS',
			nfs: 'NFS',
			ftp: 'FTP'
		},
		messages: {
			noShares: 'No shared folders configured',
			noSharesHint: 'Create a shared folder to start sharing files on your network',
			deleteConfirm: 'Are you sure you want to delete this share?',
			deleteHint: 'The shared folder will be removed but files will be preserved.',
			shareCreated: 'Share created successfully',
			shareUpdated: 'Share updated successfully',
			shareDeleted: 'Share deleted successfully',
			sambaNotRunning: 'Samba service is not running. Enable it in File Service settings.',
			loading: 'Loading shares...',
			error: 'Failed to load shares'
		}
	},

	// Folder Picker
	folderPicker: {
		title: 'Select Folder',
		browse: 'Browse',
		select: 'Select',
		selected: 'Selected',
		location: 'Location',
		noSubfolders: 'No subfolders',
		emptyFolder: 'This folder is empty',
		noLocations: 'No locations available',
		loading: 'Loading...',
		error: 'Failed to load folders'
	},

	// Docker App
	docker: {
		search: 'Search...',
		noResults: 'No results',
		serviceStatus: 'Service running',
		status: {
			normal: 'Normal',
			stopped: 'Stopped',
			error: 'Error'
		},
		stats: {
			containers: 'Containers',
			images: 'Images',
			volumes: 'Volumes',
			networks: 'Networks'
		},
		cpuUsage: 'CPU usage',
		memoryCapacity: 'Memory capacity',
		available: 'Available',
		views: {
			overview: 'Overview',
			container: 'Containers',
			image: 'Images',
			volume: 'Volumes',
			network: 'Networks'
		},
		table: {
			name: 'Name',
			image: 'Image',
			status: 'Status',
			ports: 'Ports',
			actions: 'Actions',
			repository: 'Repository',
			tag: 'Tag',
			imageId: 'Image ID',
			id: 'ID',
			size: 'Size',
			created: 'Created',
			driver: 'Driver',
			mountPoint: 'Mount Point',
			scope: 'Scope'
		},
		noContainers: 'No containers found',
		noImages: 'No images found',
		confirm: {
			stopTitle: 'Stop Container',
			stopMessage: 'Are you sure you want to stop the container "{name}"?',
			stopBtn: 'Stop',
			startTitle: 'Start Container',
			startMessage: 'Are you sure you want to start the container "{name}"?',
			startBtn: 'Start',
			restartTitle: 'Restart Container',
			restartMessage: 'Are you sure you want to restart the container "{name}"?',
			restartBtn: 'Restart',
			removeContainerTitle: 'Remove Container',
			removeContainerMessage: 'Are you sure you want to remove the container "{name}"? This action cannot be undone.',
			removeImageTitle: 'Remove Image',
			removeImageMessage: 'Are you sure you want to remove the image "{name}"? This action cannot be undone.',
			removeVolumeTitle: 'Remove Volume',
			removeVolumeMessage: 'Are you sure you want to remove the volume "{name}"? All data will be lost.',
			removeNetworkTitle: 'Remove Network',
			removeNetworkMessage: 'Are you sure you want to remove the network "{name}"?',
			removeBtn: 'Remove',
			pruneImagesTitle: 'Prune Images',
			pruneImagesMessage: 'This will remove all unused images. This action cannot be undone.',
			pruneImagesBtn: 'Prune',
			pruneVolumesTitle: 'Prune Volumes',
			pruneVolumesMessage: 'This will remove all unused volumes. All data in these volumes will be lost.',
			pruneVolumesBtn: 'Prune'
		},
		logs: {
			title: 'Logs',
			noLogs: 'No logs available',
			refresh: 'Refresh',
			lines: 'lines'
		},
		volumes: {
			title: 'Volumes',
			noVolumes: 'No volumes found'
		},
		networks: {
			title: 'Networks',
			noNetworks: 'No networks found',
			builtIn: 'built-in'
		},
		pull: {
			placeholder: 'Image name (e.g. nginx:latest)',
			button: 'Pull',
			pulling: 'Pulling...'
		}
	},

	// System Update
	systemUpdate: {
		title: 'System Update',
		currentVersion: 'Current version',
		checkForUpdates: 'Check for updates',
		checking: 'Checking...',
		upToDate: 'Your system is up to date',
		upToDateDesc: 'You are running the latest version of PiNAS.',
		updateAvailable: 'Update available',
		downloadSize: 'Download size',
		noRebootRequired: 'No reboot required',
		rebootRequired: 'Reboot required after update',
		installUpdate: 'Install update',
		installing: 'Installing update...',
		downloading: 'Downloading update...',
		applying: 'Applying update...',
		restarting: 'Restarting service...',
		updateHistory: 'Update History',
		noHistory: 'No update history yet.',
		failedToCheck: 'Failed to check for updates',
		failedToInstall: 'Failed to install update',
		modal: {
			title: 'Update Successful',
			subtitle: 'PiNAS has been updated successfully',
			updatedTo: 'Updated to version',
			changelog: 'What\'s new',
			dismiss: 'Got it!',
			thanks: 'Thank you for using PiNAS.'
		},
		screen: {
			starting: 'The update is about to start...',
			doNotTurnOff: 'Please do not turn off your device',
			inProgress: 'Update in progress',
			completed: 'Update completed successfully!',
			installed: 'PiNAS {version} installed',
			failed: 'Update failed',
			reboot: 'Reboot',
			reloadDesktop: 'Reload desktop',
			restarting: 'Restarting service...',
			close: 'Close',
			confirm: {
				title: 'Install update v{version}?',
				description: 'This will update your system. Do not unplug the device.',
				cancel: 'Cancel',
				confirm: 'Confirm'
			},
			devTest: 'Test update screen'
		}
	}
};
