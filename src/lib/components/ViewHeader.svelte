<script lang="ts">
	import { nav } from '$lib/nav.svelte';
	import { Filter } from 'lucide-svelte';
</script>

{#if nav.current.type !== 'All' || (nav.current.filter && nav.current.filter !== 'Unread')}
	<div class="px-6 py-4 animate-in fade-in slide-in-from-top-4 duration-500">
		<div class="flex items-center gap-3 text-muted-foreground/60">
			<Filter class="w-4 h-4" />
			<h2 class="text-sm font-heading italic">
				Showing <span class="text-primary">{nav.current.filter}</span> articles
				{#if nav.current.type !== 'All'}
					in <span class="text-primary">{nav.current.name}</span>
				{/if}
			</h2>
		</div>
	</div>
{/if}
