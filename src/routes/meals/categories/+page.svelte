<script lang="ts">
	import { createCategory, deleteCategory, renameCategory, runMutation } from '$lib/api';
	import CategoryManager from '$lib/components/CategoryManager.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { countByCategory } from '$lib/domain/category';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<PageHeader title="Meal categories" back="/meals" />

<CategoryManager
	categories={data.categories}
	counts={countByCategory(data.meals)}
	noun={{ one: 'meal', many: 'meals' }}
	onCreate={(name) =>
		runMutation(() => createCategory({ name, scope: 'meal' }), { success: 'Category created' })}
	onRename={(id, name) => runMutation(() => renameCategory(id, name), { success: 'Category renamed' })}
	onDelete={(id) => runMutation(() => deleteCategory(id), { success: 'Category deleted' })}
/>
