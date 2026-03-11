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
		onSaved: () => void;
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
			onSaved();
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
			</div>
			<div>
				<label for="feed-settings-type" class="text-xs font-heading font-bold text-muted-foreground block mb-1">Type</label>
				<select
					id="feed-settings-type"
					bind:value={feedType}
					class="w-full bg-muted border-none rounded-2xl px-4 py-2 font-body text-sm"
				>
					<option value="News">News (1D)</option>
					<option value="Article">Article (3D)</option>
					<option value="Essay">Essay (1W)</option>
				</select>
			</div>
			<div>
				<span class="text-xs font-heading font-bold text-muted-foreground block mb-2">Superfeeds</span>
				{#if loading}
					<p class="text-sm text-muted-foreground">Loading…</p>
				{:else}
					<div class="space-y-2 max-h-40 overflow-y-auto">
						{#each superfeeds as s (s.id)}
							<label class="flex items-center gap-2 cursor-pointer">
								<input
									type="checkbox"
									checked={superfeedIds.has(s.id)}
									onchange={() => toggleSuperfeed(s.id)}
									class="rounded border-border"
								/>
								<span class="text-sm font-body">{s.name}</span>
							</label>
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
