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
		searchApplications: 'Search applications...'
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
	}
};
