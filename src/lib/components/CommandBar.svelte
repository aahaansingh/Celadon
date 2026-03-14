<script lang="ts">
	import {
		Search,
		ChevronLeft,
		ChevronRight,
		Plus,
		Moon,
		Sun,
		Hash,
		Radio,
		Layers,
		RotateCcw,
		Rss
	} from 'lucide-svelte';
	import { nav } from '$lib/nav.svelte';
	import { decodeHtmlEntities } from '$lib/sanitizeHtml';
	import { clsx, type ClassValue } from 'clsx';
	import { twMerge } from 'tailwind-merge';
	import { theme } from '$lib/theme.svelte';
	import {
		searchFeeds,
		searchSuperfeeds,
		searchTags,
		searchArticles,
		type Feed,
		type Superfeed,
		type Tag,
		type ReadFilter
	} from '$lib/api';
	import { isSoloListCommand } from '$lib/commandBarCommands';

	function cn(...inputs: ClassValue[]) {
		return twMerge(clsx(inputs));
	}

	let { onAdd, onToggleDarkMode, darkMode, onRefresh, onOpenArticle } = $props<{
		onAdd: () => void;
		onToggleDarkMode: () => void;
		darkMode: boolean;
		onRefresh: () => void;
		onOpenArticle?: (id: number) => void;
	}>();

	let searchQuery = $state('');
	let suggestions = $state<{ id: number; name: string; type: 'feed' | 'superfeed' | 'tag' | 'article' }[]>([]);
	let showSuggestions = $state(false);
	let selectedIndex = $state(-1);

	async function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		const value = target.value;
		searchQuery = value;

		if (value.startsWith('\\fs:')) {
			const q = value.slice(4);
			const results = await searchSuperfeeds(q);
			suggestions = results.map((s: Superfeed) => ({
				id: s.id,
				name: s.name,
				type: 'superfeed' as const
			}));
			showSuggestions = suggestions.length > 0;
		} else if (value.startsWith('\\sf:')) {
			const q = value.slice(4);
			const results = await searchFeeds(q);
			suggestions = results.map((f: Feed) => ({ id: f.id, name: f.name, type: 'feed' as const }));
			showSuggestions = suggestions.length > 0;
		} else if (value.startsWith('\\f:')) {
			const q = value.slice(3);
			const results = await searchFeeds(q);
			suggestions = results.map((f: Feed) => ({ id: f.id, name: f.name, type: 'feed' as const }));
			showSuggestions = suggestions.length > 0;
		} else if (value.startsWith('\\s:')) {
			const q = value.slice(3);
			const results = await searchSuperfeeds(q);
			suggestions = results.map((s: Superfeed) => ({
				id: s.id,
				name: s.name,
				type: 'superfeed' as const
			}));
			showSuggestions = suggestions.length > 0;
		} else if (value.startsWith('\\t:')) {
			const q = value.slice(3);
			const results = await searchTags(q);
			suggestions = results.map((t: Tag) => ({ id: t.id, name: t.name, type: 'tag' as const }));
			showSuggestions = suggestions.length > 0;
		} else if (value.trim() === '\\fs' || value.trim() === '\\sf') {
			// Solo \fs / \sf: no suggestions (Enter will run parseAndExecute)
			showSuggestions = false;
			suggestions = [];
		} else if (value.trim().length >= 2) {
			// Plain search: show article suggestions as autocomplete (do not run for slash commands)
			if (value.trim().startsWith('\\')) {
				showSuggestions = false;
				suggestions = [];
			} else {
				try {
					const results = await searchArticles(value.trim(), 'Unread', 5, 0);
					suggestions = results.map((a) => ({ id: a.id, name: a.name, type: 'article' as const }));
					showSuggestions = suggestions.length > 0;
				} catch {
					suggestions = [];
					showSuggestions = false;
				}
			}
		} else {
			showSuggestions = false;
			suggestions = [];
		}
		selectedIndex = -1;
	}

	function applySuggestion(suggestion: (typeof suggestions)[0]) {
		let filter: ReadFilter = 'Unread';
		// Check if there was a chained command already typed
		if (searchQuery.includes('\\a')) filter = 'All';
		if (searchQuery.includes('\\r')) filter = 'Read';

		// \fs: → show feeds in selected superfeed
		if (searchQuery.startsWith('\\fs:') && suggestion.type === 'superfeed') {
			nav.push({ type: 'SuperfeedFeeds', id: suggestion.id, name: `Feeds in ${suggestion.name}` });
			searchQuery = '';
			showSuggestions = false;
			return;
		}
		// \sf: → show superfeeds for selected feed
		if (searchQuery.startsWith('\\sf:') && suggestion.type === 'feed') {
			nav.push({ type: 'FeedSuperfeeds', id: suggestion.id, name: `Superfeeds for ${suggestion.name}` });
			searchQuery = '';
			showSuggestions = false;
			return;
		}

		if (suggestion.type === 'feed') {
			nav.push({ type: 'Feed', id: suggestion.id, name: suggestion.name, filter });
		} else if (suggestion.type === 'superfeed') {
			nav.push({ type: 'Superfeed', id: suggestion.id, name: suggestion.name, filter });
		} else if (suggestion.type === 'tag') {
			nav.push({ type: 'Tag', id: suggestion.id, name: suggestion.name, filter });
		} else if (suggestion.type === 'article' && onOpenArticle) {
			onOpenArticle(suggestion.id);
		}

		searchQuery = '';
		showSuggestions = false;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown' && showSuggestions) {
			e.preventDefault();
			selectedIndex = (selectedIndex + 1) % suggestions.length;
		} else if (e.key === 'ArrowUp' && showSuggestions) {
			e.preventDefault();
			selectedIndex = (selectedIndex - 1 + suggestions.length) % suggestions.length;
		} else if (e.key === 'Enter') {
			const trimmed = searchQuery.trim();
			if (showSuggestions && suggestions.length > 0 && !isSoloListCommand(trimmed)) {
				// Search mode (\f:query, \s:query, \t:query, \fs:query, \sf:query, or plain search): use selected or first result
				const index = selectedIndex >= 0 ? selectedIndex : 0;
				applySuggestion(suggestions[index]);
			} else if (trimmed) {
				// No suggestions, or solo \f / \s / \t: run command (list view or exact match)
				parseAndExecute(trimmed);
				searchQuery = '';
				showSuggestions = false;
			}
		} else if (e.key === 'Escape') {
			showSuggestions = false;
		}
	}

	function parseAndExecute(raw: string) {
		const trimmed = raw.trim();
		let filter: ReadFilter = 'Unread';
		let cleanQuery = trimmed;
		const articleViewTypes = ['All', 'Feed', 'Superfeed', 'Tag', 'Search'] as const;
		function isArticleView(): boolean {
			return articleViewTypes.includes(nav.current.type as (typeof articleViewTypes)[number]);
		}

		// Solo filter commands: \a, \r, \u (no query) → update filter on current view only when in article view
		if (trimmed === '\\a' || trimmed === '\\r' || trimmed === '\\u') {
			if (isArticleView()) {
				if (trimmed === '\\a') filter = 'All';
				else if (trimmed === '\\r') filter = 'Read';
				else filter = 'Unread';
				nav.updateFilter(filter);
			}
			return;
		}

		// Prefix form: \a:query or \r:query or \u:query
		if (trimmed.startsWith('\\a:')) {
			filter = 'All';
			cleanQuery = trimmed.slice(3).trim();
			if (cleanQuery) {
				nav.push({ type: 'Search', name: `Search: ${cleanQuery}`, query: cleanQuery, filter });
				return;
			}
			if (isArticleView()) nav.updateFilter(filter);
			return;
		}
		if (trimmed.startsWith('\\r:')) {
			filter = 'Read';
			cleanQuery = trimmed.slice(3).trim();
			if (cleanQuery) {
				nav.push({ type: 'Search', name: `Search: ${cleanQuery}`, query: cleanQuery, filter });
				return;
			}
			if (isArticleView()) nav.updateFilter(filter);
			return;
		}
		if (trimmed.startsWith('\\u:')) {
			filter = 'Unread';
			cleanQuery = trimmed.slice(3).trim();
			if (cleanQuery) {
				nav.push({ type: 'Search', name: `Search: ${cleanQuery}`, query: cleanQuery, filter });
				return;
			}
			if (isArticleView()) nav.updateFilter(filter);
			return;
		}

		// Suffix form: [query]\a or [query]\r or [query]\u (query must be non-empty for search)
		if (trimmed.endsWith('\\a') && trimmed.length > 2) {
			filter = 'All';
			cleanQuery = trimmed.slice(0, -2).trim();
			if (cleanQuery) {
				nav.push({ type: 'Search', name: `Search: ${cleanQuery}`, query: cleanQuery, filter });
				return;
			}
		} else if (trimmed.endsWith('\\r') && trimmed.length > 2) {
			filter = 'Read';
			cleanQuery = trimmed.slice(0, -2).trim();
			if (cleanQuery) {
				nav.push({ type: 'Search', name: `Search: ${cleanQuery}`, query: cleanQuery, filter });
				return;
			}
		} else if (trimmed.endsWith('\\u') && trimmed.length > 2) {
			filter = 'Unread';
			cleanQuery = trimmed.slice(0, -2).trim();
			if (cleanQuery) {
				nav.push({ type: 'Search', name: `Search: ${cleanQuery}`, query: cleanQuery, filter });
				return;
			}
		}

		// List / resolve commands
		if (cleanQuery === '\\f') {
			nav.push({ type: 'FeedsList', name: 'All Feeds' });
		} else if (cleanQuery === '\\s') {
			nav.push({ type: 'SuperfeedsList', name: 'All Superfeeds' });
		} else if (cleanQuery === '\\t') {
			nav.push({ type: 'TagsList', name: 'All Tags' });
		} else if (cleanQuery === '\\fs') {
			nav.push({ type: 'SuperfeedsList', name: 'All Superfeeds' });
		} else if (cleanQuery === '\\sf') {
			nav.push({ type: 'FeedsList', name: 'All Feeds' });
		} else if (cleanQuery.startsWith('\\fs:')) {
			// No suggestion selected → go to superfeeds list so user can pick one
			nav.push({ type: 'SuperfeedsList', name: 'All Superfeeds' });
		} else if (cleanQuery.startsWith('\\sf:')) {
			// No suggestion selected → go to feeds list so user can pick one
			nav.push({ type: 'FeedsList', name: 'All Feeds' });
		} else if (cleanQuery.startsWith('\\f:')) {
			const name = cleanQuery.slice(3);
			nav.push({ type: 'Feed', name: `Feed: ${name}`, query: name, filter });
		} else if (cleanQuery.startsWith('\\s:')) {
			const name = cleanQuery.slice(3);
			nav.push({ type: 'Superfeed', name: `Superfeed: ${name}`, query: name, filter });
		} else if (cleanQuery.startsWith('\\t:')) {
			const name = cleanQuery.slice(3);
			nav.push({ type: 'Tag', name: `Tag: ${name}`, query: name, filter });
		} else {
			// Plain search
			nav.push({ type: 'Search', name: `Search: ${cleanQuery}`, query: cleanQuery, filter });
		}
	}

	const breadcrumbs = $derived.by(() => {
		const parts = ['Celadon'];
		if (nav.current.type !== 'All') {
			parts.push(nav.current.name);
		}
		if (nav.current.filter && nav.current.filter !== 'Unread') {
			parts.push(`(${nav.current.filter})`);
		}
		return parts;
	});
</script>

<div
	class="sticky top-0 z-50 bg-background/80 backdrop-blur-xl border-b border-border/50 shadow-sm"
>
	<div class="container mx-auto px-6 py-3">
		<div class="flex items-center gap-4">
			<!-- Logo & Breadcrumbs (content-sized, max-w-md so path truncates; search bar fills the rest) -->
			<div class="flex items-center gap-4 min-w-0 max-w-md shrink-0">
				<button
					type="button"
					class="relative group cursor-pointer"
					onclick={() => nav.reset()}
					aria-label="Home"
				>
					<div
						class="absolute -inset-1 bg-gradient-to-tr from-primary to-celadon-light rounded-lg blur opacity-25 group-hover:opacity-50 transition duration-500"
					></div>
					<img
						src="/celadon.svg"
						alt="Celadon"
						class="relative w-8 h-8 rounded-lg shadow-inner bg-background"
					/>
				</button>

				<nav class="flex items-center text-sm font-heading text-muted-foreground min-w-0 overflow-hidden">
					{#each breadcrumbs as part, i}
						<span
							class={cn(
								i === breadcrumbs.length - 1 && 'text-foreground font-bold',
								part !== 'Celadon' && !part.startsWith('(') && 'truncate max-w-[14rem] inline-block align-bottom'
							)}
						>
							{part === 'Celadon' || part.startsWith('(') ? part : decodeHtmlEntities(part)}
						</span>
						{#if i < breadcrumbs.length - 1}
							<span class="mx-2 opacity-30">/</span>
						{/if}
					{/each}
				</nav>
			</div>

			<!-- Navigation Controls -->
			<div class="flex gap-1 shrink-0">
				<button
					onclick={() => nav.back()}
					disabled={!nav.canGoBack}
					class="p-2 hover:bg-muted rounded-lg transition-all disabled:opacity-20"
				>
					<ChevronLeft class="w-4 h-4" />
				</button>
				<button
					onclick={() => nav.forward()}
					disabled={!nav.canGoForward}
					class="p-2 hover:bg-muted rounded-lg transition-all disabled:opacity-20"
				>
					<ChevronRight class="w-4 h-4" />
				</button>
			</div>

			<!-- Unified Search & Command Bar (dynamic: grows with space, shrinks when nav path needs room) -->
			<div class="flex-1 relative group min-w-0">
				<Search
					class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground group-focus-within:text-primary transition-colors"
				/>
				<input
					type="text"
					placeholder="Search or enter command"
					bind:value={searchQuery}
					oninput={handleInput}
					onkeydown={handleKeydown}
					onfocus={() => (showSuggestions = suggestions.length > 0)}
					onblur={() => setTimeout(() => (showSuggestions = false), 200)}
					class="w-full pl-10 pr-4 py-2 bg-muted/20 border border-primary/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary/30 focus:border-primary/50 transition-all font-body text-sm placeholder:text-muted-foreground/50"
				/>

				<!-- Suggestions Dropdown -->
				{#if showSuggestions && suggestions.length > 0}
					<div
						class="absolute top-full left-0 right-0 mt-2 bg-background border border-border rounded-xl shadow-2xl overflow-hidden z-50 animate-in fade-in slide-in-from-top-2 duration-200"
					>
						<div
							class="p-2 border-b border-border/50 bg-muted/30 text-[10px] uppercase tracking-wider font-bold text-muted-foreground flex justify-between items-center"
						>
							<span>Suggestions</span>
							<span class="flex gap-2">
								<kbd class="px-1.5 py-0.5 rounded bg-background border border-border shadow-sm"
									>↵</kbd
								>
								<kbd class="px-1.5 py-0.5 rounded bg-background border border-border shadow-sm"
									>↑↓</kbd
								>
							</span>
						</div>
						<div class="max-h-64 overflow-y-auto py-1">
							{#each suggestions as s, i}
								<button
									onclick={() => applySuggestion(s)}
									class={cn(
										'w-full px-4 py-2 text-left flex items-center gap-3 transition-colors',
										i === selectedIndex ? 'bg-primary/10 text-primary' : 'hover:bg-muted'
									)}
								>
									{#if s.type === 'feed'}
										<Radio class="w-4 h-4 opacity-50" />
									{:else if s.type === 'superfeed'}
										<Layers class="w-4 h-4 opacity-50" />
									{:else if s.type === 'tag'}
										<Hash class="w-4 h-4 opacity-50" />
									{:else}
										<Search class="w-4 h-4 opacity-50" />
									{/if}
									<span class="text-sm font-body font-medium line-clamp-1">{s.name}</span>
									<span class="ml-auto text-[10px] opacity-40 uppercase tracking-tighter shrink-0"
										>{s.type}</span
									>
								</button>
							{/each}
						</div>
					</div>
				{/if}
			</div>

			<!-- Actions -->
			<div class="flex items-center gap-3 shrink-0">
				<button
					onclick={() => nav.push({ type: 'FeedsList', name: 'Feeds' })}
					class="p-2 hover:bg-muted rounded-xl transition-all text-muted-foreground hover:text-primary"
					title="Feeds"
				>
					<Rss class="w-5 h-5" />
				</button>
				<button
					onclick={() => nav.push({ type: 'SuperfeedsList', name: 'All Superfeeds' })}
					class="p-2 hover:bg-muted rounded-xl transition-all text-muted-foreground hover:text-primary"
					title="All Superfeeds"
				>
					<Layers class="w-5 h-5" />
				</button>

				<button
					onclick={onRefresh}
					class="p-2 hover:bg-muted rounded-xl transition-all text-muted-foreground hover:text-foreground"
					title="Refresh"
				>
					<RotateCcw class="w-4 h-4" />
				</button>

				<button
					onclick={onToggleDarkMode}
					class="p-2 hover:bg-muted rounded-xl transition-all text-muted-foreground hover:text-foreground"
					title="Toggle Dark Mode"
				>
					{#if darkMode}
						<Sun class="w-4 h-4" />
					{:else}
						<Moon class="w-4 h-4" />
					{/if}
				</button>

				<button
					onclick={onAdd}
					class="bg-primary hover:bg-celadon-dark text-white px-5 py-2 rounded-xl flex items-center gap-2 font-heading text-sm shadow-lg shadow-primary/20 active:scale-95 transition-all"
				>
					<Plus class="w-4 h-4" />
					<span>Add</span>
				</button>
			</div>
		</div>
	</div>
</div>

<style>
	/* Subtle animations for the dropdown */
	:global(.animate-in) {
		animation-fill-mode: forwards;
	}
</style>
