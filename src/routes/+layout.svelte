<script>
	import { onMount } from 'svelte';
	import '../app.css';
	import HelpPopup from '$lib/components/HelpPopup.svelte';
	import { theme } from '$lib/theme.svelte';

	let { children } = $props();

	onMount(() => {
		void theme.hydrateFromBackend();
	});
</script>

{@render children()}
<HelpPopup />
