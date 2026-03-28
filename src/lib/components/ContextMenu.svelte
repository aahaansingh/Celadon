<script lang="ts">
	import type { Feed, Superfeed, Tag } from '$lib/api';
	import { Layers, Tag as TagIcon, CheckCircle, Eye, Copy, Trash2, Rss } from 'lucide-svelte';

	let {
		x,
		y,
		type,
		feed,
		feedId,
		articleActions,
		superfeedId,
		superfeed,
		tagId,
		tag,
		onCopyFeedLink,
		onShowSuperfeeds,
		onOpenAddToSuperfeed,
		onMarkAllReadFeed,
		onDeleteFeed,
		onShowFeedsInSuperfeed,
		onDeleteSuperfeed,
		onDeleteTag,
		onClose
	} = $props<{
		x: number;
		y: number;
		type: 'feed' | 'article' | 'superfeed' | 'tag';
		feed?: Feed;
		feedId?: number;
		articleActions?: {
			onToggleRead: () => void;
			onAddTag: () => void;
			onShowFeed: () => void;
			onShowFeedCard: () => void;
			onCopyLink?: () => void;
			read: boolean;
		};
		superfeedId?: number;
		superfeed?: Superfeed;
		tagId?: number;
		tag?: Tag;
		onCopyFeedLink?: () => void;
		onShowSuperfeeds?: () => void;
		onOpenAddToSuperfeed?: (feedId: number) => void;
		onMarkAllReadFeed?: (feedId: number) => void;
		onDeleteFeed?: (feedId: number) => void;
		onShowFeedsInSuperfeed?: (superfeedId: number) => void;
		onDeleteSuperfeed?: (superfeedId: number) => void;
		onDeleteTag?: (tagId: number) => void;
		onClose: () => void;
	}>();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed z-[250] min-w-[180px] py-1 bg-background border border-border rounded-xl shadow-xl overflow-hidden"
	style="left: {x}px; top: {y}px;"
	role="menu"
	tabindex="-1"
	onclick={(e) => e.stopPropagation()}
>
	{#if type === 'article' && articleActions}
		{#if articleActions.onCopyLink}
			<button
				type="button"
				class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors"
				onclick={() => {
					articleActions.onCopyLink?.();
					onClose();
				}}
			>
				<Copy class="w-4 h-4 text-muted-foreground" />
				Copy article link
			</button>
		{/if}
		<button
			type="button"
			class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors"
			onclick={() => {
				articleActions.onAddTag();
				onClose();
			}}
		>
			<TagIcon class="w-4 h-4 text-muted-foreground" />
			Add tag
		</button>
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
				articleActions.onShowFeed();
				onClose();
			}}
		>
			<Eye class="w-4 h-4 text-muted-foreground" />
			Show feed view
		</button>
		<button
			type="button"
			class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors"
			onclick={() => {
				articleActions.onShowFeedCard();
				onClose();
			}}
		>
			<Rss class="w-4 h-4 text-muted-foreground" />
			Show feed
		</button>
	{:else if type === 'feed' && feedId != null}
		{#if onCopyFeedLink && feed?.url}
			<button
				type="button"
				class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors"
				onclick={() => {
					onCopyFeedLink();
					onClose();
				}}
			>
				<Copy class="w-4 h-4 text-muted-foreground" />
				Copy feed link
			</button>
		{/if}
		{#if onShowSuperfeeds}
			<button
				type="button"
				class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors"
				onclick={() => {
					onShowSuperfeeds();
					onClose();
				}}
			>
				<Layers class="w-4 h-4 text-muted-foreground" />
				Show superfeeds
			</button>
		{/if}
		{#if onOpenAddToSuperfeed}
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
		{/if}
		{#if onMarkAllReadFeed}
			<button
				type="button"
				class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors"
				onclick={() => {
					onMarkAllReadFeed(feedId);
					onClose();
				}}
			>
				<CheckCircle class="w-4 h-4 text-muted-foreground" />
				Mark all as read
			</button>
		{/if}
		{#if onDeleteFeed}
			<button
				type="button"
				class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors text-red-600 dark:text-red-400"
				onclick={() => {
					onDeleteFeed(feedId);
					onClose();
				}}
			>
				<Trash2 class="w-4 h-4" />
				Delete feed
			</button>
		{/if}
	{:else if type === 'superfeed' && superfeedId != null}
		{#if onShowFeedsInSuperfeed}
			<button
				type="button"
				class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors"
				onclick={() => {
					onShowFeedsInSuperfeed(superfeedId);
					onClose();
				}}
			>
				<Rss class="w-4 h-4 text-muted-foreground" />
				Show feeds in superfeed
			</button>
		{/if}
		{#if onDeleteSuperfeed}
			<button
				type="button"
				class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors text-red-600 dark:text-red-400"
				onclick={() => {
					onDeleteSuperfeed(superfeedId);
					onClose();
				}}
			>
				<Trash2 class="w-4 h-4" />
				Delete superfeed
			</button>
		{/if}
	{:else if type === 'tag' && tagId != null && onDeleteTag}
		<button
			type="button"
			class="w-full px-4 py-2 text-left text-sm font-body flex items-center gap-2 hover:bg-muted transition-colors text-red-600 dark:text-red-400"
			onclick={() => {
				onDeleteTag(tagId);
				onClose();
			}}
		>
			<Trash2 class="w-4 h-4" />
			Delete tag
		</button>
	{/if}
</div>
