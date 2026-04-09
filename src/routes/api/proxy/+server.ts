import type { RequestHandler } from './$types';

const FORBIDDEN_HOSTS = ['localhost', '127.0.0.1', '0.0.0.0', '[::1]'];
const DOUBAN_IMAGE_HOSTS = [
	'img1.doubanio.com',
	'img2.doubanio.com',
	'img3.doubanio.com',
	'img9.doubanio.com'
];

function isDoubanImageHost(hostname: string): boolean {
	return DOUBAN_IMAGE_HOSTS.some((h) => hostname.includes(h));
}

export const GET: RequestHandler = async ({ url }) => {
	const targetUrl = url.searchParams.get('url');

	if (!targetUrl) {
		return new Response(JSON.stringify({ error: 'Missing url parameter' }), {
			status: 400,
			headers: { 'Content-Type': 'application/json' }
		});
	}

	let parsedUrl: URL;
	try {
		parsedUrl = new URL(targetUrl);
	} catch {
		return new Response(JSON.stringify({ error: 'Invalid URL' }), {
			status: 400,
			headers: { 'Content-Type': 'application/json' }
		});
	}

	if (!['http:', 'https:'].includes(parsedUrl.protocol)) {
		return new Response(JSON.stringify({ error: 'Invalid protocol' }), {
			status: 400,
			headers: { 'Content-Type': 'application/json' }
		});
	}

	if (FORBIDDEN_HOSTS.some((host) => parsedUrl.hostname.includes(host))) {
		return new Response(JSON.stringify({ error: 'Access denied' }), {
			status: 403,
			headers: { 'Content-Type': 'application/json' }
		});
	}

	const isDoubanImage = isDoubanImageHost(parsedUrl.hostname);

	try {
		const headers: Record<string, string> = {
			'User-Agent':
				'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36',
			Accept: 'image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8',
			'Accept-Language': 'zh-CN,zh;q=0.9,en;q=0.8',
			'Accept-Encoding': 'gzip, deflate, br',
			DNT: '1',
			'Sec-Fetch-Dest': 'image',
			'Sec-Fetch-Mode': 'no-cors',
			'Sec-Fetch-Site': 'cross-site',
			'Sec-Fetch-Storage-Access': 'none'
		};

		if (isDoubanImage) {
			headers['Referer'] = 'https://movie.douban.com/';
			headers['Origin'] = 'https://movie.douban.com';
		}

		const response = await fetch(targetUrl, {
			headers,
			signal: AbortSignal.timeout(20000)
		});

		const contentType = response.headers.get('content-type') || '';

		if (contentType.startsWith('image/') || isDoubanImage) {
			const buffer = await response.arrayBuffer();
			return new Response(buffer, {
				status: 200,
				headers: {
					'Content-Type': isDoubanImage ? 'image/jpeg' : contentType || 'image/jpeg',
					'Cache-Control': 'public, max-age=86400',
					'Access-Control-Allow-Origin': '*',
					'Access-Control-Expose-Headers': 'Content-Type, Content-Length, Cache-Control'
				}
			});
		}

		const body = await response.text();

		if (
			contentType.includes('application/json') ||
			contentType.includes('application/javascript')
		) {
			try {
				const json = JSON.parse(body);
				return new Response(JSON.stringify(json), {
					status: response.status,
					headers: {
						'Content-Type': 'application/json',
						'Access-Control-Allow-Origin': '*'
					}
				});
			} catch {
				return new Response(body, {
					status: 502,
					headers: {
						'Content-Type': 'text/plain',
						'Access-Control-Allow-Origin': '*'
					}
				});
			}
		}

		return new Response(body, {
			status: response.status,
			headers: {
				'Content-Type': contentType || 'text/plain',
				'Access-Control-Allow-Origin': '*'
			}
		});
	} catch (error) {
		const message = error instanceof Error ? error.message : 'Proxy request failed';
		return new Response(JSON.stringify({ error: message }), {
			status: 502,
			headers: { 'Content-Type': 'application/json' }
		});
	}
};
