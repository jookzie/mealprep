<script lang="ts">
	import { openUrl } from '@tauri-apps/plugin-opener';

	let { code }: { code?: string } = $props();

	const OFF = 'https://world.openfoodfacts.org';
</script>

<!-- Open Food Facts data is ODbL: it is credited wherever imported data is shown. -->
<p class="muted small">
	Data from
	<button type="button" class="link" onclick={() => openUrl(OFF)}>Open Food Facts</button>, under the
	Open Database License.
	{#if code}
		<button type="button" class="link" onclick={() => openUrl(`${OFF}/product/${code}`)}
			>View the source entry</button
		>.
	{/if}
</p>

<style>
	.link {
		display: inline;
		min-height: 0;
		padding: 0;
		border: none;
		background: none;
		color: var(--accent);
		text-decoration: underline;
		font-size: inherit;
	}
</style>
