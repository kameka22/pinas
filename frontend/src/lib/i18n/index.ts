import { writable, derived, get } from 'svelte/store';
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

const baseTranslations: Record<Locale, typeof en> = {
	en,
	fr
};

// Store for dynamically loaded app translations
// Structure: { appId: { locale: translations } }
export const appTranslations = writable<Record<string, Record<string, any>>>({});

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

// Derived store for translations (base + app translations merged)
export const t = derived(
	[locale, appTranslations],
	([$locale, $appTranslations]) => {
		// Start with base translations
		const merged = { ...baseTranslations[$locale] } as any;

		// Merge in app-specific translations
		for (const [appId, locales] of Object.entries($appTranslations)) {
			if (locales[$locale]) {
				merged[appId] = locales[$locale];
			} else if (locales['en']) {
				// Fallback to English if locale not available
				merged[appId] = locales['en'];
			}
		}

		return merged;
	}
);

// Load translations for an app from the backend
export async function loadAppTranslations(appId: string): Promise<void> {
	const currentLocale = get(locale);

	try {
		const response = await fetch(`/api/apps/${appId}/i18n/${currentLocale}`);
		if (!response.ok) {
			console.warn(`Failed to load translations for ${appId}:`, response.statusText);
			return;
		}

		const translations = await response.json();

		if (Object.keys(translations).length > 0) {
			appTranslations.update((current) => ({
				...current,
				[appId]: {
					...current[appId],
					[currentLocale]: translations
				}
			}));
		}
	} catch (error) {
		console.warn(`Failed to load translations for ${appId}:`, error);
	}
}

// Load translations for all installed apps
export async function loadAllAppTranslations(appIds: string[]): Promise<void> {
	await Promise.all(appIds.map((id) => loadAppTranslations(id)));
}

// Clear translations for an app (e.g., when uninstalled)
export function clearAppTranslations(appId: string): void {
	appTranslations.update((current) => {
		const { [appId]: _, ...rest } = current;
		return rest;
	});
}

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
