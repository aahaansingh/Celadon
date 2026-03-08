import { onMount } from "svelte";

class ThemeStore {
    darkMode = $state(true); // Default to dark mode

    constructor() {
        if (typeof window !== 'undefined') {
            const saved = localStorage.getItem('celadon-theme');
            this.darkMode = saved ? saved === 'dark' : true;
            this.apply();
        }
    }

    toggle() {
        this.darkMode = !this.darkMode;
        this.apply();
        localStorage.setItem('celadon-theme', this.darkMode ? 'dark' : 'light');
    }

    apply() {
        if (typeof document !== 'undefined') {
            if (this.darkMode) {
                document.documentElement.classList.add('dark');
            } else {
                document.documentElement.classList.remove('dark');
            }
        }
    }
}

export const theme = new ThemeStore();
