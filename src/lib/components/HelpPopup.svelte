<script lang="ts">
	import { onMount } from 'svelte';
	import { HelpCircle, X } from 'lucide-svelte';
	import { marked } from 'marked';

	let open = $state(false);
	let content = $state('');
	let error = $state<string | null>(null);

	let popupEl = $state<HTMLDivElement>();

	// Prefetch so opening the popup is instant (avoids MIME/import issues and latency)
	let cachedHtml = $state<string | null>(null);
	onMount(() => {
		fetch('/commands.md')
			.then((res) => (res.ok ? res.text() : Promise.reject(new Error(res.statusText))))
			.then((raw) => marked.parse(raw))
			.then((html) => { cachedHtml = typeof html === 'string' ? html : ''; })
			.catch(() => {});
	});

	function handleEscape(e: KeyboardEvent) {
		if (e.key === 'Escape') close();
	}

	$effect(() => {
		if (open) {
			window.addEventListener('keydown', handleEscape);
			return () => window.removeEventListener('keydown', handleEscape);
		}
	});

	function loadAndShow() {
		open = true;
		error = null;
		content = cachedHtml ?? '';
		if (cachedHtml !== null) return;
		fetch('/commands.md')
			.then((res) => (res.ok ? res.text() : Promise.reject(new Error(res.statusText))))
			.then((raw) => marked.parse(raw))
			.then((html) => {
				content = typeof html === 'string' ? html : '';
				cachedHtml = content;
			})
			.catch((e) => {
				error = e instanceof Error ? e.message : String(e);
			});
	}

	function close() {
		open = false;
	}
</script>

<button
	type="button"
	onclick={loadAndShow}
	class="fixed bottom-6 right-6 z-40 flex h-12 w-12 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-lg hover:bg-primary/90 transition-colors"
	aria-label="Show commands help"
>
	<HelpCircle class="h-6 w-6" />
</button>

{#if open}
	<!-- Backdrop: click outside to close -->
	<button
		type="button"
		class="fixed inset-0 z-[200] bg-black/40 backdrop-blur-sm"
		aria-label="Close"
		onclick={close}
	></button>

	<!-- Popup panel: light = bg-celadon; dark = translucent via .help-panel -->
	<div
		bind:this={popupEl}
		class="help-panel fixed left-1/2 top-1/2 z-[201] w-[min(90vw,42rem)] max-h-[80vh] -translate-x-1/2 -translate-y-1/2 flex flex-col rounded-xl border border-border bg-celadon shadow-2xl"
		role="dialog"
		aria-modal="true"
		aria-labelledby="help-title"
	>
		<div class="flex items-center shrink-0 border-b border-border px-4 py-3">
			<button
				type="button"
				onclick={close}
				class="p-1.5 -ml-1 rounded-lg hover:bg-muted transition-colors shrink-0"
				aria-label="Close"
			>
				<X class="h-5 w-5" />
			</button>
			<h2 id="help-title" class="font-heading font-bold text-sm flex-1 text-center">Commands</h2>
		</div>
		<div class="help-content min-h-0 flex-1 overflow-y-auto p-4">
			{#if error}
				<p class="text-red-600 dark:text-red-400">{error}</p>
			{:else if content}
				{@html content}
			{/if}
		</div>
	</div>
{/if}

<style>
	/* Dark mode only: translucent panel (light mode unchanged) */
	:global(.dark) .help-panel {
		background-color: color-mix(in srgb, var(--background) 88%, transparent);
		backdrop-filter: blur(20px);
	}

	.help-content :global(h1) {
		font-family: var(--font-heading);
		font-weight: 700;
		font-size: 1.125rem;
		line-height: 1.75rem;
		margin: 0 0 0.75rem 0;
	}
	.help-content :global(h2) {
		font-family: var(--font-heading);
		font-weight: 700;
		font-size: 1rem;
		line-height: 1.5rem;
		margin: 1rem 0 0.5rem 0;
		border-bottom: 1px solid var(--border);
		padding-bottom: 0.25rem;
	}
	.help-content :global(h3) {
		font-family: var(--font-heading);
		font-weight: 600;
		font-size: 0.875rem;
		line-height: 1.25rem;
		margin: 0.75rem 0 0.25rem 0;
	}
	.help-content :global(p) {
		font-size: 0.875rem;
		line-height: 1.25rem;
		color: var(--foreground);
		margin-bottom: 0.5rem;
	}
	.help-content :global(ul),
	.help-content :global(ol) {
		font-size: 0.875rem;
		line-height: 1.25rem;
		margin-bottom: 0.5rem;
		padding-left: 1.25rem;
	}
	.help-content :global(li) {
		margin-bottom: 0.125rem;
	}
	.help-content :global(table) {
		width: 100%;
		font-size: 0.875rem;
		line-height: 1.25rem;
		border-collapse: collapse;
		border: 1px solid var(--border);
		border-radius: var(--radius-bevel);
		overflow: hidden;
	}
	.help-content :global(th),
	.help-content :global(td) {
		border: 1px solid var(--border);
		padding: 0.5rem 0.75rem;
		text-align: left;
	}
	.help-content :global(th) {
		font-family: var(--font-heading);
		font-weight: 600;
		background-color: var(--muted);
	}
	.help-content :global(code) {
		background-color: var(--muted);
		padding: 0.125rem 0.375rem;
		border-radius: 0.25rem;
		font-size: 0.75rem;
		line-height: 1rem;
		font-family: ui-monospace, monospace;
	}
	.help-content :global(hr) {
		border: none;
		border-top: 1px solid var(--border);
		margin: 0.75rem 0;
	}
	.help-content :global(a) {
		color: var(--primary);
		text-decoration: underline;
	}
	.help-content :global(a:hover) {
		text-decoration: none;
	}
</style>
