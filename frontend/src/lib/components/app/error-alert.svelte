<script lang="ts">
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import * as Alert from '$lib/components/ui/alert';

	let { title = 'Something went wrong', message }: { title?: string; message: string } = $props();
</script>

<Alert.Root variant="destructive">
	<TriangleAlertIcon />
	<Alert.Title>{title}</Alert.Title>
	<Alert.Description>{message}</Alert.Description>
</Alert.Root>
