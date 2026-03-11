<script lang="ts">
	let { title, message, confirmLabel = 'Delete', onConfirm, onCancel } = $props<{
		title: string;
		message: string;
		confirmLabel?: string;
		onConfirm: () => void;
		onCancel: () => void;
	}>();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
	class="fixed inset-0 z-[270] bg-background/80 backdrop-blur-sm flex items-center justify-center p-6"
	onclick={onCancel}
	role="dialog"
	aria-modal="true"
	aria-labelledby="confirm-title"
>
	<div
		class="bg-card border border-border rounded-xl shadow-xl p-6 max-w-sm w-full"
		onclick={(e) => e.stopPropagation()}
	>
		<h2 id="confirm-title" class="font-heading font-bold text-sm mb-2">{title}</h2>
		<p class="text-sm text-muted-foreground font-body mb-6">{message}</p>
		<div class="flex gap-2 justify-end">
			<button
				type="button"
				onclick={onCancel}
				class="px-4 py-2 rounded-lg font-body text-sm bg-muted hover:bg-muted/80 transition-colors"
			>
				Cancel
			</button>
			<button
				type="button"
				onclick={() => {
					onConfirm();
					onCancel();
				}}
				class="px-4 py-2 rounded-lg font-body text-sm bg-red-600 hover:bg-red-700 text-white transition-colors"
			>
				{confirmLabel}
			</button>
		</div>
	</div>
</div>
