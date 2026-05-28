import {
	getLocaleFromStorageClient,
	setLocaleInStorageClient,
	type Locale
} from 'intlayer';

/**
 * Get the current locale from storage (localStorage on client).
 * Falls back to 'en' if not found.
 */
export function getIntlayerLocale(): Locale {
	if (typeof window === 'undefined') {
		return 'en';
	}
	return getLocaleFromStorageClient() ?? 'en';
}

/**
 * Set the current locale and persist it to storage + cookie.
 */
export function setIntlayerLocale(locale: Locale): void {
	setLocaleInStorageClient(locale);
	document.cookie = `intlayer_locale=${locale}; path=/; max-age=${60 * 60 * 24 * 365}; SameSite=Lax`;
}
