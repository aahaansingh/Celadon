<script lang="ts">
	import type { Article } from '$lib/api';
	import { getArticleProxyUrl, hasTauriIpc, openInBrowser } from '$lib/api';
	import { decodeHtmlEntities, sanitizeHtml } from '$lib/sanitizeHtml';
	import { X, ExternalLink, Settings } from 'lucide-svelte';
	import { fade } from 'svelte/transition';

	let { article, onClose, articleFullModeProxy, onOpenAppSettings } = $props<{
		article: Article | null;
		onClose: () => void;
		articleFullModeProxy: boolean;
		onOpenAppSettings: () => void;
	}>();

	let mode = $state<'simple' | 'full'>('simple');
	let simpleContent = $derived(article?.description ? sanitizeHtml(article.description) : '');
	let fullModeIframeSrc = $state('');
	/** When set, iframe uses `srcdoc` (avoids navigating the frame to `celadon://`, which WebKit often blocks). */
	let fullModeIframeSrcdoc = $state('');

	function escAttr(s: string): string {
		return s.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;');
	}

	/** Ensure relative URLs resolve when using `srcdoc` (rewriter already absolutizes most assets to `celadon://`). */
	function ensureBaseHref(html: string, documentUrl: string): string {
		if (/<base[\s>/]/i.test(html)) return html;
		let href: string;
		try {
			const u = new URL(documentUrl);
			u.hash = '';
			href = u.toString();
		} catch {
			return html;
		}
		const tag = `<base href="${escAttr(href)}">`;
		const headMatch = html.match(/<head([^>]*)>/i);
		if (headMatch && headMatch.index !== undefined) {
			const idx = headMatch.index + headMatch[0].length;
			return html.slice(0, idx) + tag + html.slice(idx);
		}
		return `<!DOCTYPE html><html><head>${tag}</head><body>${html}</body></html>`;
	}

	$effect(() => {
		const a = article;
		const full = mode === 'full';
		if (!a || !full) {
			fullModeIframeSrc = '';
			fullModeIframeSrcdoc = '';
			return;
		}
		const docUrl = a.url;
		const useProxy = articleFullModeProxy && hasTauriIpc();
		if (!useProxy) {
			fullModeIframeSrcdoc = '';
			fullModeIframeSrc = docUrl;
			return;
		}
		fullModeIframeSrc = '';
		fullModeIframeSrcdoc = '';
		let cancelled = false;
		(async () => {
			try {
				const proxyUrl = await getArticleProxyUrl(docUrl);
				const res = await fetch(proxyUrl);
				if (!res.ok) throw new Error(`proxy HTTP ${res.status}`);
				const html = await res.text();
				if (!cancelled) fullModeIframeSrcdoc = ensureBaseHref(html, docUrl);
			} catch {
				fullModeIframeSrcdoc = '';
				if (!cancelled) fullModeIframeSrc = docUrl;
			}
		})();
		return () => {
			cancelled = true;
		};
	});
</script>

{#if article}
	<div
		transition:fade={{ duration: 200 }}
		class="fixed inset-0 z-[100] bg-background/95 backdrop-blur-xl flex flex-col font-body"
	>
		<!-- Header -->
		<header class="px-6 py-4 border-b border-border flex items-center justify-between gap-3">
			<div class="flex items-center gap-4 min-w-0">
				<button onclick={onClose} class="p-2 hover:bg-muted rounded-full transition-colors shrink-0">
					<X class="w-5 h-5" />
				</button>
				<h1 class="font-heading font-bold text-lg truncate max-w-xl min-w-0">
					{decodeHtmlEntities(article.name)}
				</h1>
			</div>

			<div class="flex items-center gap-2 shrink-0">
				<button
					type="button"
					onclick={onOpenAppSettings}
					class="p-2 hover:bg-muted rounded-xl transition-all text-muted-foreground hover:text-foreground"
					title="App settings"
					aria-label="App settings"
				>
					<Settings class="w-4 h-4" />
				</button>
				<button
					onclick={() => (mode = mode === 'simple' ? 'full' : 'simple')}
					class="px-4 py-2 bg-muted hover:bg-muted/80 rounded-xl text-xs font-heading font-bold transition-all"
				>
					{mode === 'simple' ? 'Full Mode' : 'Simple Mode'}
				</button>

				<button
					onclick={() => openInBrowser(article.url)}
					class="p-2 hover:bg-muted rounded-xl transition-all"
					title="Open in Browser"
				>
					<ExternalLink class="w-4 h-4" />
				</button>
			</div>
		</header>

		<!-- Content Area -->
		<div class="flex-1 overflow-auto">
			{#if mode === 'simple'}
				<article class="max-w-3xl mx-auto px-6 py-12 prose dark:prose-invert prose-p:text-lg prose-p:leading-relaxed prose-p:text-foreground/80">
					{#if simpleContent}
						{@html simpleContent}
					{:else}
						<p class="text-lg leading-relaxed text-foreground/80">No summary available.</p>
					{/if}
				</article>
			{:else}
				<div class="w-full h-full flex flex-col bg-white">
					{#if articleFullModeProxy && hasTauriIpc() && fullModeIframeSrc === '' && fullModeIframeSrcdoc === ''}
						<div class="flex-1 flex items-center justify-center text-muted-foreground text-sm font-body">
							Loading article…
						</div>
					{:else if fullModeIframeSrcdoc}
						<iframe
							title={decodeHtmlEntities(article.name)}
							srcdoc={fullModeIframeSrcdoc}
							class="w-full flex-1 min-h-0 border-none"
						></iframe>
					{:else}
						<iframe
							src={fullModeIframeSrc}
							title={decodeHtmlEntities(article.name)}
							class="w-full flex-1 min-h-0 border-none"
						></iframe>
					{/if}
					<div
						class="px-4 py-2 bg-muted/50 border-t border-border text-center text-xs font-body text-[#5a7060]"
					>
						If the page doesn&apos;t appear, this site may block embedding.
						<button
							type="button"
							onclick={() => openInBrowser(article.url)}
							class="ml-2 underline text-inherit hover:opacity-80"
						>
							Open in browser
						</button>
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	/* Styling for the simple view prose */
	article {
		font-family: var(--font-body);
	}
</style>
