// Keep aligned with services/user.rs and services/network.rs.
const reservedUsernames = [
	'root',
	'kodi',
	'nobody',
	'daemon',
	'bin',
	'sys',
	'sync',
	'games',
	'man',
	'lp',
	'mail',
	'news',
	'uucp',
	'proxy',
	'www-data',
	'backup',
	'list',
	'irc',
	'gnats',
	'sshd',
	'dbus',
	'messagebus',
	'avahi',
	'systemd-network',
	'systemd-resolve',
	'systemd-timesync',
	'pinas',
	'guest',
	'smbguest',
	'nfsnobody',
];

export function validHostname(name: string): boolean {
	return (
		name.length >= 1 &&
		name.length <= 63 &&
		!/[^a-zA-Z0-9-]/.test(name) &&
		!name.startsWith('-') &&
		!name.endsWith('-')
	);
}

export function validSetupUsername(name: string): boolean {
	return (
		name.length >= 3 &&
		name.length <= 32 &&
		!/[^a-zA-Z0-9_.-]/.test(name) &&
		!name.startsWith('-') &&
		!name.startsWith('.') &&
		!/^\d+$/.test(name) &&
		!reservedUsernames.includes(name.toLowerCase())
	);
}
