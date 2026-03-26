import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	// Keep port stable so `tauri dev` (devUrl in tauri.conf.json) always hits this dev server, not a stale `build/`.
	server: {
		port: 5173,
		strictPort: true,
		// WKWebView can cache localhost aggressively; stale bundles looked like “old menu + no proxy UI”.
		headers: {
			'Cache-Control': 'no-store'
		}
	}
});
