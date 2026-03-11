<script lang="ts">
	import type { Article } from '$lib/api';
	import { openInBrowser } from '$lib/api';
	import { sanitizeHtml } from '$lib/sanitizeHtml';
	import { X, ExternalLink } from 'lucide-svelte';
	import { fade } from 'svelte/transition';

	let { article, onClose } = $props<{
		article: Article | null;
		onClose: () => void;
	}>();

	let mode = $state<'simple' | 'full'>('simple');
	let simpleContent = $derived(article?.description ? sanitizeHtml(article.description) : '');
</script>

{#if article}
	<div
		transition:fade={{ duration: 200 }}
		class="fixed inset-0 z-[100] bg-background/95 backdrop-blur-xl flex flex-col font-body"
	>
		<!-- Header -->
		<header class="px-6 py-4 border-b border-border flex items-center justify-between">
			<div class="flex items-center gap-4">
				<button onclick={onClose} class="p-2 hover:bg-muted rounded-full transition-colors">
					<X class="w-5 h-5" />
				</button>
				<h1 class="font-heading font-bold text-lg truncate max-w-xl">
					{article.name}
				</h1>
			</div>

			<div class="flex items-center gap-2">
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
					<iframe src={article.url} title={article.name} class="w-full flex-1 min-h-0 border-none"></iframe>
					<div class="px-4 py-2 bg-muted/50 border-t border-border text-center text-xs text-muted-foreground font-body">
						If the page doesn&apos;t appear, this site may block embedding.
						<button
							onclick={() => openInBrowser(article.url)}
							class="ml-2 underline hover:text-primary"
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
