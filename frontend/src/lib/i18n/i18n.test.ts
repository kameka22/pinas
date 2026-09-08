import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import en from './en';
import fr from './fr';
import { locale, t, translate } from './index';

function flatten(obj: Record<string, unknown>, prefix = ''): Record<string, string> {
	const out: Record<string, string> = {};
	for (const [k, v] of Object.entries(obj)) {
		const key = prefix ? `${prefix}.${k}` : k;
		if (v && typeof v === 'object') Object.assign(out, flatten(v as Record<string, unknown>, key));
		else out[key] = String(v);
	}
	return out;
}

describe('translations', () => {
	const enKeys = flatten(en as unknown as Record<string, unknown>);
	const frKeys = flatten(fr as unknown as Record<string, unknown>);

	it('en and fr expose exactly the same keys', () => {
		const onlyEn = Object.keys(enKeys).filter((k) => !(k in frKeys));
		const onlyFr = Object.keys(frKeys).filter((k) => !(k in enKeys));
		expect(onlyEn).toEqual([]);
		expect(onlyFr).toEqual([]);
	});

	it('has no empty translations', () => {
		const empty = [...Object.entries(enKeys), ...Object.entries(frKeys)].filter(([, v]) => v.trim() === '');
		expect(empty).toEqual([]);
	});

	it('placeholders are preserved between languages', () => {
		for (const [k, v] of Object.entries(enKeys)) {
			const ph = (v.match(/\{[a-zA-Z_]+\}/g) || []).sort();
			const phFr = (frKeys[k].match(/\{[a-zA-Z_]+\}/g) || []).sort();
			expect(phFr, k).toEqual(ph);
		}
	});

	it('switching locale changes the derived dictionary', () => {
		locale.setLocale('fr');
		expect(get(t).common.save).toBe('Enregistrer');
		locale.setLocale('en');
		expect(get(t).common.save).toBe('Save');
	});

	it('translate() resolves dotted paths and falls back to the path', () => {
		expect(translate(en, 'common.errors.forbidden')).toBe(en.common.errors.forbidden);
		expect(translate(en, 'does.not.exist')).toBe('does.not.exist');
	});
});
