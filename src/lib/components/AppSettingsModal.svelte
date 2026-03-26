<script lang="ts">
	import { onMount } from 'svelte';
	import { getAppSettings, updateAppSettings, type AppSettings } from '$lib/api';
	import { theme } from '$lib/theme.svelte';
	import { X } from 'lucide-svelte';
	import { fade, scale } from 'svelte/transition';

	let { onClose, onSaved } = $props<{
		onClose: () => void;
		onSaved?: (s: AppSettings) => void;
	}>();

	let loading = $state(true);
	let saving = $state(false);
	let themeChoice = $state<'light' | 'dark'>('dark');
	let articleProxy = $state(false);
	let loadError = $state<string | null>(null);

	onMount(() => {
		loadError = null;
		loading = true;
		getAppSettings()
			.then((s) => {
				themeChoice = s.theme === 'light' ? 'light' : 'dark';
				articleProxy = s.articleFullModeProxy;
			})
			.catch((e) => {
				loadError = e instanceof Error ? e.message : String(e);
			})
			.finally(() => {
				loading = false;
			});
	});

	async function handleSave() {
		saving = true;
		try {
			const s = await updateAppSettings({
				theme: themeChoice,
				articleFullModeProxy: articleProxy
			});
			theme.setFromThemeString(s.theme);
			onSaved?.(s);
			onClose();
		} catch (e) {
			loadError = e instanceof Error ? e.message : String(e);
		} finally {
			saving = false;
		}
	}
</script>

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
			<h2 class="font-heading font-bold text-sm">App settings</h2>
			<button onclick={onClose} class="p-1 hover:bg-muted rounded-full transition-colors">
				<X class="w-4 h-4" />
			</button>
		</div>
		<div class="p-6 space-y-6">
			{#if loadError && !loading}
				<p class="text-sm text-destructive font-body">{loadError}</p>
			{/if}
			{#if loading}
				<p class="text-sm text-muted-foreground font-body">Loading…</p>
			{:else}
				<div>
					<span class="text-xs font-heading font-bold text-muted-foreground block mb-2">Appearance</span>
					<div class="flex w-full gap-2 p-1 bg-muted rounded-2xl">
						<button
							type="button"
							onclick={() => (themeChoice = 'light')}
							class="flex-1 min-w-0 py-2 rounded-xl text-xs font-heading font-bold transition-all {themeChoice ===
							'light'
								? 'bg-background shadow-sm ring-1 ring-border text-primary'
								: 'text-muted-foreground hover:text-foreground'}"
						>
							Light
						</button>
						<button
							type="button"
							onclick={() => (themeChoice = 'dark')}
							class="flex-1 min-w-0 py-2 rounded-xl text-xs font-heading font-bold transition-all {themeChoice ===
							'dark'
								? 'bg-background shadow-sm ring-1 ring-border text-primary'
								: 'text-muted-foreground hover:text-foreground'}"
						>
							Dark
						</button>
					</div>
				</div>
				<div>
					<span class="text-xs font-heading font-bold text-muted-foreground block mb-2"
						>Full Mode article view</span
					>
					<div
						class="flex w-full gap-2 p-1 bg-muted rounded-2xl"
						role="radiogroup"
						aria-label="Full Mode proxy"
					>
						<button
							type="button"
							role="radio"
							aria-checked={articleProxy}
							onclick={() => (articleProxy = true)}
							class="flex-1 min-w-0 py-2 rounded-xl text-xs font-heading font-bold transition-all {articleProxy
								? 'bg-background shadow-sm ring-1 ring-border text-primary'
								: 'text-muted-foreground hover:text-foreground'}"
						>
							Proxy
						</button>
						<button
							type="button"
							role="radio"
							aria-checked={!articleProxy}
							onclick={() => (articleProxy = false)}
							class="flex-1 min-w-0 py-2 rounded-xl text-xs font-heading font-bold transition-all {!articleProxy
								? 'bg-background shadow-sm ring-1 ring-border text-primary'
								: 'text-muted-foreground hover:text-foreground'}"
						>
							Direct
						</button>
					</div>
					<p class="mt-2 text-xs text-muted-foreground font-body leading-relaxed">
						Proxy uses the in-app pipeline (adblock + embed-friendly HTML) via <code class="text-[10px]">celadon://</code>.
						Direct loads the original URL in the iframe.
					</p>
				</div>
				<div class="flex gap-2 pt-2">
					<button
						type="button"
						disabled={loading || saving}
						onclick={handleSave}
						class="flex-1 py-2 bg-primary hover:bg-primary-dark disabled:opacity-50 text-white font-heading font-bold text-sm rounded-lg transition-colors"
					>
						{saving ? 'Saving…' : 'Save'}
					</button>
					<button
						type="button"
						onclick={onClose}
						class="flex-1 py-2 bg-muted hover:bg-muted/80 rounded-lg font-heading text-sm transition-colors"
					>
						Cancel
					</button>
				</div>
			{/if}
		</div>
	</div>
</div>
