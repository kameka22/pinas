import { expect, it } from 'vitest';
import { validHostname, validSetupUsername } from './setupValidation.js';

it('refuses invalid hostnames before submitting setup', () => {
	for (const name of [
		'',
		'-bad',
		'bad-',
		'a.b',
		'two words',
		'abc\n',
		'é',
		'x'.repeat(64),
	]) {
		expect(validHostname(name), name).toBe(false);
	}
	for (const name of ['x', 'PiNAS-5', 'x'.repeat(63)])
		expect(validHostname(name), name).toBe(true);
});
it('refuses system accounts, unsafe names and numeric usernames', () => {
	for (const name of [
		'root',
		'ROOT',
		'Kodi',
		'pinas',
		'-bad',
		'.bad',
		'123',
		'a/b',
		'abc\n',
		'ab',
		'x'.repeat(33),
	]) {
		expect(validSetupUsername(name), name).toBe(false);
	}
	for (const name of ['alice', 'alice-2', 'a.b', '_admin'])
		expect(validSetupUsername(name), name).toBe(true);
});
