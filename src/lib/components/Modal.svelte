<script lang="ts">
	import { modalStore } from '$lib/stores/modal.svelte';

	function handleConfirm() {
		modalStore.info?.onConfirm?.();
		modalStore.hide();
	}

	function handleCancel() {
		modalStore.info?.onCancel?.();
		modalStore.hide();
	}
</script>

{#if modalStore.showModal && modalStore.info}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="w-full max-w-md rounded-lg bg-card p-6 shadow-lg">
			<h2 class="text-xl font-bold">{modalStore.info.title}</h2>
			<p class="mt-2 text-sm text-muted-foreground">{modalStore.info.content}</p>
			<div class="mt-4 flex justify-end gap-2">
				{#if modalStore.info.cancelText}
					<button onclick={handleCancel} class="rounded border px-4 py-2 hover:bg-muted">
						{modalStore.info.cancelText}
					</button>
				{/if}
				<button
					onclick={handleConfirm}
					class="rounded bg-primary px-4 py-2 text-primary-foreground hover:bg-primary/90"
				>
					{modalStore.info.confirmText ?? '确定'}
				</button>
			</div>
		</div>
	</div>
{/if}
