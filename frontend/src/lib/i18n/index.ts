import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';
import en from './en';
import fr from './fr';

export type Locale = 'en' | 'fr';

export interface Language {
	code: Locale;
	name: string;
	nativeName: string;
	flag: string; // Emoji flag
}

export const languages: Language[] = [
	{ code: 'en', name: 'English', nativeName: 'English', flag: '🇬🇧' },
	{ code: 'fr', name: 'French', nativeName: 'Français', flag: '🇫🇷' }
];

const translations: Record<Locale, typeof en> = {
	en,
	fr
};

const STORAGE_KEY = 'pinas-locale';

function getInitialLocale(): Locale {
	if (!browser) return 'en';

	// Check localStorage
	const stored = localStorage.getItem(STORAGE_KEY);
	if (stored && (stored === 'en' || stored === 'fr')) {
		return stored;
	}

	// Check browser language
	const browserLang = navigator.language.split('-')[0];
	if (browserLang === 'fr') {
		return 'fr';
	}

	return 'en';
}

function createI18nStore() {
	const { subscribe, set, update } = writable<Locale>(getInitialLocale());

	return {
		subscribe,

		setLocale: (locale: Locale) => {
			if (browser) {
				localStorage.setItem(STORAGE_KEY, locale);
			}
			set(locale);
		},

		init: () => {
			set(getInitialLocale());
		}
	};
}

export const locale = createI18nStore();

// Derived store for translations
export const t = derived(locale, ($locale) => translations[$locale]);

// Helper function to get nested translation with fallback
export function translate(obj: any, path: string): string {
	const keys = path.split('.');
	let result = obj;

	for (const key of keys) {
		if (result && typeof result === 'object' && key in result) {
			result = result[key];
		} else {
			return path; // Return path as fallback
		}
	}

	return typeof result === 'string' ? result : path;
}

// Re-export for convenience
export { en, fr };
