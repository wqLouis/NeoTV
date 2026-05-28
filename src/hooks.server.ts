import type { Handle } from '@sveltejs/kit';
import { getCookie } from 'intlayer';
import { Locales } from 'intlayer';

export const handle: Handle = async ({ event, resolve }) => {
	const lang = getCookie(event.request.headers.get('cookie') ?? '', 'intlayer_locale') ?? Locales.ENGLISH;

	return resolve(event, {
		transformPageChunk: ({ html }) => html.replace('%lang%', lang)
	});
};
