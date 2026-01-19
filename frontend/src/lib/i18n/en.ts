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
		noApplicationsFound: 'No applications found',
		searchApplications: 'Search applications...',
		refresh: 'Refresh',
		retry: 'Retry',
		copy: 'Copy'
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
			hardwarePower: 'Hardware & Power',
			timeLanguage: 'Time & Language',
			network: 'Network',
			security: 'Security',
			indexingService: 'Indexing Service',
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
			sharedFolder: 'Shared folder',
			noAccess: 'No Access',
			readOnly: 'Read Only',
			readWrite: 'Read/Write'
		},
		messages: {
			deleteConfirm: 'Are you sure you want to delete user "{username}"?',
			deleteGroupConfirm: 'Are you sure you want to delete group "{groupName}"?',
			cannotBeUndone: 'This action cannot be undone.',
			userDataShown: 'User data currently shown',
			me: 'Me'
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
				system: 'System',
				storageFiles: 'Storage & Files',
				services: 'Services'
			}
		}
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
			logout: 'Logout'
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
			language: 'Language',
			device: 'Device',
			account: 'Account',
			password: 'Password'
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
		buttons: {
			back: 'Back',
			next: 'Next',
			complete: 'Complete Setup'
		},
		validation: {
			machineNameRequired: 'Machine name is required',
			machineNameMinLength: 'Machine name must be at least 2 characters',
			machineNameInvalid: 'Only letters, numbers and hyphens allowed',
			usernameRequired: 'Username is required',
			usernameMinLength: 'Username must be at least 3 characters',
			usernameInvalid: 'Only letters, numbers and underscores allowed',
			passwordRequired: 'Password is required',
			passwordMinLength: 'Password must be at least 6 characters',
			passwordMismatch: 'Passwords do not match'
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
		toolbar: {
			refresh: 'Refresh',
			search: 'Search...',
			newFolder: 'New folder',
			upload: 'Upload',
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
			download: 'Download',
			rename: 'Rename',
			properties: 'Properties'
		},
		statusBar: {
			item: 'item in total',
			items: 'items in total',
			selected: 'selected'
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
			open: 'Open',
			update: 'Update'
		},
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

	// Docker App
	docker: {
		serviceStatus: 'Service running',
		status: {
			normal: 'Normal',
			stopped: 'Stopped',
			error: 'Error'
		},
		stats: {
			projects: 'Projects',
			containers: 'Containers',
			local: 'Local',
			data: 'Data'
		},
		cpuUsage: 'CPU usage',
		memoryCapacity: 'Memory capacity',
		available: 'Available',
		views: {
			overview: 'Overview',
			project: 'Project',
			container: 'Container',
			image: 'Image',
			network: 'Network',
			log: 'Log',
			management: 'Management'
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
			size: 'Size',
			created: 'Created'
		},
		noContainers: 'No containers found',
		noImages: 'No images found',
		underDevelopment: 'This section is under development'
	}
};
