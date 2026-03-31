import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { open as shellOpen } from '@tauri-apps/plugin-shell';

/** Shown when the UI runs in a normal browser (Vite only); avoids calling into missing `__TAURI_INTERNALS__`. */
export const NEEDS_TAURI_DESKTOP_MESSAGE =
	'This app needs the Celadon desktop shell (Rust/SQLite). A plain browser at localhost has no backend. Run: npm run dev:desktop';

/** True only when the Tauri webview has injected IPC (`isTauri()` alone is not enough — it can be wrong in some browsers). */
export function hasTauriIpc(): boolean {
	if (typeof window === 'undefined') return false;
	const invokeFn = (window as unknown as { __TAURI_INTERNALS__?: { invoke?: unknown } }).__TAURI_INTERNALS__
		?.invoke;
	return typeof invokeFn === 'function';
}

function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	if (!hasTauriIpc()) {
		return Promise.reject(new Error(NEEDS_TAURI_DESKTOP_MESSAGE));
	}
	return tauriInvoke<T>(cmd, (args ?? {}) as never);
}

/** Open a URL in the system default browser (Tauri shell). No-op or fallback if not in Tauri. */
export async function openInBrowser(url: string): Promise<void> {
	if (!hasTauriIpc()) {
		window.open(url, '_blank');
		return;
	}
	try {
		await shellOpen(url);
	} catch {
		window.open(url, '_blank');
	}
}

export interface Article {
    id: number;
    url: string;
    name: string;
    published: string;
    expiry_at: string;
    read: boolean;
    description: string;
    feed: number;
    deleted: boolean;
}

export interface Feed {
    id: number;
    url: string;
    name: string;
    category: string;
    added: string;
    last_fetched: string;
    /** 0 = healthy, 1 = rate limited, 2–599 = most recent HTTP error code (for hover) */
    status: number;
    feed_type: 'News' | 'Article' | 'Essay' | 'Update';
    deleted: boolean;
    /** 0 = healthy, 1–2 = consecutive failures (yellow), 3 = dead (red) */
    consecutive_http_errors: number;
}

export interface Superfeed {
    id: number;
    name: string;
    deleted: boolean;
}

/** Default "All" superfeed id; every feed must belong to it. Not shown as add/remove in UI. */
export const ALL_SUPERFEED_ID = 1;

export interface Tag {
    id: number;
    name: string;
    deleted: boolean;
}

export type ReadFilter = 'Unread' | 'Read' | 'All';

export interface ArticleQuery {
    id?: number;
    filter: ReadFilter;
    num?: number;
    offset?: number;
}

// Tauri expects camelCase for multi-word argument keys (e.g. feedId, tagId, superfeedId).
// Single-word keys (id, name, filter, etc.) stay as-is.
export const getArticles = (id: number, filter: ReadFilter, num?: number, offset?: number) =>
    invoke<Article[]>('get_articles', { id, filter, num, offset });
export const getArticle = (id: number) => invoke<Article>('get_article', { id });

export const getSuperfeedArticles = (id: number, filter: ReadFilter, num?: number, offset?: number) =>
    invoke<Article[]>('get_superfeed_articles', { id, filter, num, offset });

export const getTagArticles = (id: number, filter: ReadFilter, num?: number, offset?: number) =>
    invoke<Article[]>('get_tagged_articles', { id, filter, num, offset });

export const getAllArticles = (filter: ReadFilter, num?: number, offset?: number) =>
    invoke<Article[]>('get_all_articles', { filter, num, offset });

export const searchArticles = (query: string, filter: ReadFilter, num?: number, offset?: number) =>
    invoke<Article[]>('search_articles', { query, filter, num, offset });

export const readArticle = (id: number) => invoke<void>('read_article', { id });
export const unreadArticle = (id: number) => invoke<void>('unread_article', { id });
export const deleteArticle = (id: number) => invoke<void>('delete_article', { id });
export const getArticleTags = (articleId: number) =>
	invoke<Tag[]>('get_article_tags', { id: articleId });

// Feed Commands
export const getAllFeeds = () => invoke<Feed[]>('get_all_feeds');
export const getFeed = (id: number) => invoke<Feed>('get_feed', { id });
export const searchFeeds = (query: string) => invoke<Feed[]>('search_feeds', { query });
export const getSuperfeedIdsForFeed = (feedId: number) =>
	invoke<number[]>('get_superfeed_ids_for_feed', { feedId });
export const updateFeedName = (id: number, name: string) =>
	invoke<void>('update_feed_name', { id, name });
export const updateFeedType = (id: number, feedType: 'News' | 'Article' | 'Essay' | 'Update') =>
	invoke<void>('update_feed_type', { id, feedType });
export const addFeedToSuperfeed = (feedId: number, superfeedId: number) =>
	invoke<void>('add_feed_to_superfeed', { feedId, superfeedId });
export const removeFeedFromSuperfeed = (feedId: number, superfeedId: number) =>
	invoke<void>('remove_feed_from_superfeed', { feedId, superfeedId });
export const deleteFeed = (id: number) => invoke<void>('delete_feed', { id });
export const readAllArticlesInFeed = (feedId: number) =>
	invoke<void>('read_all_articles_in_feed', { feedId });

// Superfeed Commands
export const getAllSuperfeeds = () => invoke<Superfeed[]>('get_all_superfeeds');
export const searchSuperfeeds = (query: string) => invoke<Superfeed[]>('search_superfeeds', { query });
export const renameSuperfeed = (id: number, name: string) =>
	invoke<void>('rename_superfeed', { id, name });
export const deleteSuperfeed = (id: number) => invoke<void>('delete_superfeed', { id });
export const getSuperfeedFeeds = (id: number, num?: number) =>
	invoke<Feed[]>('get_superfeed_feeds', { id, num });

// Tag Commands
export const getAllTags = () => invoke<Tag[]>('get_all_tags');
export const searchTags = (query: string) => invoke<Tag[]>('search_tags', { query });
export const renameTag = (id: number, name: string) => invoke<void>('rename_tag', { id, name });
export const deleteTag = (id: number) => invoke<void>('delete_tag', { id });

// Undo
export const undo = () => invoke<void>('undo');
export const clearUndo = () => invoke<void>('clear_undo');

// OPML
export const importOpml = (path: string) => invoke<void>('import_opml', { path });
export const importOpmlFromContent = (xml: string) => invoke<void>('import_opml_from_content', { xml });
export const exportOpml = (path: string) => invoke<void>('export_opml', { path });

export const createSuperfeed = (name: string) => invoke<void>('create_superfeed', { name });
export const createTag = (name: string) => invoke<number>('create_tag', { name });
export const tagArticle = (tagId: number, articleId: number) =>
	invoke<void>('tag_article', { tagId, articleId });
export const untagArticle = (tagId: number, articleId: number) =>
	invoke<void>('untag_article', { tagId, articleId });

// Syndication
export const addFeed = (url: string, feedType: string, superfeedId: number = 1) =>
    invoke<void>('add_feed', { url, superfeedId, feedType });
/** Re-fetch all feeds and ingest new articles (runs `clean_expired_articles` first on the backend). Background poller calls this only — it updates SQLite; the UI is not refetched until nav change or toolbar refresh. */
export const refreshAllFeeds = () => invoke<void>('refresh_all_feeds');

/** Mark unread articles past `expiry_at` as read (matches startup maintenance). Call before reloading lists so Unread reflects expiry. */
export const cleanExpiredArticles = () => invoke<void>('clean_expired_articles');

// App settings (SQLite via Tauri)
export interface AppSettings {
	theme: string;
	articleFullModeProxy: boolean;
}

export type UpdateAppSettingsPatch = {
	theme?: string;
	articleFullModeProxy?: boolean;
};

export const getAppSettings = () => invoke<AppSettings>('get_app_settings');

export const updateAppSettings = (patch: UpdateAppSettingsPatch) =>
	invoke<AppSettings>('update_app_settings', { patch });

export const getArticleProxyUrl = (documentUrl: string) =>
	invoke<string>('get_article_proxy_url', { documentUrl });
