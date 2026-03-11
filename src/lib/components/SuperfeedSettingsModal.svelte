<script lang="ts">
	import type { Superfeed } from '$lib/api';
	import { renameSuperfeed } from '$lib/api';
	import { X } from 'lucide-svelte';
	import { fade, scale } from 'svelte/transition';

	let { superfeed, onClose, onSaved } = $props<{
		superfeed: Superfeed;
		onClose: () => void;
		onSaved: () => void;
	}>();

	let name = $state(superfeed.name);
	let saving = $state(false);

	async function handleSave() {
		if (name.trim() === superfeed.name) {
			onClose();
			return;
		}
		saving = true;
		try {
			await renameSuperfeed(superfeed.id, name.trim());
			onSaved();
			onClose();
		} catch (e) {
			console.error(e);
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
			<h2 class="font-heading font-bold text-sm">Superfeed settings</h2>
			<button onclick={onClose} class="p-1 hover:bg-muted rounded-full transition-colors">
				<X class="w-4 h-4" />
			</button>
		</div>
		<div class="p-6 space-y-4">
			<div>
				<label for="superfeed-settings-name" class="text-xs font-heading font-bold text-muted-foreground block mb-1">Name</label>
				<input
					id="superfeed-settings-name"
					type="text"
					bind:value={name}
					class="w-full bg-muted border-none rounded-2xl px-4 py-2 font-body text-sm"
				/>
			</div>
			<div class="flex gap-2 pt-2">
				<button
					onclick={handleSave}
					disabled={saving}
					class="flex-1 py-2 bg-primary hover:bg-primary-dark disabled:opacity-50 text-white font-heading font-bold rounded-xl"
				>
					{saving ? 'Saving…' : 'Save'}
				</button>
				<button
					onclick={onClose}
					class="px-4 py-2 bg-muted hover:bg-muted/80 rounded-xl font-heading text-sm"
				>
					Cancel
				</button>
			</div>
		</div>
	</div>
</div>
