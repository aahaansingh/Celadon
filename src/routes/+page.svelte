<script lang="ts">
	import { onMount, tick, untrack } from 'svelte';
	import { nav } from '$lib/nav.svelte';
	import { theme } from '$lib/theme.svelte';
	import {
		getArticles,
		getSuperfeedArticles,
		getTagArticles,
		getAllFeeds,
		getAllSuperfeeds,
		getAllTags,
		getAllArticles,
		getArticle,
		searchArticles,
		searchFeeds,
		searchSuperfeeds,
		searchTags,
		readArticle,
		unreadArticle,
		addFeed,
		addFeedToSuperfeed,
		removeFeedFromSuperfeed,
		refreshAllFeeds,
		cleanExpiredArticles,
		createSuperfeed,
		createTag,
		tagArticle,
		untagArticle,
		getArticleTags,
		getSuperfeedIdsForFeed,
		getSuperfeedFeeds,
		deleteFeed,
		deleteSuperfeed,
		deleteTag,
		readAllArticlesInFeed,
		undo,
		getAppSettings,
		NEEDS_TAURI_DESKTOP_MESSAGE,
		type AppSettings,
		type Article,
		type Feed,
		type Superfeed,
		ALL_SUPERFEED_ID,
		type Tag as TagType
	} from '$lib/api';
	import CommandBar from '$lib/components/CommandBar.svelte';
	import ArticleCard from '$lib/components/ArticleCard.svelte';
	import ArticleViewer from '$lib/components/ArticleViewer.svelte';
	import FeedCard from '$lib/components/FeedCard.svelte';
	import SuperfeedCard from '$lib/components/SuperfeedCard.svelte';
	import TagCard from '$lib/components/TagCard.svelte';
	import AddDialog from '$lib/components/AddDialog.svelte';
	import ContextMenu from '$lib/components/ContextMenu.svelte';
	import FeedSettingsModal from '$lib/components/FeedSettingsModal.svelte';
	import SuperfeedSettingsModal from '$lib/components/SuperfeedSettingsModal.svelte';
	import TagSettingsModal from '$lib/components/TagSettingsModal.svelte';
	import AddTagToArticleModal from '$lib/components/AddTagToArticleModal.svelte';
	import AddFeedToSuperfeedModal from '$lib/components/AddFeedToSuperfeedModal.svelte';
	import ConfirmDeleteModal from '$lib/components/ConfirmDeleteModal.svelte';
	import AppSettingsModal from '$lib/components/AppSettingsModal.svelte';

	let articles = $state<Article[]>([]);
	let feeds = $state<Record<number, Feed>>({});
	let allFeeds = $state<Feed[]>([]);
	let allSuperfeeds = $state<Superfeed[]>([]);
	let allTags = $state<TagType[]>([]);
	let loading = $state(true);
	let loadingMore = $state(false);
	let selectedArticle = $state<Article | null>(null);
	let isAddOpen = $state(false);
	let endOfList = $state(false);
	let sentinel = $state<HTMLElement>();
	let errorMsg = $state<string | null>(null);
	let addingFeed = $state(false);
	let articleLoadError = $state<string | null>(null);
	let contextMenu = $state<{
		x: number;
		y: number;
		type: 'feed';
		feedId: number;
		feed?: Feed;
	} | {
		x: number;
		y: number;
		type: 'article';
		article: Article;
	} | {
		x: number;
		y: number;
		type: 'superfeed';
		superfeedId: number;
		superfeed?: Superfeed;
	} | {
		x: number;
		y: number;
		type: 'tag';
		tagId: number;
		tag?: TagType;
	} | null>(null);
	let pendingDelete = $state<{ type: 'feed' | 'superfeed' | 'tag'; id: number; name: string } | null>(null);
	let superfeedFeedsList = $state<Feed[]>([]);
	let feedSuperfeedsList = $state<Superfeed[]>([]);
	let feedSuperfeeds = $state<Record<number, { id: number; name: string }[]>>({});
	let articleTags = $state<Record<number, TagType[]>>({});
	let loadId = 0;
	let settingsTarget = $state<
		| { type: 'feed'; feed: Feed }
		| { type: 'superfeed'; superfeed: Superfeed }
		| { type: 'tag'; tag: TagType }
		| null
	>(null);
	let addTagTargetData = $state<{ article: Article; assignedTagIds: number[] } | null>(null);
	let addToSuperfeedTargetData = $state<{
		feedId: number;
		feedName: string;
		assignedSuperfeedIds: number[];
	} | null>(null);
	let appSettingsOpen = $state(false);
	/** Matches backend default (proxy on) until `getAppSettings` returns. */
	let articleFullModeProxy = $state(true);

	const PAGE_SIZE = 50;

	/** Refetch only the current view's article list + articleTags (no global lists). No-op for non-article views. */
	async function refetchCurrentViewArticlesOnly() {
		const type = nav.current.type;
		if (
			type !== 'All' &&
			type !== 'Feed' &&
			type !== 'Superfeed' &&
			type !== 'Tag' &&
			type !== 'Search'
		) {
			return;
		}
		const filter = nav.current.filter || 'Unread';
		let newArticles: Article[] = [];
		if (type === 'All') {
			newArticles = await getAllArticles(filter, PAGE_SIZE, 0);
		} else if (type === 'Feed' && nav.current.id) {
			newArticles = await getArticles(nav.current.id, filter, PAGE_SIZE, 0);
		} else if (type === 'Superfeed' && nav.current.id) {
			newArticles = await getSuperfeedArticles(nav.current.id, filter, PAGE_SIZE, 0);
		} else if (type === 'Tag' && nav.current.id) {
			newArticles = await getTagArticles(nav.current.id, filter, PAGE_SIZE, 0);
		} else if (type === 'Search' && nav.current.query) {
			newArticles = await searchArticles(nav.current.query, filter, PAGE_SIZE, 0);
		}
		endOfList = newArticles.length < PAGE_SIZE;
		articles = newArticles;
		const tagMap: Record<number, TagType[]> = {};
		await Promise.all(
			newArticles.map(async (a) => {
				const t = await getArticleTags(a.id);
				tagMap[a.id] = t;
			})
		);
		articleTags = tagMap;
	}

	/** Patch feed in allFeeds, feeds, and feedSuperfeeds (no API call). */
	function patchFeedInState(
		feedId: number,
		updates: { name?: string; feed_type?: 'News' | 'Article' | 'Essay' | 'Update'; superfeedIds?: number[] }
	) {
		const feed = allFeeds.find((f) => f.id === feedId) ?? feeds[feedId];
		if (!feed) return;
		const name = updates.name ?? feed.name;
		const feed_type = updates.feed_type ?? feed.feed_type;
		allFeeds = allFeeds.map((f) =>
			f.id === feedId ? { ...f, name, feed_type } : f
		);
		feeds = { ...feeds, [feedId]: { ...feed, name, feed_type } };
		if (updates.superfeedIds != null) {
			const list = updates.superfeedIds
				.map((id) => {
					const s = allSuperfeeds.find((x) => x.id === id);
					return s ? { id: s.id, name: s.name } : null;
				})
				.filter(Boolean) as { id: number; name: string }[];
			feedSuperfeeds = { ...feedSuperfeeds, [feedId]: list };
		}
	}

	/** Patch superfeed name in allSuperfeeds and in every feedSuperfeeds entry that references it. */
	function patchSuperfeedInState(superfeedId: number, updates: { name: string }) {
		allSuperfeeds = allSuperfeeds.map((s) =>
			s.id === superfeedId ? { ...s, name: updates.name } : s
		);
		const next: Record<number, { id: number; name: string }[]> = {};
		for (const [fid, list] of Object.entries(feedSuperfeeds)) {
			next[+fid] = list.map((x) =>
				x.id === superfeedId ? { ...x, name: updates.name } : x
			);
		}
		feedSuperfeeds = next;
	}

	/** Patch tag name in allTags and in every articleTags entry that contains this tag. */
	function patchTagInState(tagId: number, updates: { name: string }) {
		allTags = allTags.map((t) => (t.id === tagId ? { ...t, name: updates.name } : t));
		const next: Record<number, TagType[]> = {};
		for (const [aid, tags] of Object.entries(articleTags)) {
			next[+aid] = tags.map((t) => (t.id === tagId ? { ...t, name: updates.name } : t));
		}
		articleTags = next;
	}

	/** Refetch data for the current view only (articles for article views, or the list for list views). */
	async function refetchCurrentView() {
		const type = nav.current.type;
		if (
			type === 'All' ||
			type === 'Feed' ||
			type === 'Superfeed' ||
			type === 'Tag' ||
			type === 'Search'
		) {
			await refetchCurrentViewArticlesOnly();
			return;
		}
		if (type === 'FeedCard') {
			const [feedsRes, superfeedsRes, tagsRes] = await Promise.all([
				getAllFeeds(),
				getAllSuperfeeds(),
				getAllTags()
			]);
			allFeeds = feedsRes;
			allSuperfeeds = superfeedsRes;
			allTags = tagsRes;
			feeds = allFeeds.reduce(
				(acc: Record<number, Feed>, f: Feed) => ({ ...acc, [f.id]: f }),
				{}
			);
			const superfeedsMap: Record<number, { id: number; name: string }[]> = {};
			await Promise.all(
				allFeeds.map(async (f) => {
					const ids = await getSuperfeedIdsForFeed(f.id);
					superfeedsMap[f.id] = ids
						.map((id) => {
							const s = allSuperfeeds.find((s) => s.id === id);
							return s ? { id: s.id, name: s.name } : null;
						})
						.filter(Boolean) as { id: number; name: string }[];
				})
			);
			feedSuperfeeds = superfeedsMap;
			return;
		}
		if (type === 'SuperfeedFeeds' && nav.current.id != null) {
			const [list, superfeedsRes] = await Promise.all([
				getSuperfeedFeeds(nav.current.id),
				getAllSuperfeeds()
			]);
			superfeedFeedsList = list;
			const superfeedsMap: Record<number, { id: number; name: string }[]> = {};
			await Promise.all(
				list.map(async (f) => {
					const ids = await getSuperfeedIdsForFeed(f.id);
					superfeedsMap[f.id] = ids
						.map((id) => {
							const s = superfeedsRes.find((s) => s.id === id);
							return s ? { id: s.id, name: s.name } : null;
						})
						.filter(Boolean) as { id: number; name: string }[];
				})
			);
			feedSuperfeeds = superfeedsMap;
			return;
		}
		if (type === 'FeedSuperfeeds' && nav.current.id != null) {
			const [ids, superfeedsRes] = await Promise.all([
				getSuperfeedIdsForFeed(nav.current.id),
				getAllSuperfeeds()
			]);
			feedSuperfeedsList = superfeedsRes.filter((s) => ids.includes(s.id));
			return;
		}
	}

	async function loadData(append = false) {
		const myLoadId = ++loadId;
		if (append) {
			loadingMore = true;
		} else {
			loading = true;
			articles = [];
			endOfList = false;
			articleLoadError = null;
			// Don't mutate nav.current here — can re-trigger effect and race with article load.
			// offset is already 0 from initial state / push() / updateFilter().
		}

		try {
			if (nav.current.type === 'FeedCard') {
				if (!append) {
					const [feedsRes, superfeedsRes, tagsRes] = await Promise.all([
						getAllFeeds(),
						getAllSuperfeeds(),
						getAllTags()
					]);
					allFeeds = feedsRes;
					allSuperfeeds = superfeedsRes;
					allTags = tagsRes;
					feeds = allFeeds.reduce(
						(acc: Record<number, Feed>, f: Feed) => ({ ...acc, [f.id]: f }),
						{}
					);
					const superfeedsMap: Record<number, { id: number; name: string }[]> = {};
					await Promise.all(
						allFeeds.map(async (f) => {
							const ids = await getSuperfeedIdsForFeed(f.id);
							superfeedsMap[f.id] = ids
								.map((id) => {
									const s = allSuperfeeds.find((s) => s.id === id);
									return s ? { id: s.id, name: s.name } : null;
								})
								.filter(Boolean) as { id: number; name: string }[];
						})
					);
					feedSuperfeeds = superfeedsMap;
				}
				loading = false;
				loadingMore = false;
				return;
			}
			if (nav.current.type === 'SuperfeedFeeds' && nav.current.id != null) {
				const [list, superfeedsRes] = await Promise.all([
					getSuperfeedFeeds(nav.current.id),
					getAllSuperfeeds()
				]);
				superfeedFeedsList = list;
				const superfeedsMap: Record<number, { id: number; name: string }[]> = {};
				await Promise.all(
					list.map(async (f) => {
						const ids = await getSuperfeedIdsForFeed(f.id);
						superfeedsMap[f.id] = ids
							.map((id) => {
								const s = superfeedsRes.find((s) => s.id === id);
								return s ? { id: s.id, name: s.name } : null;
							})
							.filter(Boolean) as { id: number; name: string }[];
					})
				);
				feedSuperfeeds = superfeedsMap;
				loading = false;
				return;
			}
			superfeedFeedsList = [];

			if (nav.current.type === 'FeedSuperfeeds' && nav.current.id != null) {
				const [ids, superfeedsRes] = await Promise.all([
					getSuperfeedIdsForFeed(nav.current.id),
					getAllSuperfeeds()
				]);
				feedSuperfeedsList = superfeedsRes.filter((s) => ids.includes(s.id));
				loading = false;
				return;
			}
			feedSuperfeedsList = [];

			if (!append) {
				const [feedsRes, superfeedsRes, tagsRes] = await Promise.all([
					getAllFeeds(),
					getAllSuperfeeds(),
					getAllTags()
				]);
				allFeeds = feedsRes;
				allSuperfeeds = superfeedsRes;
				allTags = tagsRes;
				feeds = allFeeds.reduce(
					(acc: Record<number, Feed>, f: Feed) => ({ ...acc, [f.id]: f }),
					{}
				);
				const superfeedsMap: Record<number, { id: number; name: string }[]> = {};
				await Promise.all(
					allFeeds.map(async (f) => {
						const ids = await getSuperfeedIdsForFeed(f.id);
						superfeedsMap[f.id] = ids
							.map((id) => {
								const s = allSuperfeeds.find((s) => s.id === id);
								return s ? { id: s.id, name: s.name } : null;
							})
							.filter(Boolean) as { id: number; name: string }[];
					})
				);
				feedSuperfeeds = superfeedsMap;
			}

			let newArticles: Article[] = [];
			const filter = nav.current.filter || 'Unread';
			if (!append) {
				nav.current.offset = 0;
			}
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

			if (myLoadId !== loadId) return;
			if (append) {
				articles = [...articles, ...newArticles];
			} else {
				articles = newArticles;
			}

			// Load tags for displayed articles (batch for current page only)
			const articleList = newArticles;
			const tagMap: Record<number, TagType[]> = {};
			await Promise.all(
				articleList.map(async (a) => {
					const t = await getArticleTags(a.id);
					tagMap[a.id] = t;
				})
			);
			if (myLoadId !== loadId) return;
			articleTags = append ? { ...articleTags, ...tagMap } : tagMap;
			if (!append) {
				await tick();
				window.scrollTo(0, 0);
			}
		} catch (e) {
			const msg = e instanceof Error ? e.message : String(e);
			articleLoadError = msg;
			console.error('Failed to load articles:', e);
		} finally {
			loading = false;
			loadingMore = false;
		}
	}

	let lastRefreshTime = 0;

	/** Toolbar refresh: must not rely on `nav.forceRefresh()` — the article $effect only watches type/id/filter/query, so a shallow copy of `nav.current` does not re-run and `loadData` never fires. */
	async function handleToolbarRefresh() {
		try {
			await cleanExpiredArticles();
		} catch {
			/* DB maintenance best-effort */
		}
		try {
			await refreshAllFeeds();
		} catch {
			/* still reload from DB if syndication fails */
		}
		await loadData(false);
	}

	$effect(() => {
		// Track only type, id, filter, query so changing offset in loadData doesn't re-trigger
		const type = nav.current.type;
		const id = nav.current.id;
		const filter = nav.current.filter;
		const query = nav.current.query;
		untrack(() => {
			loadData().then(() => {
				// Run refresh at the next view change after an hour has passed (avoids interrupting article reading)
				const oneHourMs = 60 * 60 * 1000;
				if (lastRefreshTime > 0 && Date.now() - lastRefreshTime >= oneHourMs) {
					lastRefreshTime = Date.now();
					refreshAllFeeds()
						.then(() => refetchCurrentViewArticlesOnly())
						.catch(() => {});
				}
			});
		});
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

		// Once at startup: refresh due feeds so new articles appear
		refreshAllFeeds()
			.then(() => refetchCurrentViewArticlesOnly())
			.then(() => {
				lastRefreshTime = Date.now();
			})
			.catch(() => {});

		getAppSettings()
			.then((s) => {
				articleFullModeProxy = s.articleFullModeProxy;
			})
			.catch(() => {});

		// Undo: Cmd+Z (Mac) or Ctrl+Z (Windows)
		const handleKeydown = (e: KeyboardEvent) => {
			if (e.key === 'z' && (e.metaKey || e.ctrlKey)) {
				e.preventDefault();
				undo()
					.then(() => loadData())
					.catch(() => {});
			}
		};
		document.addEventListener('keydown', handleKeydown);

		return () => {
			observer.disconnect();
			document.removeEventListener('keydown', handleKeydown);
		};
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

	async function handleToggleRead(article: Article) {
		const newRead = !article.read;
		try {
			if (newRead) {
				await readArticle(article.id);
			} else {
				await unreadArticle(article.id);
			}
			// Update local state; do not remove from grid until view changes
			articles = articles.map((a) => (a.id === article.id ? { ...a, read: newRead } : a));
			if (selectedArticle?.id === article.id) {
				selectedArticle = { ...selectedArticle, read: newRead };
			}
		} catch (e) {
			console.error(e);
		}
	}

	async function handleAddFeed(url: string, feedType: 'News' | 'Article' | 'Essay' | 'Update' = 'News', selectedSuperfeedIds: number[] = []) {
		try {
			addingFeed = true;
			errorMsg = null;
			await addFeed(url, feedType, 1); // Always add to "All" first
			const feedsRes = await getAllFeeds();
			const newFeed = feedsRes.find((f) => f.url === url);
			if (newFeed) {
				for (const id of selectedSuperfeedIds) {
					if (id === 1) continue; // Already in All
					await addFeedToSuperfeed(newFeed.id, id);
				}
				allFeeds = feedsRes;
				feeds = { ...feeds, [newFeed.id]: newFeed };
				const superfeedList = selectedSuperfeedIds
					.map((id) => {
						const s = allSuperfeeds.find((x) => x.id === id);
						return s ? { id: s.id, name: s.name } : null;
					})
					.filter(Boolean) as { id: number; name: string }[];
				feedSuperfeeds = { ...feedSuperfeeds, [newFeed.id]: superfeedList };
				// If on All or a Superfeed that now includes this feed, refresh article list
				if (nav.current.type === 'All' || (nav.current.type === 'Superfeed' && nav.current.id != null && selectedSuperfeedIds.includes(nav.current.id))) {
					await refetchCurrentViewArticlesOnly();
				}
			}
		} catch (e: unknown) {
			const msg = e instanceof Error ? e.message : String(e);
			errorMsg = `Failed to add feed: ${msg}`;
			setTimeout(() => (errorMsg = null), 6000);
		} finally {
			addingFeed = false;
		}
	}

	async function handleCreateSuperfeed(name: string) {
		try {
			await createSuperfeed(name);
			allSuperfeeds = await getAllSuperfeeds();
		} catch (e) {
			console.error(e);
		}
	}

	async function handleCreateTag(name: string) {
		try {
			await createTag(name);
			allTags = await getAllTags();
		} catch (e) {
			console.error(e);
		}
	}

	async function handleSuperfeedModalApply(selectedSuperfeedIds: number[]) {
		const data = addToSuperfeedTargetData;
		if (!data) return;
		const { feedId, assignedSuperfeedIds } = data;
		const initial = new Set(assignedSuperfeedIds);
		const final = new Set(selectedSuperfeedIds);
		try {
			for (const id of final) {
				if (!initial.has(id)) await addFeedToSuperfeed(feedId, id);
			}
			for (const id of initial) {
				if (!final.has(id)) {
					if (id === ALL_SUPERFEED_ID) continue; // All feeds must stay in "All"
					await removeFeedFromSuperfeed(feedId, id);
				}
			}
			addToSuperfeedTargetData = null;
			errorMsg = null;
			// Patch feedSuperfeeds for this feed
			const list = selectedSuperfeedIds
				.map((id) => {
					const s = allSuperfeeds.find((x) => x.id === id);
					return s ? { id: s.id, name: s.name } : null;
				})
				.filter(Boolean) as { id: number; name: string }[];
			feedSuperfeeds = { ...feedSuperfeeds, [feedId]: list };
			// If we're on a Superfeed view that includes this feed, refresh article list
			if (nav.current.type === 'Superfeed' && nav.current.id != null && selectedSuperfeedIds.includes(nav.current.id)) {
				await refetchCurrentViewArticlesOnly();
			}
		} catch (e) {
			const msg = e instanceof Error ? e.message : String(e);
			errorMsg = `Couldn't update superfeeds: ${msg}`;
			console.error(e);
		}
	}

	async function handleAddFeedToSuperfeed(feedId: number, superfeedId: number) {
		try {
			await addFeedToSuperfeed(feedId, superfeedId);
			const s = allSuperfeeds.find((x) => x.id === superfeedId);
			if (s) {
				const current = feedSuperfeeds[feedId] ?? [];
				if (!current.some((e) => e.id === superfeedId)) {
					feedSuperfeeds = { ...feedSuperfeeds, [feedId]: [...current, { id: s.id, name: s.name }] };
				}
			}
			if (nav.current.type === 'Superfeed' && nav.current.id === superfeedId) {
				await refetchCurrentViewArticlesOnly();
			}
		} catch (e) {
			console.error(e);
		}
	}

	async function handleTagModalApply(selectedTagIds: number[]) {
		const data = addTagTargetData;
		if (!data) return;
		const articleId = data.article.id;
		const initial = new Set(data.assignedTagIds);
		const final = new Set(selectedTagIds);
		try {
			for (const id of final) {
				if (!initial.has(id)) await tagArticle(id, articleId);
			}
			for (const id of initial) {
				if (!final.has(id)) await untagArticle(id, articleId);
			}
			addTagTargetData = null;
			errorMsg = null;
		} catch (e) {
			const msg = e instanceof Error ? e.message : String(e);
			errorMsg = `Couldn't update tags: ${msg}`;
			console.error(e);
		}
	}

	async function handleCreateAndTagForArticle(name: string): Promise<number> {
		if (!addTagTargetData) throw new Error('No article');
		const newTagId = await createTag(name);
		allTags = await getAllTags();
		return newTagId;
	}
</script>

<div class="min-h-screen bg-background text-foreground transition-colors duration-500">
	<CommandBar
		onAdd={() => (isAddOpen = true)}
		onRefresh={() => void handleToolbarRefresh()}
		onOpenAppSettings={() => (appSettingsOpen = true)}
		onOpenArticle={(id) => {
			getArticle(id)
				.then((a) => {
					selectedArticle = a;
					readArticle(a.id);
					articles = articles.map((art) => (art.id === a.id ? { ...art, read: true } : art));
				})
				.catch((e) => console.error(e));
		}}
	/>

	<!-- Error toast -->
	{#if errorMsg}
		<div
			class="fixed top-20 left-1/2 -translate-x-1/2 z-[300] bg-red-900/90 text-white text-sm px-5 py-3 rounded-xl shadow-2xl border border-red-700/50 backdrop-blur-md max-w-md text-center"
		>
			{errorMsg}
		</div>
	{/if}

	<!-- Adding feed indicator -->
	{#if addingFeed}
		<div
			class="fixed top-20 left-1/2 -translate-x-1/2 z-[300] bg-primary/90 text-primary-foreground text-sm px-5 py-3 rounded-xl shadow-2xl backdrop-blur-md"
		>
			Syndicating feed… this may take a moment.
		</div>
	{/if}

	<main class="container mx-auto pb-10 pt-6">
		{#if loading}
			<div class="flex items-center justify-center h-64">
				<div class="animate-pulse text-primary font-heading text-sm">
					Gathering articles…
				</div>
			</div>
		{:else if nav.current.type === 'FeedCard' && nav.current.id != null}
			{@const feedForCard = allFeeds.find((f) => f.id === nav.current.id) ?? feeds[nav.current.id!]}
			{#if feedForCard}
				<div class="flex justify-center px-6">
					<div class="w-full min-w-72 max-w-sm">
						<FeedCard
							feed={feedForCard}
							superfeeds={(feedSuperfeeds[feedForCard.id] ?? []).filter((s) => s.id !== ALL_SUPERFEED_ID)}
							onSuperfeedClick={(id, name) => nav.push({ type: 'Superfeed', id, name })}
							onClick={() => nav.push({ type: 'Feed', id: feedForCard.id, name: feedForCard.name })}
							onSettings={() => (settingsTarget = { type: 'feed', feed: feedForCard })}
							onContextMenu={(e) => {
								contextMenu = { x: e.clientX, y: e.clientY, type: 'feed', feedId: feedForCard.id, feed: feedForCard };
							}}
						/>
					</div>
				</div>
			{:else}
				<div class="flex items-center justify-center h-64 text-muted-foreground">Loading feed…</div>
			{/if}
		{:else if nav.current.type === 'FeedsList'}
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 px-6">
				{#each allFeeds as f (f.id)}
					<FeedCard
						feed={f}
						superfeeds={(feedSuperfeeds[f.id] ?? []).filter((s) => s.id !== ALL_SUPERFEED_ID)}
						onSuperfeedClick={(id, name) => nav.push({ type: 'Superfeed', id, name })}
						onClick={() => nav.push({ type: 'Feed', id: f.id, name: f.name })}
						onSettings={() => (settingsTarget = { type: 'feed', feed: f })}
						onContextMenu={(e) => {
							contextMenu = { x: e.clientX, y: e.clientY, type: 'feed', feedId: f.id, feed: f };
						}}
					/>
				{/each}
			</div>
		{:else if nav.current.type === 'SuperfeedsList'}
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 px-6">
				{#each allSuperfeeds as s (s.id)}
					<SuperfeedCard
						superfeed={s}
						onClick={() => nav.push({ type: 'Superfeed', id: s.id, name: s.name })}
						onSettings={() => (settingsTarget = { type: 'superfeed', superfeed: s })}
						onContextMenu={(e) => {
							contextMenu = {
								x: e.clientX,
								y: e.clientY,
								type: 'superfeed',
								superfeedId: s.id,
								superfeed: s
							};
						}}
					/>
				{/each}
			</div>
		{:else if nav.current.type === 'TagsList'}
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 px-6">
				{#each allTags as t (t.id)}
					<TagCard
						tag={t}
						onClick={() => nav.push({ type: 'Tag', id: t.id, name: t.name })}
						onSettings={() => (settingsTarget = { type: 'tag', tag: t })}
						onContextMenu={(e) => {
							contextMenu = {
								x: e.clientX,
								y: e.clientY,
								type: 'tag',
								tagId: t.id,
								tag: t
							};
						}}
					/>
				{/each}
			</div>
		{:else if nav.current.type === 'SuperfeedFeeds' && nav.current.id != null}
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 px-6">
				{#each superfeedFeedsList as f (f.id)}
					<FeedCard
						feed={f}
						superfeeds={(feedSuperfeeds[f.id] ?? []).filter((s) => s.id !== ALL_SUPERFEED_ID)}
						onSuperfeedClick={(id, name) => nav.push({ type: 'Superfeed', id, name })}
						onClick={() => nav.push({ type: 'Feed', id: f.id, name: f.name })}
						onSettings={() => (settingsTarget = { type: 'feed', feed: f })}
						onContextMenu={(e) => {
							contextMenu = { x: e.clientX, y: e.clientY, type: 'feed', feedId: f.id, feed: f };
						}}
					/>
				{/each}
			</div>
		{:else if nav.current.type === 'FeedSuperfeeds' && nav.current.id != null}
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 px-6">
				{#each feedSuperfeedsList as s (s.id)}
					<SuperfeedCard
						superfeed={s}
						onClick={() => nav.push({ type: 'Superfeed', id: s.id, name: s.name })}
						onSettings={() => (settingsTarget = { type: 'superfeed', superfeed: s })}
						onContextMenu={(e) => {
							contextMenu = {
								x: e.clientX,
								y: e.clientY,
								type: 'superfeed',
								superfeedId: s.id,
								superfeed: s
							};
						}}
					/>
				{/each}
			</div>
		{:else if articles.length === 0}
			<div class="flex flex-col items-center justify-center h-64 text-muted-foreground/40">
				{#if articleLoadError}
					<p class="font-body text-lg text-red-600 dark:text-red-400 mb-2">
						Could not load articles: {articleLoadError}
					</p>
					{#if articleLoadError === NEEDS_TAURI_DESKTOP_MESSAGE}
						<p class="text-sm opacity-80 mb-4 max-w-md text-center">
							Plain <code class="text-xs">npm run dev</code> only starts Vite. Use
							<code class="text-xs">npm run dev:desktop</code> (or <code class="text-xs">npm run tauri:dev</code>) to open
							the Tauri window with SQLite and IPC.
						</p>
					{:else}
						<p class="text-sm opacity-70 mb-4">Check the browser console (F12 → Console) for details.</p>
					{/if}
					<button
						onclick={() => { articleLoadError = null; loadData(); }}
						class="px-4 py-2 bg-primary text-primary-foreground rounded-xl font-heading text-sm"
					>
						Retry
					</button>
				{:else}
					<p class="font-body italic text-lg opacity-50">The cabinet is empty.</p>
					<button
						onclick={() => nav.reset()}
						class="mt-4 text-xs uppercase tracking-widest hover:text-primary transition-colors"
					>
						Return Home
					</button>
				{/if}
			</div>
		{:else}
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 px-6">
				{#each articles as article (article.id)}
					<ArticleCard
						{article}
						feed={feeds[article.feed]}
						tags={articleTags[article.id] ?? []}
						onTagClick={(id, name) => nav.push({ type: 'Tag', id, name })}
						onClick={() => openArticle(article)}
						onToggleRead={() => handleToggleRead(article)}
						onAddTag={() => {}}
						onShowFeed={() =>
							nav.push({
								type: 'Feed',
								id: article.feed,
								name: feeds[article.feed]?.name || 'Feed'
							})}
						onContextMenu={(e) => {
							contextMenu = {
								x: e.clientX,
								y: e.clientY,
								type: 'article',
								article
							};
						}}
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
						Bottom of the Cabinet
					</div>
				{/if}
			</div>
		{/if}
	</main>

	<ArticleViewer
		article={selectedArticle}
		articleFullModeProxy={articleFullModeProxy}
		onClose={() => (selectedArticle = null)}
		onOpenAppSettings={() => (appSettingsOpen = true)}
	/>

	{#if appSettingsOpen}
		<AppSettingsModal
			onClose={() => (appSettingsOpen = false)}
			onSaved={(s: AppSettings) => {
				articleFullModeProxy = s.articleFullModeProxy;
			}}
		/>
	{/if}

	{#if contextMenu}
		{@const cm = contextMenu}
		{@const feedUrlToCopy = cm.type === 'feed' ? cm.feed?.url : undefined}
		{@const feedForMenu = cm.type === 'feed' ? cm : null}
		{@const superfeedForMenu = cm.type === 'superfeed' ? cm : null}
		{@const tagForMenu = cm.type === 'tag' ? cm : null}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="fixed inset-0 z-[240]"
			onclick={() => (contextMenu = null)}
			oncontextmenu={(e) => {
				e.preventDefault();
				contextMenu = null;
			}}
		></div>
		<ContextMenu
			x={cm.x}
			y={cm.y}
			type={cm.type}
			feed={cm.type === 'feed' ? cm.feed : undefined}
			feedId={cm.type === 'feed' ? cm.feedId : undefined}
			superfeedId={cm.type === 'superfeed' ? cm.superfeedId : undefined}
			superfeed={cm.type === 'superfeed' ? cm.superfeed : undefined}
			tagId={cm.type === 'tag' ? cm.tagId : undefined}
			tag={cm.type === 'tag' ? cm.tag : undefined}
			articleActions={
				cm.type === 'article'
					? (() => {
							const a = cm.article;
							return {
								onToggleRead: () => handleToggleRead(a),
								onAddTag: async () => {
									contextMenu = null;
									try {
										const tagModels = await getArticleTags(a.id);
										addTagTargetData = {
											article: a,
											assignedTagIds: tagModels.map((t) => t.id)
										};
										errorMsg = null;
									} catch (e) {
										const msg = e instanceof Error ? e.message : String(e);
										errorMsg = `Couldn't load tags for this article. ${msg}`;
										addTagTargetData = { article: a, assignedTagIds: [] };
									}
								},
								onShowFeed: () =>
									nav.push({
										type: 'Feed',
										id: a.feed,
										name: feeds[a.feed]?.name || 'Feed'
									}),
								onShowFeedCard: () =>
									nav.push({
										type: 'FeedCard',
										id: a.feed,
										name: feeds[a.feed]?.name || 'Feed'
									}),
								read: a.read
							};
						})()
					: undefined
			}
			onCopyFeedLink={
				feedUrlToCopy
					? () => {
							navigator.clipboard.writeText(feedUrlToCopy);
						}
					: undefined
			}
			onShowSuperfeeds={
				feedForMenu
					? () => {
							const feedId = feedForMenu.feedId;
							const feedName = feedForMenu.feed?.name ?? feeds[feedId]?.name ?? 'Feed';
							contextMenu = null;
							nav.push({ type: 'FeedSuperfeeds', id: feedId, name: `Superfeeds for ${feedName}` });
						}
					: undefined
			}
			onOpenAddToSuperfeed={
				feedForMenu
					? async (feedId) => {
							const feedName = feedForMenu.feed?.name ?? feeds[feedId]?.name ?? 'Feed';
							contextMenu = null;
							try {
								const ids = await getSuperfeedIdsForFeed(feedId);
								addToSuperfeedTargetData = { feedId, feedName, assignedSuperfeedIds: ids };
								errorMsg = null;
							} catch (e) {
								const msg = e instanceof Error ? e.message : String(e);
								errorMsg = `Couldn't load superfeeds for this feed. ${msg}`;
								addToSuperfeedTargetData = { feedId, feedName, assignedSuperfeedIds: [] };
							}
						}
					: undefined
			}
			onMarkAllReadFeed={
				feedForMenu
					? (feedId) => {
							contextMenu = null;
							readAllArticlesInFeed(feedId).then(() => refetchCurrentViewArticlesOnly()).catch((e) => console.error(e));
						}
					: undefined
			}
			onDeleteFeed={
				feedForMenu
					? (feedId) => {
							const name = feedForMenu.feed?.name ?? feeds[feedId]?.name ?? 'Feed';
							pendingDelete = { type: 'feed', id: feedId, name };
						}
					: undefined
			}
			onShowFeedsInSuperfeed={
				superfeedForMenu
					? (superfeedId) => {
							const name = superfeedForMenu.superfeed?.name ?? allSuperfeeds.find((s) => s.id === superfeedId)?.name ?? 'Superfeed';
							contextMenu = null;
							nav.push({ type: 'SuperfeedFeeds', id: superfeedId, name: `Feeds in ${name}` });
						}
					: undefined
			}
			onDeleteSuperfeed={
				superfeedForMenu && superfeedForMenu.superfeedId !== ALL_SUPERFEED_ID
					? (superfeedId) => {
							const name = superfeedForMenu.superfeed?.name ?? allSuperfeeds.find((s) => s.id === superfeedId)?.name ?? 'Superfeed';
							pendingDelete = { type: 'superfeed', id: superfeedId, name };
						}
					: undefined
			}
			onDeleteTag={
				tagForMenu
					? (tagId) => {
							const name = tagForMenu.tag?.name ?? allTags.find((t) => t.id === tagId)?.name ?? 'Tag';
							pendingDelete = { type: 'tag', id: tagId, name };
						}
					: undefined
			}
			onClose={() => (contextMenu = null)}
		/>
	{/if}

	{#if addTagTargetData}
		<AddTagToArticleModal
			article={addTagTargetData.article}
			tags={allTags}
			assignedTagIds={addTagTargetData.assignedTagIds}
			onApply={handleTagModalApply}
			onCreateAndTag={handleCreateAndTagForArticle}
			onClose={() => (addTagTargetData = null)}
		/>
	{/if}

	{#if addToSuperfeedTargetData}
		<AddFeedToSuperfeedModal
			feedId={addToSuperfeedTargetData.feedId}
			feedName={addToSuperfeedTargetData.feedName}
			superfeeds={allSuperfeeds.filter((s) => s.id !== ALL_SUPERFEED_ID)}
			assignedSuperfeedIds={addToSuperfeedTargetData.assignedSuperfeedIds}
			onApply={handleSuperfeedModalApply}
			onClose={() => (addToSuperfeedTargetData = null)}
		/>
	{/if}

	{#if pendingDelete}
		<ConfirmDeleteModal
			title="Delete {pendingDelete.type}"
			message="Are you sure you want to delete {pendingDelete.name}? You can undo with Cmd+Z (Mac) or Ctrl+Z (Windows)."
			confirmLabel="Delete"
			onConfirm={async () => {
				const p = pendingDelete;
				pendingDelete = null;
				if (!p) return;
				const scrollY = window.scrollY;
				const scrollX = window.scrollX;
				try {
					if (p.type === 'feed') {
						await deleteFeed(p.id);
						// Navigate off if we're viewing the deleted feed
						if (nav.current.type === 'Feed' && nav.current.id === p.id) {
							nav.push({ type: 'FeedsList', name: 'Feeds' });
						}
						// Remove from local lists so UI updates immediately
						allFeeds = allFeeds.filter((f) => f.id !== p.id);
						feeds = allFeeds.reduce(
							(acc: Record<number, Feed>, f: Feed) => ({ ...acc, [f.id]: f }),
							{}
						);
						superfeedFeedsList = superfeedFeedsList.filter((f) => f.id !== p.id);
						feedSuperfeeds = Object.fromEntries(
							Object.entries(feedSuperfeeds).filter(([k]) => +k !== p.id)
						) as Record<number, { id: number; name: string }[]>;
					} else if (p.type === 'superfeed') {
						await deleteSuperfeed(p.id);
						if (nav.current.type === 'Superfeed' && nav.current.id === p.id) {
							nav.push({ type: 'SuperfeedsList', name: 'Superfeeds' });
						}
						allSuperfeeds = allSuperfeeds.filter((s) => s.id !== p.id);
						feedSuperfeedsList = feedSuperfeedsList.filter((s) => s.id !== p.id);
						// Remove this superfeed from every feed's feedSuperfeeds entry
						const next: Record<number, { id: number; name: string }[]> = {};
						for (const [fid, list] of Object.entries(feedSuperfeeds)) {
							next[+fid] = list.filter((x) => x.id !== p.id);
						}
						feedSuperfeeds = next;
					} else if (p.type === 'tag') {
						await deleteTag(p.id);
						if (nav.current.type === 'Tag' && nav.current.id === p.id) {
							nav.push({ type: 'TagsList', name: 'Tags' });
						}
						allTags = allTags.filter((t) => t.id !== p.id);
						// Remove this tag from every article's articleTags entry
						const next: Record<number, TagType[]> = {};
						for (const [aid, tags] of Object.entries(articleTags)) {
							next[+aid] = tags.filter((t) => t.id !== p.id);
						}
						articleTags = next;
					}
					await refetchCurrentView();
					await tick();
					window.scrollTo(scrollX, scrollY);
				} catch (e) {
					errorMsg = e instanceof Error ? e.message : String(e);
				}
			}}
			onCancel={() => (pendingDelete = null)}
		/>
	{/if}

	{#if settingsTarget?.type === 'feed' && settingsTarget.feed}
		{@const feedId = settingsTarget.feed.id}
		<FeedSettingsModal
			feed={settingsTarget.feed}
			superfeeds={allSuperfeeds.filter((s) => s.id !== ALL_SUPERFEED_ID)}
			onClose={() => (settingsTarget = null)}
			onSaved={(updated) => patchFeedInState(feedId, updated)}
		/>
	{:else if settingsTarget?.type === 'superfeed' && settingsTarget.superfeed}
		{@const superfeedId = settingsTarget.superfeed.id}
		<SuperfeedSettingsModal
			superfeed={settingsTarget.superfeed}
			onClose={() => (settingsTarget = null)}
			onSaved={(updated) => patchSuperfeedInState(superfeedId, updated)}
		/>
	{:else if settingsTarget?.type === 'tag' && settingsTarget.tag}
		{@const tagId = settingsTarget.tag.id}
		<TagSettingsModal
			tag={settingsTarget.tag}
			onClose={() => (settingsTarget = null)}
			onSaved={(updated) => patchTagInState(tagId, updated)}
		/>
	{/if}

	<AddDialog
		isOpen={isAddOpen}
		onClose={() => (isAddOpen = false)}
		onAddFeed={handleAddFeed}
		onCreateSuperfeed={handleCreateSuperfeed}
		onCreateTag={handleCreateTag}
		onOpmlComplete={() => loadData()}
		superfeeds={allSuperfeeds}
	/>
</div>

<style>
	/* Page-specific refinements */
	:global(body) {
		overflow-y: scroll; /* Prevent layout shift on load */
	}
</style>
