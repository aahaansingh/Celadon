<script lang="ts">
	import type { Tag as TagType } from '$lib/api';
	import { Tag, Settings } from 'lucide-svelte';

	let { tag, onClick, onSettings, onContextMenu } = $props<{
		tag: TagType;
		onClick: () => void;
		onSettings: () => void;
		onContextMenu?: (e: MouseEvent) => void;
	}>();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	onclick={onClick}
	oncontextmenu={(e) => {
		e.preventDefault();
		onContextMenu?.(e);
	}}
	class="bg-card border border-border beveled p-4 cursor-pointer hover:shadow-lg transition-all duration-300 aspect-square flex flex-col justify-between group"
>
	<div class="flex flex-col items-center justify-center flex-1 gap-3">
		<div class="relative">
			<Tag class="w-10 h-10 text-muted-foreground/30" />
			<div
				class="absolute -top-1 -right-1 w-4 h-4 bg-celadon rounded-full border-2 border-card shadow-sm"
			></div>
		</div>
		<h3 class="font-heading font-bold text-center line-clamp-1 px-2">
			{tag.name}
		</h3>
		<p class="text-[10px] text-muted-foreground uppercase tracking-widest font-bold">Tag</p>
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
