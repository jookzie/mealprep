<script lang="ts">
	import { page } from '$app/state';
	import PageHeader from '$lib/components/PageHeader.svelte';
</script>

<PageHeader title={page.status === 404 ? 'Not found' : 'Something went wrong'} />
<p class="card">{page.error?.message}</p>
<div class="row">
	<a class="button" href="/calendar">Go to the calendar</a>
</div>
