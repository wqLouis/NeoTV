import { paraglideVitePlugin } from '@inlang/paraglide-js';
import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { join } from 'path';
import { execSync } from 'child_process';

function lrudSpatialBuild(): { name: string; buildStart: () => void } {
	let built = false;
	return {
		name: 'lrud-spatial-build',
		buildStart() {
			if (built) return;
			built = true;
			const pkgPath = join(process.cwd(), 'node_modules/@bbc/tv-lrud-spatial');
			const lrudMinPath = join(pkgPath, 'lib/lrud.min.js');
			if (!require('fs').existsSync(lrudMinPath)) {
				console.log('[lrud-spatial] Building minified bundle...');
				execSync('bun run build', { cwd: pkgPath, stdio: 'pipe' });
			}
		}
	};
}

export default defineConfig({
	plugins: [
		lrudSpatialBuild(),
		tailwindcss(),
		sveltekit(),
		paraglideVitePlugin({ project: './project.inlang', outdir: './src/lib/paraglide' })
	]
});
