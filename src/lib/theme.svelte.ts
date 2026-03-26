import { getAppSettings, updateAppSettings, type AppSettings } from '$lib/api';

const THEME_LS = 'celadon-theme';
/** Once true, localStorage theme has been copied into DB (first run after upgrade). */
const MIGRATION_LS = 'celadon-settings-theme-migrated-v1';

class ThemeStore {
	darkMode = $state(true);

	constructor() {
		if (typeof window !== 'undefined') {
			const saved = localStorage.getItem(THEME_LS);
			this.darkMode = saved ? saved === 'dark' : true;
			this.apply();
		}
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

	/** Apply theme string from backend: `"dark"` | `"light"`. */
	setFromThemeString(theme: string) {
		this.darkMode = theme !== 'light';
		this.apply();
		if (typeof window !== 'undefined') {
			localStorage.setItem(THEME_LS, this.darkMode ? 'dark' : 'light');
		}
	}

	/** Load settings from DB (Tauri); one-time migration from localStorage → DB. No-op if invoke fails (e.g. browser-only dev). */
	async hydrateFromBackend(): Promise<void> {
		if (typeof window === 'undefined') return;
		try {
			if (!localStorage.getItem(MIGRATION_LS)) {
				const ls = localStorage.getItem(THEME_LS);
				const theme = ls === 'light' ? 'light' : 'dark';
				await updateAppSettings({ theme });
				localStorage.setItem(MIGRATION_LS, '1');
			}
			const s: AppSettings = await getAppSettings();
			this.setFromThemeString(s.theme);
		} catch {
			// Not in Tauri or DB unavailable — keep constructor/localStorage state
		}
	}

	toggle() {
		this.darkMode = !this.darkMode;
		this.apply();
		if (typeof window !== 'undefined') {
			localStorage.setItem(THEME_LS, this.darkMode ? 'dark' : 'light');
		}
		void updateAppSettings({ theme: this.darkMode ? 'dark' : 'light' }).catch(() => {});
	}
}

export const theme = new ThemeStore();
