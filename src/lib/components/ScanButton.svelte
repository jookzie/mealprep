<script lang="ts">
	import ScanBarcode from '@lucide/svelte/icons/scan-barcode';
	import { goto } from '$app/navigation';
	import { messageOf } from '$lib/api';
	import { scanner } from '$lib/scanner.svelte';
	import { toasts } from '$lib/toasts.svelte';

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
	<button type="button" onclick={start} disabled={scanner.active}>
		<ScanBarcode size={18} aria-hidden="true" />
		Scan
	</button>
{/if}
