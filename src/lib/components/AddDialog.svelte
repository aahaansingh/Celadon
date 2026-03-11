<script lang="ts">
	import { X, Plus, Rss, Layers, Hash } from 'lucide-svelte';
	import { fade, scale } from 'svelte/transition';

	let { isOpen, onClose, onAddFeed, onCreateSuperfeed, onCreateTag } = $props<{
		isOpen: boolean;
		onClose: () => void;
		onAddFeed: (url: string, feedType: 'News' | 'Article' | 'Essay') => void;
		onCreateSuperfeed: (name: string) => void;
		onCreateTag: (name: string) => void;
	}>();

	let tab = $state<'feed' | 'superfeed' | 'tag'>('feed');
	let inputVal = $state('');
	let feedType = $state<'News' | 'Article' | 'Essay'>('News');

	function handleSubmit() {
		if (!inputVal.trim()) return;
		if (tab === 'feed') {
			onAddFeed(inputVal.trim(), feedType);
		} else if (tab === 'superfeed') {
			onCreateSuperfeed(inputVal.trim());
		} else {
			onCreateTag(inputVal.trim());
		}
		inputVal = '';
		onClose();
	}
</script>

{#if isOpen}
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
				<h2 class="font-heading font-bold text-sm">Fill Your Cabinet</h2>
				<button onclick={onClose} class="p-1 hover:bg-muted rounded-full transition-colors">
					<X class="w-4 h-4" />
				</button>
			</div>

			<div class="p-6 space-y-6">
				<!-- Tabs -->
				<div class="flex p-1 bg-muted rounded-xl gap-1">
					<button
						onclick={() => (tab = 'feed')}
						class="flex-1 py-2 text-xs font-heading font-bold rounded-lg transition-all {tab ===
						'feed'
							? 'bg-background shadow-md ring-1 ring-border text-primary'
							: 'text-muted-foreground'}"
					>
						New Feed
					</button>
					<button
						onclick={() => (tab = 'superfeed')}
						class="flex-1 py-2 text-xs font-heading font-bold rounded-lg transition-all {tab ===
						'superfeed'
							? 'bg-background shadow-md ring-1 ring-border text-primary'
							: 'text-muted-foreground'}"
					>
						New Superfeed
					</button>
					<button
						onclick={() => (tab = 'tag')}
						class="flex-1 py-2 text-xs font-heading font-bold rounded-lg transition-all {tab ===
						'tag'
							? 'bg-background shadow-md ring-1 ring-border text-primary'
							: 'text-muted-foreground'}"
					>
						New Tag
					</button>
				</div>

				<div class="space-y-4">
					<div class="space-y-2">
						<label
							for="add-input"
							class="text-xs font-heading font-bold text-muted-foreground ml-1"
						>
							{tab === 'feed' ? 'RSS/Atom URL' : tab === 'superfeed' ? 'Superfeed name' : 'Tag name'}
						</label>
						<div class="relative">
							<input
								id="add-input"
								type="text"
								bind:value={inputVal}
								placeholder={tab === 'feed' ? 'https://example.com/feed.xml' : tab === 'superfeed' ? 'My Collection' : 'My Tag'}
								class="w-full bg-muted border-none rounded-2xl px-5 py-3 pr-12 focus:ring-2 focus:ring-primary/20 transition-all font-body text-sm"
								onkeydown={(e) => e.key === 'Enter' && handleSubmit()}
							/>
							<div class="absolute right-4 top-1/2 -translate-y-1/2">
								{#if tab === 'feed'}
									<Rss class="w-4 h-4 text-muted-foreground" />
								{:else if tab === 'superfeed'}
									<Layers class="w-4 h-4 text-muted-foreground" />
								{:else}
									<Hash class="w-4 h-4 text-muted-foreground" />
								{/if}
							</div>
						</div>
					</div>

					{#if tab === 'feed'}
						<div class="space-y-2">
							<label for="add-feed-type" class="text-xs font-heading font-bold text-muted-foreground ml-1">Type</label>
							<select
								id="add-feed-type"
								bind:value={feedType}
								class="w-full bg-muted border-none rounded-2xl px-4 py-2 font-body text-sm"
							>
								<option value="News">News (1D)</option>
								<option value="Article">Article (3D)</option>
								<option value="Essay">Essay (1W)</option>
							</select>
						</div>
					{/if}

					<p class="text-[10px] text-muted-foreground italic px-1">
						{tab === 'feed'
							? 'Adding a feed will start the syndicator engine to gather articles.'
							: tab === 'superfeed'
								? 'Superfeeds allow you to group multiple sources into a single view.'
								: 'Tags let you label articles for later filtering.'}
					</p>
				</div>

				<button
					onclick={handleSubmit}
					disabled={!inputVal.trim()}
					class="w-full py-4 bg-primary hover:bg-primary-dark disabled:opacity-50 disabled:hover:bg-primary text-white font-heading font-bold beveled flex items-center justify-center gap-3 transition-all"
				>
					<Plus class="w-5 h-5" />
					<span>Confirm</span>
				</button>
			</div>
		</div>
	</div>
{/if}
