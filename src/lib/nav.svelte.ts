import type { ReadFilter } from './api';

export type ViewType = 'All' | 'Feed' | 'Superfeed' | 'Tag' | 'Search' | 'Settings' | 'FeedsList' | 'SuperfeedsList' | 'TagsList' | 'SuperfeedFeeds' | 'FeedSuperfeeds';

export interface ViewState {
    type: ViewType;
    id?: number;
    name: string;
    query?: string;
    filter?: ReadFilter;
    offset?: number;
}

class NavigationStore {
    history = $state<ViewState[]>([{ type: 'All', name: 'All Articles', filter: 'Unread', offset: 0 }]);
    currentIndex = $state(0);

    current = $state<ViewState>({ type: 'All', name: 'All Articles', filter: 'Unread', offset: 0 });

    push(view: ViewState) {
        if (!view.filter) view.filter = 'Unread';
        if (view.offset === undefined) view.offset = 0;

        // If it's a new view, clear forward history
        if (this.currentIndex < this.history.length - 1) {
            this.history = this.history.slice(0, this.currentIndex + 1);
        }
        this.history = [...this.history, view];
        this.currentIndex = this.history.length - 1;
        this.current = view;
    }

    back() {
        if (this.currentIndex > 0) {
            this.currentIndex--;
            this.current = this.history[this.currentIndex];
        }
    }

    forward() {
        if (this.currentIndex < this.history.length - 1) {
            this.currentIndex++;
            this.current = this.history[this.currentIndex];
        }
    }

    get canGoBack() {
        return this.currentIndex > 0;
    }

    get canGoForward() {
        return this.currentIndex < this.history.length - 1;
    }

    updateFilter(filter: ReadFilter) {
        this.current.filter = filter;
        this.current.offset = 0; // Reset offset when filter changes
    }

    nextPage() {
        if (this.current.offset !== undefined) {
            this.current.offset += 50;
        }
    }

    reset() {
        this.push({ type: 'All', name: 'All Articles', filter: 'Unread', offset: 0 });
    }
}

export const nav = new NavigationStore();
