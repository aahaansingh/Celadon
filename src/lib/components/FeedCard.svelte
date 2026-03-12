<script lang="ts">
	import type { Feed } from '$lib/api';
	import { Rss, Settings, Layers } from 'lucide-svelte';

	let { feed, onClick, onSettings, onContextMenu, superfeeds = [], onSuperfeedClick } = $props<{
		feed: Feed;
		onClick: () => void;
		onSettings: () => void;
		onContextMenu?: (e: MouseEvent) => void;
		superfeeds?: { id: number; name: string }[];
		onSuperfeedClick?: (id: number, name: string) => void;
	}>();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	onclick={onClick}
	oncontextmenu={(e) => {
		e.preventDefault();
		onContextMenu?.(e);
	}}
	class="bg-card border border-border beveled p-4 cursor-pointer hover:shadow-lg transition-all duration-300 aspect-square flex flex-col justify-between group"
>
	<div class="flex flex-col items-center justify-center flex-1 gap-3">
		<div class="flex items-center gap-2 w-full justify-center">
			<div class="p-3 bg-celadon/20 rounded-2xl group-hover:bg-celadon/30 transition-colors">
				<Rss class="w-8 h-8 text-celadon-dark" />
			</div>
			{#if feed.status === 1}
				<span
					class="w-2.5 h-2.5 rounded-full bg-amber-500 shrink-0"
					title="Rate limited; will retry after backoff"
				></span>
			{:else if feed.consecutive_http_errors >= 1 && feed.consecutive_http_errors <= 2}
				<span
					class="w-2.5 h-2.5 rounded-full bg-amber-500 shrink-0"
					title="Error code: {feed.status}"
				></span>
			{:else if feed.consecutive_http_errors >= 3}
				<span
					class="w-2.5 h-2.5 rounded-full bg-red-500 shrink-0"
					title={feed.status !== 0 ? `Error code: ${feed.status}` : 'Feed broken'}
				></span>
			{/if}
		</div>
		<h3 class="font-heading font-bold text-center line-clamp-2 px-2">
			{feed.name}
		</h3>
		<p class="text-[10px] text-muted-foreground uppercase tracking-widest font-bold">
			{feed.feed_type}
		</p>
	</div>

	<div class="flex items-center justify-between gap-2 pt-3 border-t border-border min-w-0">
		<div class="flex items-center gap-2 min-w-0 flex-1 overflow-hidden">
			<Layers class="w-3.5 h-3.5 text-muted-foreground shrink-0" />
			<div class="flex gap-1 overflow-x-auto overflow-y-hidden min-w-0">
				{#each superfeeds as s}
					{#if onSuperfeedClick}
						<button
							type="button"
							onclick={(e) => {
								e.stopPropagation();
								onSuperfeedClick(s.id, s.name);
							}}
							class="text-[10px] font-body px-2 py-0.5 rounded-full bg-muted text-muted-foreground hover:bg-primary/20 hover:text-primary pill-hover-lighten whitespace-nowrap shrink-0 transition-colors"
						>
							{s.name}
						</button>
					{:else}
						<span class="text-[10px] font-body px-2 py-0.5 rounded-full bg-muted text-muted-foreground whitespace-nowrap shrink-0">
							{s.name}
						</span>
					{/if}
				{/each}
			</div>
		</div>
		<button
			onclick={(e) => {
				e.stopPropagation();
				onSettings();
			}}
			class="p-1.5 hover:bg-muted rounded-lg transition-colors text-muted-foreground hover:text-primary shrink-0"
		>
			<Settings class="w-3.5 h-3.5" />
		</button>
	</div>
</div>
