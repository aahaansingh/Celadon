<script lang="ts">
	import type { Feed, Superfeed } from '$lib/api';
	import {
		getSuperfeedIdsForFeed,
		updateFeedName,
		updateFeedType,
		addFeedToSuperfeed,
		removeFeedFromSuperfeed
	} from '$lib/api';
	import { X } from 'lucide-svelte';
	import { fade, scale } from 'svelte/transition';

	let { feed, superfeeds, onClose, onSaved } = $props<{
		feed: Feed;
		superfeeds: Superfeed[];
		onClose: () => void;
		onSaved?: (updated: { name: string; feed_type: 'News' | 'Article' | 'Essay'; superfeedIds: number[] }) => void;
	}>();

	let name = $state(feed.name);
	let feedType = $state< 'News' | 'Article' | 'Essay'>(feed.feed_type);
	let superfeedIds = $state<Set<number>>(new Set());
	let loading = $state(true);
	let saving = $state(false);

	$effect(() => {
		getSuperfeedIdsForFeed(feed.id).then((ids) => {
			superfeedIds = new Set(ids);
			loading = false;
		});
	});

	async function handleSave() {
		saving = true;
		try {
			if (name.trim() !== feed.name) await updateFeedName(feed.id, name.trim());
			if (feedType && feedType !== feed.feed_type)
				await updateFeedType(feed.id, feedType);
			const current = await getSuperfeedIdsForFeed(feed.id);
			const currentSet = new Set(current);
			for (const s of superfeeds) {
				const want = superfeedIds.has(s.id);
				const has = currentSet.has(s.id);
				if (want && !has) await addFeedToSuperfeed(feed.id, s.id);
				if (!want && has) await removeFeedFromSuperfeed(feed.id, s.id);
			}
			onSaved?.({ name: name.trim(), feed_type: feedType, superfeedIds: [...superfeedIds] });
			onClose();
		} catch (e) {
			console.error(e);
		} finally {
			saving = false;
		}
	}

	function toggleSuperfeed(id: number) {
		superfeedIds = new Set(superfeedIds);
		if (superfeedIds.has(id)) superfeedIds.delete(id);
		else superfeedIds.add(id);
		superfeedIds = new Set(superfeedIds);
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	transition:fade={{ duration: 150 }}
	class="fixed inset-0 z-[200] bg-background/80 backdrop-blur-sm flex items-center justify-center p-6"
	onclick={onClose}
>
	<div
		transition:scale={{ duration: 200, start: 0.95 }}
		class="bg-card border border-border beveled w-full max-w-md shadow-2xl overflow-hidden"
		onclick={(e) => e.stopPropagation()}
	>
		<div class="px-6 py-4 border-b border-border flex items-center justify-between">
			<h2 class="font-heading font-bold text-sm">Feed settings</h2>
			<button onclick={onClose} class="p-1 hover:bg-muted rounded-full transition-colors">
				<X class="w-4 h-4" />
			</button>
		</div>
		<div class="p-6 space-y-4">
			<div>
				<label for="feed-settings-name" class="text-xs font-heading font-bold text-muted-foreground block mb-1">Name</label>
				<input
					id="feed-settings-name"
					type="text"
					bind:value={name}
					class="w-full bg-muted border-none rounded-2xl px-4 py-2 font-body text-sm"
				/>
				<p class="mt-1.5 text-xs font-body text-muted-foreground/80 truncate" title={feed.url}>
					{feed.url}
				</p>
			</div>
			<div>
				<span class="text-xs font-heading font-bold text-muted-foreground block mb-1">Type</span>
				<div class="flex gap-2 p-1 bg-muted rounded-2xl">
					<button
						type="button"
						onclick={() => (feedType = 'News')}
						class="flex-1 py-2 rounded-xl text-xs font-body font-medium transition-all {feedType === 'News'
							? 'bg-background shadow-sm ring-1 ring-border text-primary'
							: 'text-muted-foreground'}"
					>
						News (1D)
					</button>
					<button
						type="button"
						onclick={() => (feedType = 'Article')}
						class="flex-1 py-2 rounded-xl text-xs font-body font-medium transition-all {feedType === 'Article'
							? 'bg-background shadow-sm ring-1 ring-border text-primary'
							: 'text-muted-foreground'}"
					>
						Article (3D)
					</button>
					<button
						type="button"
						onclick={() => (feedType = 'Essay')}
						class="flex-1 py-2 rounded-xl text-xs font-body font-medium transition-all {feedType === 'Essay'
							? 'bg-background shadow-sm ring-1 ring-border text-primary'
							: 'text-muted-foreground'}"
					>
						Essay (1W)
					</button>
				</div>
			</div>
			<div>
				<span class="text-xs font-heading font-bold text-muted-foreground block mb-2">Superfeeds</span>
				{#if loading}
					<p class="text-sm text-muted-foreground">Loading…</p>
				{:else}
					<div class="space-y-1 max-h-40 overflow-y-auto">
						{#each superfeeds as s (s.id)}
							<button
								type="button"
								onclick={() => toggleSuperfeed(s.id)}
								class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left transition-colors hover:bg-muted/50"
							>
								<span
									class="w-4 h-4 rounded-full border-2 shrink-0 flex items-center justify-center transition-colors {superfeedIds.has(s.id)
										? 'border-primary bg-primary/10'
										: 'border-muted-foreground/50'}"
								>
									{#if superfeedIds.has(s.id)}
										<span class="w-2 h-2 rounded-full bg-primary"></span>
									{/if}
								</span>
								<span class="text-sm font-body">{s.name}</span>
							</button>
						{/each}
					</div>
				{/if}
			</div>
			<div class="flex gap-2 pt-2">
				<button
					onclick={handleSave}
					disabled={saving}
					class="flex-1 py-2 bg-primary hover:bg-primary-dark disabled:opacity-50 text-white font-heading font-bold rounded-xl"
				>
					{saving ? 'Saving…' : 'Save'}
				</button>
				<button
					onclick={onClose}
					class="px-4 py-2 bg-muted hover:bg-muted/80 rounded-xl font-heading text-sm"
				>
					Cancel
				</button>
			</div>
		</div>
	</div>
</div>
