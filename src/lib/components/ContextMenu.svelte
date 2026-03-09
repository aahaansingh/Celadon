<script lang="ts">
	import { Layers, Tag, CheckCircle, Eye } from 'lucide-svelte';

	let {
		x,
		y,
		type,
		feedId,
		articleActions,
		onOpenAddToSuperfeed,
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
		/** Called when user clicks "Add to superfeed"; opens the add-to-superfeed modal. */
		onOpenAddToSuperfeed?: (feedId: number) => void;
		onClose: () => void;
	}>();
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
	{#if type === 'feed' && feedId != null && onOpenAddToSuperfeed}
		<button
			type="button"
			class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors"
			onclick={() => {
				onOpenAddToSuperfeed(feedId);
				onClose();
			}}
		>
			<Layers class="w-4 h-4 text-muted-foreground" />
			Add to superfeed
		</button>
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
