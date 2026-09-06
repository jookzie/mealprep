<script lang="ts">
	import { page } from '$app/state';
	import ErrorAlert from '$lib/components/app/error-alert.svelte';
	import { Button } from '$lib/components/ui/button';
</script>

<div class="space-y-4">
	<ErrorAlert
		title={page.status === 404 ? 'Not found' : 'Something went wrong'}
		message={page.error?.message ?? 'The page could not be loaded.'}
	/>
	<Button href="/calendar" variant="outline">Back to the calendar</Button>
</div>
