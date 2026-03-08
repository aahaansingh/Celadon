<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { nav } from '$lib/nav.svelte';
	import { theme } from '$lib/theme.svelte';
	import {
		getArticles,
		getSuperfeedArticles,
		getTagArticles,
		getAllFeeds,
		getAllArticles,
		searchArticles,
		readArticle,
		addFeed,
		createSuperfeed,
		type Article,
		type Feed,
		type Superfeed,
		type Tag as TagType
	} from '$lib/api';
	import CommandBar from '$lib/components/CommandBar.svelte';
	import ArticleCard from '$lib/components/ArticleCard.svelte';
	import ViewHeader from '$lib/components/ViewHeader.svelte';
	import ArticleViewer from '$lib/components/ArticleViewer.svelte';
	import FeedCard from '$lib/components/FeedCard.svelte';
	import SuperfeedCard from '$lib/components/SuperfeedCard.svelte';
	import TagCard from '$lib/components/TagCard.svelte';
	import AddDialog from '$lib/components/AddDialog.svelte';

	let articles = $state<Article[]>([]);
	let feeds = $state<Record<number, Feed>>({});
	let allFeeds = $state<Feed[]>([]);
	let loading = $state(true);
	let loadingMore = $state(false);
	let selectedArticle = $state<Article | null>(null);
	let isAddOpen = $state(false);
	let endOfList = $state(false);
	let sentinel = $state<HTMLElement>();

	const PAGE_SIZE = 50;

	async function loadData(append = false) {
		if (append) {
			loadingMore = true;
		} else {
			loading = true;
			articles = [];
			endOfList = false;
			nav.current.offset = 0;
		}

		try {
			if (!append) {
				allFeeds = await getAllFeeds();
				feeds = allFeeds.reduce(
					(acc: Record<number, Feed>, f: Feed) => ({ ...acc, [f.id]: f }),
					{}
				);
			}

			let newArticles: Article[] = [];
			const filter = nav.current.filter || 'Unread';
			const offset = nav.current.offset || 0;

			if (nav.current.type === 'All') {
				newArticles = await getAllArticles(filter, PAGE_SIZE, offset);
			} else if (nav.current.type === 'Feed' && nav.current.id) {
				newArticles = await getArticles(nav.current.id, filter, PAGE_SIZE, offset);
			} else if (nav.current.type === 'Superfeed' && nav.current.id) {
				newArticles = await getSuperfeedArticles(nav.current.id, filter, PAGE_SIZE, offset);
			} else if (nav.current.type === 'Tag' && nav.current.id) {
				newArticles = await getTagArticles(nav.current.id, filter, PAGE_SIZE, offset);
			} else if (nav.current.type === 'Search' && nav.current.query) {
				newArticles = await searchArticles(nav.current.query, filter, PAGE_SIZE, offset);
			}

			if (newArticles.length < PAGE_SIZE) {
				endOfList = true;
			}

			if (append) {
				articles = [...articles, ...newArticles];
			} else {
				articles = newArticles;
			}
		} catch (e) {
			console.error(e);
		} finally {
			loading = false;
			loadingMore = false;
		}
	}

	$effect(() => {
		// Watch nav.current for changes (type, id, filter, query)
		// but ignore offset changes to handle them with infinite scroll logic manually
		const { type, id, filter, query } = nav.current;
		untrack(() => loadData());
	});

	// Handle infinite scroll
	let observer: IntersectionObserver;

	onMount(() => {
		observer = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting && !loading && !loadingMore && !endOfList) {
					nav.nextPage();
					loadData(true);
				}
			},
			{ threshold: 0.1 }
		);

		return () => observer.disconnect();
	});

	$effect(() => {
		if (sentinel && observer) {
			observer.observe(sentinel);
		}
	});

	function openArticle(article: Article) {
		selectedArticle = article;
		readArticle(article.id);
		article.read = true;
	}

	async function handleAddFeed(url: string) {
		try {
			loading = true;
			await addFeed(url, 'News');
			await loadData();
		} catch (e) {
			console.error(e);
		} finally {
			loading = false;
		}
	}

	async function handleCreateSuperfeed(name: string) {
		try {
			await createSuperfeed(name);
			await loadData();
		} catch (e) {
			console.error(e);
		}
	}
</script>

<div class="min-h-screen bg-background text-foreground transition-colors duration-500">
	<CommandBar
		onAdd={() => (isAddOpen = true)}
		onToggleDarkMode={() => theme.toggle()}
		darkMode={theme.darkMode}
	/>

	<main class="container mx-auto pb-10">
		<ViewHeader />

		{#if loading}
			<div class="flex items-center justify-center h-64">
				<div class="animate-pulse text-primary font-heading uppercase tracking-widest text-sm">
					Gathering Articles...
				</div>
			</div>
		{:else if nav.current.type === 'FeedsList'}
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 px-6">
				{#each allFeeds as f (f.id)}
					<FeedCard
						feed={f}
						onClick={() => nav.push({ type: 'Feed', id: f.id, name: f.name })}
						onSettings={() => {}}
					/>
				{/each}
			</div>
		{:else if articles.length === 0}
			<div class="flex flex-col items-center justify-center h-64 text-muted-foreground/40">
				<p class="font-body italic text-lg opacity-50">The garden is empty.</p>
				<button
					onclick={() => nav.reset()}
					class="mt-4 text-xs uppercase tracking-widest hover:text-primary transition-colors"
				>
					Return Home
				</button>
			</div>
		{:else}
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 px-6">
				{#each articles as article (article.id)}
					<ArticleCard
						{article}
						feed={feeds[article.feed]}
						onClick={() => openArticle(article)}
						onToggleRead={() => {}}
						onAddTag={() => {}}
						onShowFeed={() =>
							nav.push({
								type: 'Feed',
								id: article.feed,
								name: feeds[article.feed]?.name || 'Feed'
							})}
					/>
				{/each}
			</div>

			<!-- Infinite Scroll Sentinel -->
			<div bind:this={sentinel} class="h-20 flex items-center justify-center w-full mt-10">
				{#if loadingMore}
					<div class="flex gap-1 animate-pulse">
						<div class="w-1 h-1 bg-primary rounded-full"></div>
						<div class="w-1 h-1 bg-primary rounded-full"></div>
						<div class="w-1 h-1 bg-primary rounded-full"></div>
					</div>
				{:else if endOfList && articles.length > 0}
					<div class="text-[10px] uppercase tracking-[0.3em] text-muted-foreground/30 font-bold">
						End of the trail
					</div>
				{/if}
			</div>
		{/if}
	</main>

	<ArticleViewer article={selectedArticle} onClose={() => (selectedArticle = null)} />

	<AddDialog
		isOpen={isAddOpen}
		onClose={() => (isAddOpen = false)}
		onAddFeed={handleAddFeed}
		onCreateSuperfeed={handleCreateSuperfeed}
	/>
</div>

<style>
	/* Page-specific refinements */
	:global(body) {
		overflow-y: scroll; /* Prevent layout shift on load */
	}
</style>
