<script lang="ts">
	import type { Superfeed } from '$lib/api';
	import { Layers, Tag, CheckCircle, Eye } from 'lucide-svelte';

	let {
		x,
		y,
		type,
		feedId,
		articleActions,
		superfeeds,
		onAddToSuperfeed,
		onClose
	} = $props<{
		x: number;
		y: number;
		type: 'feed' | 'article';
		feedId?: number;
		articleActions?: {
			onToggleRead: () => void;
			onAddTag: () => void;
			onShowFeed: () => void;
			read: boolean;
		};
		superfeeds?: Superfeed[];
		onAddToSuperfeed?: (feedId: number, superfeedId: number) => void;
		onClose: () => void;
	}>();

	let showSuperfeedSubmenu = $state(false);

	function handleAddToSuperfeed(superfeedId: number) {
		if (feedId != null && onAddToSuperfeed) {
			onAddToSuperfeed(feedId, superfeedId);
		}
		showSuperfeedSubmenu = false;
		onClose();
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed z-[250] min-w-[180px] py-1 bg-card border border-border rounded-xl shadow-xl overflow-hidden"
	style="left: {x}px; top: {y}px;"
	role="menu"
	tabindex="-1"
	onclick={(e) => e.stopPropagation()}
>
	{#if type === 'feed' && superfeeds && superfeeds.length > 0 && feedId != null && onAddToSuperfeed}
		<div
			class="relative"
			onmouseenter={() => (showSuperfeedSubmenu = true)}
			onmouseleave={() => (showSuperfeedSubmenu = false)}
		>
			<button
				type="button"
				class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors"
				onclick={() => (showSuperfeedSubmenu = !showSuperfeedSubmenu)}
			>
				<Layers class="w-4 h-4 text-muted-foreground" />
				Add to superfeed
			</button>
			{#if showSuperfeedSubmenu}
				<div
					class="absolute left-full top-0 ml-0.5 min-w-[160px] py-1 bg-card border border-border rounded-xl shadow-xl"
					role="menu"
				>
					{#each superfeeds as s (s.id)}
						<button
							type="button"
							class="w-full px-4 py-2 text-left text-sm font-body hover:bg-muted transition-colors"
							onclick={() => handleAddToSuperfeed(s.id)}
						>
							{s.name}
						</button>
					{/each}
				</div>
			{/if}
		</div>
	{:else if type === 'article' && articleActions}
		<button
			type="button"
			class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors"
			onclick={() => {
				articleActions.onToggleRead();
				onClose();
			}}
		>
			{#if articleActions.read}
				<CheckCircle class="w-4 h-4 text-muted-foreground" />
				Mark unread
			{:else}
				<CheckCircle class="w-4 h-4 text-primary" />
				Mark read
			{/if}
		</button>
		<button
			type="button"
			class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors"
			onclick={() => {
				articleActions.onAddTag();
				onClose();
			}}
		>
			<Tag class="w-4 h-4 text-muted-foreground" />
			Add tag
		</button>
		<button
			type="button"
			class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors"
			onclick={() => {
				articleActions.onShowFeed();
				onClose();
			}}
		>
			<Eye class="w-4 h-4 text-muted-foreground" />
			Show feed view
		</button>
	{/if}
</div>
