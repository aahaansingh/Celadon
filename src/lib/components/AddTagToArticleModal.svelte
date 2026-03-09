<script lang="ts">
	import type { Article, Tag } from '$lib/api';
	import { X, Hash, Plus } from 'lucide-svelte';
	import { fade, scale } from 'svelte/transition';

	let {
		article,
		tags,
		assignedTagIds = [],
		onApply,
		onCreateAndTag,
		onClose
	} = $props<{
		article: Article;
		tags: Tag[];
		/** Tag IDs currently assigned to this article. */
		assignedTagIds?: number[];
		/** Called with the final set of selected tag IDs when user clicks Apply. */
		onApply: (selectedTagIds: number[]) => void;
		/** Create a new tag and return its id; modal will add it to selection and refresh tag list. */
		onCreateAndTag: (name: string) => Promise<number>;
		onClose: () => void;
	}>();

	let selectedIds = $state<Set<number>>(new Set(assignedTagIds));
	let newTagName = $state('');
	let creating = $state(false);

	// Sync selection only when we open for a different article (don't overwrite user toggles)
	let prevArticleId = $state<number | null>(null);
	$effect(() => {
		if (article.id !== prevArticleId) {
			prevArticleId = article.id;
			selectedIds = new Set(assignedTagIds ?? []);
		}
	});

	function toggle(tagId: number) {
		selectedIds = new Set(selectedIds);
		if (selectedIds.has(tagId)) selectedIds.delete(tagId);
		else selectedIds.add(tagId);
	}

	async function handleCreateAndTag() {
		const name = newTagName.trim();
		if (!name || creating) return;
		creating = true;
		try {
			const newId = await onCreateAndTag(name);
			selectedIds = new Set(selectedIds).add(newId);
			newTagName = '';
		} catch (e) {
			console.error(e);
		} finally {
			creating = false;
		}
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
			<h2 class="font-heading font-bold text-sm">Tags for article</h2>
			<button type="button" onclick={onClose} class="p-1 hover:bg-muted rounded-lg transition-colors">
				<X class="w-4 h-4" />
			</button>
		</div>
		<div class="p-4 space-y-4">
			<p class="text-xs text-muted-foreground line-clamp-2 font-body">{article.name}</p>
			{#if tags.length > 0}
				<div class="space-y-1">
					<p class="text-[10px] font-heading uppercase tracking-wider text-muted-foreground">Select tags</p>
					<div class="max-h-40 overflow-y-auto space-y-0.5">
						{#each tags as tag (tag.id)}
							<button
								type="button"
								onclick={() => toggle(tag.id)}
								class="w-full px-3 py-2 text-left text-sm font-body flex items-center gap-3 hover:bg-muted rounded-lg transition-colors"
							>
								<!-- Circle: empty when unselected, inner dot when selected (like read circles but multi-select) -->
								<span
									class="shrink-0 w-4 h-4 rounded-full border-2 flex items-center justify-center transition-colors {selectedIds.has(tag.id)
										? 'border-primary bg-primary/10'
										: 'border-muted-foreground/50 bg-transparent'}"
								>
									{#if selectedIds.has(tag.id)}
										<span class="w-2 h-2 rounded-full bg-primary"></span>
									{/if}
								</span>
								<Hash class="w-3.5 h-3.5 text-muted-foreground shrink-0" />
								<span class="min-w-0 truncate">{tag.name}</span>
							</button>
						{/each}
					</div>
				</div>
			{/if}
			<div class="space-y-2">
				<p class="text-[10px] font-heading uppercase tracking-wider text-muted-foreground">Create new tag</p>
				<div class="flex gap-2">
					<input
						type="text"
						bind:value={newTagName}
						placeholder="Tag name"
						class="flex-1 bg-muted border border-border rounded-lg px-3 py-2 text-sm font-body focus:ring-2 focus:ring-primary/20 focus:outline-none"
						onkeydown={(e) => e.key === 'Enter' && handleCreateAndTag()}
					/>
					<button
						type="button"
						onclick={handleCreateAndTag}
						disabled={!newTagName.trim() || creating}
						class="px-3 py-2 bg-primary text-primary-foreground rounded-lg font-heading text-sm font-bold disabled:opacity-50 flex items-center gap-1.5"
					>
						<Plus class="w-4 h-4" />
						{creating ? '…' : 'Add'}
					</button>
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
				<button type="button" onclick={onClose} class="px-4 py-2 bg-muted hover:bg-muted/80 rounded-lg font-body text-sm">
					Cancel
				</button>
			</div>
		</div>
	</div>
</div>
