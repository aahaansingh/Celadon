<script lang="ts">
	import type { Superfeed } from '$lib/api';
	import { X, Layers } from 'lucide-svelte';
	import { fade, scale } from 'svelte/transition';

	let {
		feedId,
		feedName,
		superfeeds,
		assignedSuperfeedIds = [],
		onApply,
		onClose
	} = $props<{
		feedId: number;
		feedName: string;
		superfeeds: Superfeed[];
		/** Superfeed IDs this feed is currently in. */
		assignedSuperfeedIds?: number[];
		/** Called with the final set of selected superfeed IDs when user clicks Apply. */
		onApply: (selectedSuperfeedIds: number[]) => void;
		onClose: () => void;
	}>();

	let selectedIds = $state<Set<number>>(new Set(assignedSuperfeedIds));

	// Sync selection only when we open for a different feed (don't overwrite user toggles)
	let prevFeedId = $state<number | null>(null);
	$effect(() => {
		if (feedId !== prevFeedId) {
			prevFeedId = feedId;
			selectedIds = new Set(assignedSuperfeedIds ?? []);
		}
	});

	function toggle(superfeedId: number) {
		selectedIds = new Set(selectedIds);
		if (selectedIds.has(superfeedId)) selectedIds.delete(superfeedId);
		else selectedIds.add(superfeedId);
	}

	function handleApply() {
		onApply([...selectedIds]);
		onClose();
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	transition:fade={{ duration: 150 }}
	class="fixed inset-0 z-[260] bg-background/80 backdrop-blur-sm flex items-center justify-center p-6"
	onclick={onClose}
>
	<div
		transition:scale={{ duration: 200, start: 0.95 }}
		class="bg-card border border-border rounded-xl w-full max-w-sm shadow-2xl overflow-hidden"
		onclick={(e) => e.stopPropagation()}
	>
		<div class="px-4 py-3 border-b border-border flex items-center justify-between">
			<h2 class="font-heading font-bold text-sm">Feed in superfeeds</h2>
			<button type="button" onclick={onClose} class="p-1 hover:bg-muted rounded-lg transition-colors">
				<X class="w-4 h-4" />
			</button>
		</div>
		<div class="p-4 space-y-4">
			<p class="text-xs text-muted-foreground line-clamp-2 font-body">{feedName}</p>
			{#if superfeeds.length > 0}
				<div class="space-y-1">
					<p class="text-[10px] font-heading text-muted-foreground">Select superfeeds</p>
					<div class="max-h-48 overflow-y-auto space-y-0.5">
						{#each superfeeds as s (s.id)}
							<button
								type="button"
								onclick={() => toggle(s.id)}
								class="w-full px-3 py-2 text-left text-sm font-body flex items-center gap-3 hover:bg-muted rounded-lg transition-colors"
							>
								<!-- Circle: empty when unselected, inner dot when selected -->
								<span
									class="shrink-0 w-4 h-4 rounded-full border-2 flex items-center justify-center transition-colors {selectedIds.has(s.id)
										? 'border-primary bg-primary/10'
										: 'border-muted-foreground/50 bg-transparent'}"
								>
									{#if selectedIds.has(s.id)}
										<span class="w-2 h-2 rounded-full bg-primary"></span>
									{/if}
								</span>
								<Layers class="w-3.5 h-3.5 text-muted-foreground shrink-0" />
								<span class="min-w-0 truncate">{s.name}</span>
							</button>
						{/each}
					</div>
				</div>
				<div class="flex gap-2 pt-2">
					<button
						type="button"
						onclick={handleApply}
						class="flex-1 py-2 bg-primary text-primary-foreground rounded-lg font-heading text-sm font-bold"
					>
						Apply
					</button>
					<button type="button" onclick={onClose} class="flex-1 py-2 bg-muted hover:bg-muted/80 rounded-lg font-heading text-sm">
						Cancel
					</button>
				</div>
			{:else}
				<p class="text-sm text-muted-foreground font-body italic">No superfeeds yet. Create one from the Add menu.</p>
			{/if}
		</div>
	</div>
</div>
