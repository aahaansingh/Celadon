/**
 * Sanitize HTML for safe display (e.g. RSS descriptions).
 * Allows a minimal allowlist of tags and strips the rest (including script, iframe, etc.).
 */
const ALLOWED_TAGS = new Set([
	'p', 'br', 'a', 'span', 'div', 'em', 'strong', 'b', 'i', 'u', 'code', 'pre',
	'ul', 'ol', 'li', 'blockquote', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'img'
]);
const ALLOWED_ATTRS: Record<string, string[]> = {
	a: ['href', 'title'],
	img: ['src', 'alt', 'title']
};

export function sanitizeHtml(html: string): string {
	if (!html || typeof html !== 'string') return '';
	const doc = typeof document !== 'undefined' ? document.createElement('div') : null;
	if (!doc) return html.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '');
	doc.innerHTML = html;
	const walk = (node: Node): void => {
		if (node.nodeType === Node.ELEMENT_NODE) {
			const el = node as Element;
			const tag = el.tagName.toLowerCase();
			if (!ALLOWED_TAGS.has(tag)) {
				const parent = el.parentNode;
				while (el.firstChild) parent?.insertBefore(el.firstChild, el);
				parent?.removeChild(el);
				return;
			}
			const allowed = ALLOWED_ATTRS[tag];
			if (allowed) {
				for (const a of Array.from(el.attributes)) {
					if (!allowed.includes(a.name.toLowerCase())) el.removeAttribute(a.name);
				}
			} else {
				for (const a of Array.from(el.attributes)) {
					if (a.name.toLowerCase().startsWith('on')) el.removeAttribute(a.name);
				}
			}
		}
		for (const child of Array.from(node.childNodes)) walk(child);
	};
	walk(doc);
	return doc.innerHTML;
}

/** Decode HTML entities in plain text (e.g. &#8220; → "). */
export function decodeHtmlEntities(text: string): string {
	if (!text || typeof text !== 'string') return '';
	const doc = typeof document !== 'undefined' ? document.createElement('div') : null;
	if (doc) {
		doc.innerHTML = text;
		return doc.textContent || doc.innerText || text;
	}
	return text.replace(/&#(\d+);/g, (_, code) => String.fromCharCode(parseInt(code, 10)))
		.replace(/&quot;/g, '"')
		.replace(/&apos;/g, "'")
		.replace(/&lt;/g, '<')
		.replace(/&gt;/g, '>')
		.replace(/&amp;/g, '&');
}
