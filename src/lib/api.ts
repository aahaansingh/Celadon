import { invoke } from '@tauri-apps/api/tauri';

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
    healthy: boolean;
    feed_type: 'News' | 'Article' | 'Essay';
    deleted: boolean;
}

export interface Superfeed {
    id: number;
    name: string;
    deleted: boolean;
}

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

// Article Commands
export const getArticles = (id: number, filter: ReadFilter, num?: number, offset?: number) =>
    invoke<Article[]>('get_articles', { id, filter, num, offset });

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

// Feed Commands
export const getAllFeeds = () => invoke<Feed[]>('get_all_feeds');
export const getFeed = (id: number) => invoke<Feed>('get_feed', { id });
export const searchFeeds = (query: string) => invoke<Feed[]>('search_feeds', { query });

// Superfeed Commands
export const getAllSuperfeeds = () => invoke<Superfeed[]>('get_all_superfeeds');
export const searchSuperfeeds = (query: string) => invoke<Superfeed[]>('search_superfeeds', { query });

// Tag Commands
export const getAllTags = () => invoke<Tag[]>('get_all_tags');
export const searchTags = (query: string) => invoke<Tag[]>('search_tags', { query });

// Undo
export const undo = () => invoke<void>('undo');
export const clearUndo = () => invoke<void>('clear_undo');

// OPML
export const importOpml = (path: string) => invoke<void>('import_opml', { path });
export const exportOpml = (path: string) => invoke<void>('export_opml', { path });

export const createSuperfeed = (name: string) => invoke<void>('create_superfeed', { name });

// Syndication
export const addFeed = (url: string, feedType: string, superfeedId: number = 1) =>
    invoke<void>('add_feed', { url, superfeedId, feedType });
