<script lang="ts">
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import { type MutationOptions, runMutation } from '$lib/api';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import { buttonVariants } from '$lib/components/ui/button';

	// Deletes are soft on the server, but they still remove the entity from every
	// view, so each one is confirmed. One component covers all five domains.
	let {
		name,
		description,
		action,
		result,
		label = 'Delete',
		variant = 'ghost',
		size = 'sm'
	}: {
		name: string;
		description?: string;
		action: () => Promise<unknown>;
		result?: MutationOptions;
		label?: string;
		variant?: 'ghost' | 'outline' | 'destructive';
		size?: 'sm' | 'default' | 'icon';
	} = $props();

	let open = $state(false);
	let pending = $state(false);

	async function confirm() {
		pending = true;
		await runMutation(action, { success: `${name} deleted`, ...result });
		pending = false;
		open = false;
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Trigger class={buttonVariants({ variant, size })}>
		<TrashIcon class="size-4" />
		{#if size !== 'icon'}{label}{/if}
	</AlertDialog.Trigger>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Delete {name}?</AlertDialog.Title>
			<AlertDialog.Description>
				{description ?? `${name} will be removed from every view. This cannot be undone here.`}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={pending}>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action
				disabled={pending}
				class={buttonVariants({ variant: 'destructive' })}
				onclick={(event) => {
					event.preventDefault();
					confirm();
				}}
			>
				{pending ? 'Deleting…' : 'Delete'}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
