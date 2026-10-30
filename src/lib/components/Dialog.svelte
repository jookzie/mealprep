<script lang="ts">
	import X from '@lucide/svelte/icons/x';
	import type { Snippet } from 'svelte';
	import { SHEET_CLOSE_PX } from '$lib/domain/sections';

	let {
		open = $bindable(false),
		title,
		subtitle,
		tall = false,
		onclose,
		children,
	}: {
		open: boolean;
		title: string;
		subtitle?: string;
		/** Most of the screen, for a sheet that holds a whole conversation or a long picker. */
		tall?: boolean;
		onclose?: () => void;
		children: Snippet;
	} = $props();

	const uid = $props.id();
	let dialog: HTMLDialogElement | undefined = $state();
	let drag = $state(0);
	let start = $state<number | null>(null);

	// The native dialog is imperative, so its open state has to be pushed to the DOM.
	$effect(() => {
		if (!dialog) return;
		if (open && !dialog.open) dialog.showModal();
		if (!open && dialog.open) dialog.close();
	});

	// Dragged down by the grabber, the sheet follows the finger and closes past a threshold;
	// the close button stays as the single-pointer way out.
	function grab(event: PointerEvent) {
		if ((event.target as HTMLElement).closest('button')) return;
		start = event.clientY;
		(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
	}

	function move(event: PointerEvent) {
		if (start === null) return;
		drag = Math.max(0, event.clientY - start);
	}

	function release() {
		if (start === null) return;
		start = null;
		if (drag > SHEET_CLOSE_PX) open = false;
		drag = 0;
	}

	// A tap on the backdrop lands on the dialog element itself, outside its content box.
	function backdrop(event: MouseEvent) {
		if (event.target === dialog) open = false;
	}
</script>

<dialog
	bind:this={dialog}
	class:tall
	style:transform={drag > 0 ? `translateY(${drag}px)` : undefined}
	style:transition={start === null ? 'transform 180ms ease-out' : 'none'}
	onclose={() => {
		open = false;
		onclose?.();
	}}
	onclick={backdrop}
	aria-labelledby="{uid}-title"
>
	{#if open}
		<div class="sheet">
			<div
				class="handle"
				role="presentation"
				onpointerdown={grab}
				onpointermove={move}
				onpointerup={release}
				onpointercancel={release}
			>
				<span class="grabber" aria-hidden="true"></span>
				<div class="row spread heading">
					<div class="grow">
						<h2 id="{uid}-title">{title}</h2>
						{#if subtitle}<p class="muted small subtitle">{subtitle}</p>{/if}
					</div>
					<button class="icon close" onclick={() => (open = false)} aria-label="Close">
						<X size={18} strokeWidth={2.6} aria-hidden="true" />
					</button>
				</div>
			</div>
			{@render children()}
		</div>
	{/if}
</dialog>

<style>
	.tall {
		height: calc(100dvh - env(safe-area-inset-top) - 2.5rem);
	}

	.sheet {
		display: flex;
		flex-direction: column;
		gap: 0.9rem;
		min-height: 100%;
	}

	/* The whole top strip is the drag target, so it is easy to catch with a thumb. */
	.handle {
		touch-action: none;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		padding-top: 0.25rem;
	}

	.grabber {
		align-self: center;
		width: 40px;
		height: 5px;
		border-radius: 3px;
		background: var(--surface-2);
	}

	.heading {
		flex-wrap: nowrap;
		align-items: flex-start;
	}

	h2 {
		font-size: 1.75rem;
	}

	.subtitle {
		font-weight: 600;
	}
</style>
