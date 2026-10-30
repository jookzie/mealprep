<script lang="ts">
	import ScanBarcode from '@lucide/svelte/icons/scan-barcode';
	import { goto } from '$app/navigation';
	import { messageOf } from '$lib/api';
	import { scanner } from '$lib/scanner.svelte';
	import { toasts } from '$lib/toasts.svelte';

	let { compact = false }: { compact?: boolean } = $props();

	async function start() {
		try {
			const code = await scanner.scan();
			if (code !== null) await goto(`/products/import/${encodeURIComponent(code)}`);
		} catch (reason) {
			toasts.push('error', messageOf(reason));
		}
	}
</script>

<!-- Only phones have a camera to scan with; elsewhere the button is simply absent. -->
{#if scanner.available}
	<button
		type="button"
		class="scan"
		class:icon={compact}
		onclick={start}
		disabled={scanner.active}
		aria-label={compact ? 'Scan a barcode' : undefined}
	>
		<ScanBarcode size={20} strokeWidth={2.4} aria-hidden="true" />
		{#if !compact}Scan{/if}
	</button>
{/if}

<style>
	.scan {
		background: var(--protein);
		color: var(--on-macro);
	}

	.icon {
		width: 48px;
		height: 48px;
	}
</style>
