<script lang="ts">
	import type { Superfeed } from '$lib/api';
	import { Layers, Settings } from 'lucide-svelte';

	let { superfeed, onClick, onSettings } = $props<{
		superfeed: Superfeed;
		onClick: () => void;
		onSettings: () => void;
	}>();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	onclick={onClick}
	class="bg-card border border-border beveled p-4 cursor-pointer hover:shadow-lg transition-all duration-300 aspect-square flex flex-col justify-between group bg-gradient-to-br from-card to-muted/10"
>
	<div class="flex flex-col items-center justify-center flex-1 gap-3">
		<div class="p-3 bg-primary/10 rounded-2xl group-hover:bg-primary/20 transition-colors">
			<Layers class="w-8 h-8 text-primary" />
		</div>
		<h3 class="font-heading font-bold text-center line-clamp-2 px-2">
			{superfeed.name}
		</h3>
		<p class="text-[10px] text-muted-foreground uppercase tracking-widest font-bold">Superfeed</p>
	</div>

	<div class="flex items-center justify-end pt-3 border-t border-border">
		<button
			onclick={(e) => {
				e.stopPropagation();
				onSettings();
			}}
			class="p-1.5 hover:bg-muted rounded-lg transition-colors text-muted-foreground hover:text-primary"
		>
			<Settings class="w-3.5 h-3.5" />
		</button>
	</div>
</div>
