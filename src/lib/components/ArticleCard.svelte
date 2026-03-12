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

	const typeColors: Record<string, string> = {
		News: 'bg-blue-100 text-blue-900 dark:bg-blue-900/30 dark:text-blue-300',
		Article: 'bg-amber-100 text-amber-900 dark:bg-amber-900/30 dark:text-amber-300',
		Essay: 'bg-purple-200 text-purple-900 dark:bg-purple-900/30 dark:text-purple-300'
	};

	const typeLabels: Record<string, string> = {
		News: '1D',
		Article: '3D',
		Essay: '1W'
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
			<div class="min-w-0 flex-1 overflow-hidden" style="max-height: calc(3 * 1.25 * 0.875rem);">
				<h3
					class="line-clamp-3 text-sm font-heading leading-snug group-hover:text-primary transition-colors break-words"
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
			<div class="min-h-0 overflow-hidden">
				<p class="text-xs text-muted-foreground font-body italic line-clamp-2 overflow-hidden leading-snug">
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

	<div class="flex items-center justify-between gap-2 mt-3 pt-3 border-t border-border">
		<div class="flex items-center gap-2 min-w-0 flex-1 overflow-hidden">
			{#if feed}
				<span
					class={`text-[10px] font-bold px-2 py-0.5 rounded-full uppercase tracking-wider shrink-0 ${typeColors[feed.feed_type]}`}
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
</style>
