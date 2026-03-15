<script>
	import '../app.css';
	import HelpPopup from '$lib/components/HelpPopup.svelte';
	let { children } = $props();
</script>

{@render children()}
<HelpPopup />
