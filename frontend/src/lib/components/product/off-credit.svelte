<script lang="ts">
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
	import { Badge } from '$lib/components/ui/badge';

	// Requirements §6: imported product data is credited wherever it is shown, and the
	// detail view links back to the source entry. A product created by hand has no
	// sourceCode and shows nothing — its origin changes nothing else (PR-5).
	let { sourceCode, variant = 'badge' }: { sourceCode?: string; variant?: 'badge' | 'full' } =
		$props();

	const href = $derived(
		sourceCode ? `https://world.openfoodfacts.org/product/${sourceCode}` : undefined
	);
</script>

{#if sourceCode}
	{#if variant === 'full'}
		<p class="text-muted-foreground text-sm">
			Imported from
			<a
				{href}
				target="_blank"
				rel="noreferrer noopener"
				class="text-foreground inline-flex items-center gap-1 underline underline-offset-4"
			>
				Open Food Facts
				<ExternalLinkIcon class="size-3.5" />
			</a>
			· data under the Open Database License.
		</p>
	{:else}
		<Badge variant="secondary">Open Food Facts</Badge>
	{/if}
{/if}
