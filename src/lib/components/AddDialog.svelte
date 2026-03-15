<script lang="ts">
	import type { Superfeed } from '$lib/api';
	import { ALL_SUPERFEED_ID, importOpmlFromContent, exportOpml } from '$lib/api';
	import { X, Plus, Rss, Layers, Hash, FileDown, Upload } from 'lucide-svelte';
	import { fade, scale } from 'svelte/transition';

	let { isOpen, onClose, onAddFeed, onCreateSuperfeed, onCreateTag, onOpmlComplete, superfeeds = [] } = $props<{
		isOpen: boolean;
		onClose: () => void;
		onAddFeed: (url: string, feedType: 'News' | 'Article' | 'Essay' | 'Update', selectedSuperfeedIds: number[]) => void;
		onCreateSuperfeed: (name: string) => void;
		onCreateTag: (name: string) => void;
		onOpmlComplete?: () => void;
		superfeeds?: Superfeed[];
	}>();

	let tab = $state<'feed' | 'superfeed' | 'tag' | 'opml'>('feed');
	let inputVal = $state('');
	let feedType = $state<'News' | 'Article' | 'Essay' | 'Update'>('News');
	let selectedSuperfeedIds = $state<Set<number>>(new Set());
	let opmlStatus = $state<'idle' | 'importing' | 'exporting' | 'success' | 'error'>('idle');
	let opmlMessage = $state('');
	let fileInputRef = $state<HTMLInputElement | null>(null);

	const superfeedsExcludingAll = $derived(superfeeds.filter((s: Superfeed) => s.id !== ALL_SUPERFEED_ID));

	const isTauri = typeof window !== 'undefined' && !!(window as { __TAURI__?: unknown }).__TAURI__;

	$effect(() => {
		if (isOpen) {
			opmlStatus = 'idle';
			opmlMessage = '';
		}
	});

	function toggleSuperfeed(id: number) {
		selectedSuperfeedIds = new Set(selectedSuperfeedIds);
		if (selectedSuperfeedIds.has(id)) selectedSuperfeedIds.delete(id);
		else selectedSuperfeedIds.add(id);
		selectedSuperfeedIds = new Set(selectedSuperfeedIds);
	}

	async function handleOpmlImportPath() {
		fileInputRef?.click();
	}

	async function handleOpmlImportInput(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		input.value = '';
		if (!file) return;
		if (!file.name.endsWith('.opml') && !file.name.endsWith('.xml')) {
			opmlStatus = 'error';
			opmlMessage = 'Please choose an OPML or XML file.';
			return;
		}
		await handleOpmlImportFile(file);
	}

	async function handleOpmlImportFile(file: File) {
		opmlStatus = 'importing';
		opmlMessage = '';
		try {
			const MAX_OPML_BYTES = 5 * 1024 * 1024; // 5MB
			if (file.size > MAX_OPML_BYTES) {
				opmlStatus = 'error';
				opmlMessage = 'File is too large (max 5MB). Use a smaller OPML or split your feeds.';
				return;
			}
			const text = await file.text();
			await importOpmlFromContent(text);
			opmlStatus = 'success';
			opmlMessage = 'Import complete.';
			onOpmlComplete?.();
		} catch (e) {
			opmlStatus = 'error';
			opmlMessage = e instanceof Error ? e.message : String(e);
		}
	}

	async function handleOpmlExport() {
		if (!isTauri) {
			opmlStatus = 'error';
			opmlMessage = 'OPML export is available in the desktop app.';
			return;
		}
		try {
			const { save } = await import('@tauri-apps/api/dialog');
			const path = await save({
				filters: [{ name: 'OPML', extensions: ['opml'] }],
				defaultPath: 'celadon-export.opml'
			});
			if (path) {
				opmlStatus = 'exporting';
				opmlMessage = '';
				await exportOpml(path);
				opmlStatus = 'success';
				opmlMessage = 'Export complete.';
			}
		} catch (e) {
			opmlStatus = 'error';
			opmlMessage = e instanceof Error ? e.message : String(e);
		}
	}

	function handleSubmit() {
		if (!inputVal.trim()) return;
		if (tab === 'feed') {
			onAddFeed(inputVal.trim(), feedType, Array.from(selectedSuperfeedIds));
		} else if (tab === 'superfeed') {
			onCreateSuperfeed(inputVal.trim());
		} else {
			onCreateTag(inputVal.trim());
		}
		inputVal = '';
		selectedSuperfeedIds = new Set();
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
					<button
						onclick={() => (tab = 'opml')}
						class="flex-1 py-2 text-xs font-heading font-bold rounded-lg transition-all {tab ===
						'opml'
							? 'bg-background shadow-md ring-1 ring-border text-primary'
							: 'text-muted-foreground'}"
					>
						OPML
					</button>
				</div>

				<div class="space-y-4">
					{#if tab === 'opml'}
						<div class="space-y-4">
							<p class="text-sm text-muted-foreground">
								Import feeds from an OPML file, or export your feeds to OPML.
							</p>
							<input
								bind:this={fileInputRef}
								type="file"
								accept=".opml,.xml"
								onchange={handleOpmlImportInput}
								class="hidden"
							/>
							<div class="flex gap-2">
								<button
									type="button"
									onclick={() => fileInputRef?.click()}
									disabled={opmlStatus === 'importing'}
									class="flex-1 py-2.5 bg-muted hover:bg-muted/80 disabled:opacity-50 rounded-xl text-sm font-body flex items-center justify-center gap-2 transition-colors"
								>
									<Upload class="w-4 h-4" />
									Choose file
								</button>
								<button
									type="button"
									onclick={handleOpmlExport}
									disabled={opmlStatus === 'exporting' || !isTauri}
									class="flex-1 py-2.5 bg-muted hover:bg-muted/80 disabled:opacity-50 rounded-xl text-sm font-body flex items-center justify-center gap-2 transition-colors"
									title={!isTauri ? 'Export is available in the desktop app' : undefined}
								>
									<FileDown class="w-4 h-4" />
									Export OPML
								</button>
							</div>
							{#if opmlMessage}
								<p
									class="text-sm px-3 py-2 rounded-xl {opmlStatus === 'error'
										? 'bg-red-900/20 text-red-700 dark:text-red-300'
										: opmlStatus === 'success'
											? 'bg-primary/10 text-primary'
											: 'text-muted-foreground'}"
								>
									{opmlMessage}
								</p>
							{/if}
							{#if opmlStatus === 'importing' || opmlStatus === 'exporting'}
								<p class="text-xs text-muted-foreground italic">Please wait…</p>
							{/if}
						</div>
					{:else}
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
							<span class="text-xs font-heading font-bold text-muted-foreground ml-1 block mb-1">Type</span>
							<div class="flex flex-wrap gap-2 p-1 bg-muted rounded-2xl">
								<button
									type="button"
									onclick={() => (feedType = 'Update')}
									class="flex-1 min-w-0 py-2 rounded-xl text-xs font-body font-medium transition-all {feedType === 'Update'
										? 'bg-background shadow-sm ring-1 ring-border text-primary'
										: 'text-muted-foreground'}"
								>
									Update (6h)
								</button>
								<button
									type="button"
									onclick={() => (feedType = 'News')}
									class="flex-1 min-w-0 py-2 rounded-xl text-xs font-body font-medium transition-all {feedType === 'News'
										? 'bg-background shadow-sm ring-1 ring-border text-primary'
										: 'text-muted-foreground'}"
								>
									News (1D)
								</button>
								<button
									type="button"
									onclick={() => (feedType = 'Article')}
									class="flex-1 min-w-0 py-2 rounded-xl text-xs font-body font-medium transition-all {feedType === 'Article'
										? 'bg-background shadow-sm ring-1 ring-border text-primary'
										: 'text-muted-foreground'}"
								>
									Article (3D)
								</button>
								<button
									type="button"
									onclick={() => (feedType = 'Essay')}
									class="flex-1 min-w-0 py-2 rounded-xl text-xs font-body font-medium transition-all {feedType === 'Essay'
										? 'bg-background shadow-sm ring-1 ring-border text-primary'
										: 'text-muted-foreground'}"
								>
									Essay (1W)
								</button>
							</div>
						</div>
						{#if superfeedsExcludingAll.length > 0}
							<div class="space-y-2">
								<span class="text-xs font-heading font-bold text-muted-foreground ml-1 block mb-1">Add to superfeeds</span>
								<div class="space-y-1 max-h-28 overflow-y-auto p-1 rounded-2xl">
									{#each superfeedsExcludingAll as s (s.id)}
										<button
											type="button"
											onclick={() => toggleSuperfeed(s.id)}
											class="w-full flex items-center gap-3 px-3 py-2 rounded-xl text-left transition-colors hover:bg-background/50"
										>
											<span
												class="w-4 h-4 rounded-full border-2 shrink-0 flex items-center justify-center transition-colors {selectedSuperfeedIds.has(s.id)
													? 'border-primary bg-primary/10'
													: 'border-muted-foreground/50'}"
											>
												{#if selectedSuperfeedIds.has(s.id)}
													<span class="w-2 h-2 rounded-full bg-primary"></span>
												{/if}
											</span>
											<span class="text-sm font-body">{s.name}</span>
										</button>
									{/each}
								</div>
							</div>
						{/if}
					{/if}

					<p class="text-[10px] text-muted-foreground italic px-1">
						{tab === 'feed'
							? 'Adding a feed will start the syndicator engine to gather articles.'
							: tab === 'superfeed'
								? 'Superfeeds allow you to group multiple sources into a single view.'
								: 'Tags let you label articles for later filtering.'}
					</p>
					{/if}
				</div>

				{#if tab !== 'opml'}
				<button
					onclick={handleSubmit}
					disabled={!inputVal.trim()}
					class="w-full py-4 bg-primary hover:bg-primary-dark disabled:opacity-50 disabled:hover:bg-primary text-white font-heading font-bold beveled flex items-center justify-center gap-3 transition-all"
				>
					<Plus class="w-5 h-5" />
					<span>Confirm</span>
				</button>
				{/if}
			</div>
		</div>
	</div>
{/if}
