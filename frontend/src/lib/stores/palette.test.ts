import { describe, it, expect } from 'vitest';
import { scoreMatch } from './palette';

describe('scoreMatch', () => {
	it('ranks exact > prefix > word start > substring', () => {
		const exact = scoreMatch('docker', 'Docker');
		const prefix = scoreMatch('doc', 'Docker Manager');
		const word = scoreMatch('man', 'Docker Manager');
		const sub = scoreMatch('ker', 'Docker Manager');
		expect(exact).toBeGreaterThan(prefix);
		expect(prefix).toBeGreaterThan(word);
		expect(word).toBeGreaterThan(sub);
		expect(sub).toBeGreaterThan(0);
	});

	it('is case-insensitive and rejects non-matches / empty queries', () => {
		expect(scoreMatch('FILE', 'file manager')).toBeGreaterThan(0);
		expect(scoreMatch('zzz', 'Docker')).toBe(0);
		expect(scoreMatch('   ', 'Docker')).toBe(1); // empty query lists everything
	});
});
