<script lang="ts">
	import type { Article, Feed } from '$lib/api';
	import { openInBrowser } from '$lib/api';
	import { decodeHtmlEntities } from '$lib/sanitizeHtml';
	import { Clock, CheckCircle2, Circle, Tag, ExternalLink } from 'lucide-svelte';

	let { article, feed, onToggleRead, onAddTag, onShowFeed, onClick, onContextMenu, tags = [], onTagClick } = $props<{
		article: Article;
		feed?: Feed;
		onToggleRead: () => void;
		onAddTag: () => void;
		onShowFeed: () => void;
		onClick: () => void;
		onContextMenu?: (e: MouseEvent) => void;
		tags?: { id: number; name: string }[];
		onTagClick?: (id: number, name: string) => void;
	}>();

	// Use fixed class names so Tailwind doesn't purge them (dynamic class strings are not detected)
	const typePillClass: Record<string, string> = {
		News: 'feed-pill feed-pill-news',
		Article: 'feed-pill feed-pill-article',
		Essay: 'feed-pill feed-pill-essay',
		Update: 'feed-pill feed-pill-update'
	};

	const typeLabels: Record<string, string> = {
		News: '1D',
		Article: '3D',
		Essay: '1W',
		Update: '6h'
	};

	function getTimeAgo(dateStr: string) {
		const date = new Date(dateStr);
		const now = new Date();
		const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);

		if (diffInSeconds < 60) return 'Just now';
		if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)}m ago`;
		if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)}h ago`;
		return `${Math.floor(diffInSeconds / 86400)}d ago`;
	}

	/** Strip HTML tags and take first ~100 chars for preview; decode entities. */
	function previewText(html: string | null | undefined): string {
		if (!html || !html.trim()) return '';
		const div = typeof document !== 'undefined' ? document.createElement('div') : null;
		if (!div) {
			const stripped = html.replace(/<[^>]+>/g, '').trim();
			return decodeHtmlEntities(stripped).slice(0, 100) + (stripped.length > 100 ? '…' : '');
		}
		div.innerHTML = html;
		const text = (div.textContent || div.innerText || '').replace(/\s+/g, ' ').trim();
		return decodeHtmlEntities(text).slice(0, 100) + (text.length > 100 ? '…' : '');
	}

	let timeAgo = $derived(getTimeAgo(article.published));
	let descriptionPreview = $derived(previewText(article.description));
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	onclick={onClick}
	oncontextmenu={(e) => {
		e.preventDefault();
		onContextMenu?.(e);
	}}
	class="bg-card border border-border beveled p-4 cursor-pointer hover:shadow-lg transition-all duration-300 min-h-[260px] flex flex-col justify-between group relative overflow-hidden"
>
	<div class="flex-1 overflow-hidden flex flex-col gap-2">
		<div class="flex items-start justify-between gap-2 min-h-0">
			<div class="min-w-0 flex-1 overflow-hidden" style="max-height: calc(3 * 1.25 * 0.875rem + 0.2rem);">
				<h3
					class="line-clamp-3 text-sm font-heading leading-snug group-hover:text-primary transition-colors break-words pb-px"
				>
					{decodeHtmlEntities(article.name)}
				</h3>
			</div>
			<button
				onclick={(e) => {
					e.stopPropagation();
					onToggleRead();
				}}
				class="shrink-0 mt-0.5 p-1 hover:bg-muted rounded-full transition-colors"
			>
				{#if article.read}
					<CheckCircle2 class="w-4 h-4 text-muted-foreground" />
				{:else}
					<Circle class="w-4 h-4 text-primary fill-primary/20" />
				{/if}
			</button>
		</div>

		{#if descriptionPreview}
			<div class="min-h-0 overflow-hidden" style="max-height: calc(2 * 1.25 * 0.75rem + 0.15rem);">
				<p class="text-xs text-muted-foreground font-body italic line-clamp-2 overflow-hidden leading-snug pb-px">
					{descriptionPreview}
				</p>
			</div>
		{/if}

		<div class="space-y-1.5 text-xs text-muted-foreground font-body">
			<p class="truncate font-medium">{feed ? decodeHtmlEntities(feed.name) : 'Unknown Feed'}</p>
			<div class="flex items-center gap-1">
				<Clock class="w-3 h-3" />
				<span>{timeAgo}</span>
			</div>
		</div>
	</div>

	<div class="flex items-center justify-between gap-2 mt-3 pt-3 border-t border-border min-h-[28px] items-center">
		<div class="flex items-center gap-2 min-w-0 flex-1 overflow-hidden">
			{#if feed}
				<span
					class={`text-[10px] font-bold px-2 py-1 rounded-full uppercase tracking-wider shrink-0 leading-none flex items-center ${typePillClass[feed.feed_type]}`}
				>
					{typeLabels[feed.feed_type]}
				</span>
			{/if}
			<Tag class="w-3 h-3 text-muted-foreground shrink-0" />
			<div class="flex gap-1 overflow-x-auto overflow-y-hidden min-w-0">
				{#each tags as tag}
					{#if onTagClick}
						<button
							type="button"
							onclick={(e) => {
								e.stopPropagation();
								onTagClick(tag.id, tag.name);
							}}
							class="text-[10px] font-body px-2 py-0.5 rounded-full bg-muted text-muted-foreground hover:bg-primary/20 hover:text-primary pill-hover-lighten whitespace-nowrap shrink-0 transition-colors"
						>
							{decodeHtmlEntities(tag.name)}
						</button>
					{:else}
						<span class="text-[10px] font-body px-2 py-0.5 rounded-full bg-muted text-muted-foreground whitespace-nowrap shrink-0">
							{decodeHtmlEntities(tag.name)}
						</span>
					{/if}
				{/each}
			</div>
		</div>

		<button
			onclick={(e) => {
				e.stopPropagation();
				openInBrowser(article.url);
			}}
			class="text-muted-foreground hover:text-primary transition-colors"
		>
			<ExternalLink class="w-3 link-icon" />
		</button>
	</div>
</div>

<style>
	.link-icon {
		width: 14px;
		height: 14px;
	}
	/* Feed type pills: explicit colors so they are not overridden by parent text-foreground or purged as dynamic Tailwind classes */
	.feed-pill-news {
		background-color: #dbeafe;
		color: #1d4ed8;
	}
	.feed-pill-article {
		background-color: #fef3c7;
		color: #b45309;
	}
	.feed-pill-essay {
		background-color: #e9d5ff;
		color: #7e22ce;
	}
	.feed-pill-update {
		background-color: #ccfbf1;
		color: #0f766e;
	}
	:global(.dark) .feed-pill-news {
		background-color: rgb(30 58 138 / 0.3);
		color: rgb(147 197 253);
	}
	:global(.dark) .feed-pill-article {
		background-color: rgb(120 53 15 / 0.3);
		color: rgb(253 224 71);
	}
	:global(.dark) .feed-pill-essay {
		background-color: rgb(88 28 135 / 0.3);
		color: rgb(216 180 254);
	}
	:global(.dark) .feed-pill-update {
		background-color: rgb(19 78 74 / 0.3);
		color: rgb(94 234 212);
	}
</style>
