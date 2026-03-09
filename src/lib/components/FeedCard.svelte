<script lang="ts">
	import type { Feed } from '$lib/api';
	import { Rss, Settings } from 'lucide-svelte';

	let { feed, onClick, onSettings, onContextMenu } = $props<{
		feed: Feed;
		onClick: () => void;
		onSettings: () => void;
		onContextMenu?: (e: MouseEvent) => void;
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
		<div class="p-3 bg-celadon/20 rounded-2xl group-hover:bg-celadon/30 transition-colors">
			<Rss class="w-8 h-8 text-celadon-dark" />
		</div>
		<h3 class="font-heading font-bold text-center line-clamp-2 px-2">
			{feed.name}
		</h3>
		<p class="text-[10px] text-muted-foreground uppercase tracking-widest font-bold">
			{feed.feed_type}
		</p>
	</div>

	<div class="flex items-center justify-between pt-3 border-t border-border">
		<span class="text-[10px] text-muted-foreground font-body">
			Last: {new Date(feed.last_fetched).toLocaleDateString()}
		</span>
		<button
			onclick={(e) => {
				e.stopPropagation();
				onSettings();
			}}
			class="p-1.5 hover:bg-muted rounded-lg transition-colors text-muted-foreground hover:text-primary"
		>
			<Settings class="w-3.5 h-3.5" />
		</button>
	</div>
</div>
