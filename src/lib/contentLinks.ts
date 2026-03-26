import { openInBrowser } from '$lib/api';

const SAFE_OPEN_SCHEMES = new Set(['http:', 'https:', 'mailto:', 'tel:']);

/**
 * Intercepts activation on `<a href>` inside `{@html ...}` so the Tauri webview never navigates
 * (left-click and middle-click). Safe schemes open in the system browser; other schemes are blocked.
 */
export function onInjectedHtmlLinkActivate(e: MouseEvent, baseUrlForRelativeHrefs: string): void {
	if (e.type === 'click' && e.button !== 0) return;
	if (e.type === 'auxclick' && e.button !== 1) return;

	const t = e.target;
	if (!(t instanceof Element)) return;
	const a = t.closest('a[href]');
	if (!a || !(a instanceof HTMLAnchorElement)) return;

	const hrefAttr = a.getAttribute('href');
	if (hrefAttr == null) return;

	const trimmed = hrefAttr.trim();
	if (trimmed === '' || trimmed.startsWith('#')) {
		e.preventDefault();
		return;
	}

	if (/^javascript:/i.test(trimmed)) {
		e.preventDefault();
		return;
	}

	let resolved: URL;
	try {
		resolved = new URL(trimmed, baseUrlForRelativeHrefs);
	} catch {
		e.preventDefault();
		return;
	}

	const scheme = resolved.protocol.toLowerCase();
	if (SAFE_OPEN_SCHEMES.has(scheme)) {
		e.preventDefault();
		void openInBrowser(resolved.href);
		return;
	}

	e.preventDefault();
}
