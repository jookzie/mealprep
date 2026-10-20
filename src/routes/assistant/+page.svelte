<script lang="ts">
	import {
		applyAssistantEdit,
		clearAssistantKey,
		type Preview,
		proposeAssistantEdit,
		runMutation,
		setAssistantKey,
	} from '$lib/api';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import {
		dayDelta,
		isPastDay,
		operationLabel,
		recordSummary,
		reversalHint,
		summarisePreview,
	} from '$lib/domain/assistant';
	import { formatSigned } from '$lib/domain/format';
	import { MACRO_KEYS, MACRO_LABELS } from '$lib/domain/macros';
	import { dayMonthLabel, fullDateLabel, todayIso, weekdayLabel } from '$lib/domain/week';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let request = $state('');
	let keyDraft = $state('');
	let editingKey = $state(false);
	let busy = $state(false);

	// The proposal lives here and nowhere else: nothing is written until Apply, and
	// Discard has to leave no trace of it.
	let preview = $state<Preview | null>(null);

	// Read once per proposal rather than per render: a day that crosses midnight mid-review
	// would otherwise change category under the user's cursor.
	const today = todayIso();
	const summary = $derived(preview ? summarisePreview(preview, today) : null);

	const examples = [
		'next 4 days I have 40 g less whey',
		'swap tomorrow for a rest day',
		'add a banana to Friday',
	];

	async function ask() {
		const text = request.trim();
		if (text === '') return;
		busy = true;
		// A proposal reads nothing the loaded data owns, so there is nothing to invalidate.
		const result = await runMutation(() => proposeAssistantEdit(text), { invalidate: false });
		busy = false;
		if (result) preview = result;
	}

	async function apply() {
		const proposal = preview?.proposal;
		if (!proposal) return;
		busy = true;
		const applied = await runMutation(() => applyAssistantEdit(proposal), {
			success: 'Applied to your plan',
		});
		busy = false;
		if (applied) {
			preview = null;
			request = '';
		}
	}

	function discard() {
		preview = null;
	}

	async function saveKey() {
		const key = keyDraft.trim();
		if (key === '') return;
		const status = await runMutation(() => setAssistantKey(key), { success: 'Key saved' });
		if (status) {
			keyDraft = '';
			editingKey = false;
		}
	}

	async function forgetKey() {
		await runMutation(() => clearAssistantKey(), { success: 'Key forgotten' });
		preview = null;
	}

	function formatMacro(key: (typeof MACRO_KEYS)[number], value: number): string {
		return formatSigned(value, key === 'energyKcal' ? 'kcal' : 'g');
	}

	const time = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' });
</script>

<PageHeader title="Assistant" subtitle="Say what changed and review the edit before it lands" />

{#if !data.key.present}
	<section class="card stack">
		<h2>Turn the assistant on</h2>
		<p class="small">
			The assistant reads a plain-language correction — "next 4 days I have 40 g less whey" — and
			proposes the edit. It never writes: you see exactly which days change and confirm.
		</p>
		<p class="small">
			It needs your own Anthropic API key. The key is stored on this device and is sent nowhere
			but to <code>api.anthropic.com</code>, as the header of the one request that asks for a
			proposal.
		</p>
		<details class="small">
			<summary>What leaves the device when you ask</summary>
			<p class="muted">
				Your request, and a compact view of the plan around today: the dates in view with the
				labels and amounts on them, and the names and ids of your plans, meals and products,
				with your macro targets. Nothing else — no weigh-ins, no prices, no history. Nothing is
				sent until you ask a question, and nothing at all is sent while there is no key.
			</p>
		</details>
		<label class="field">
			<span>Anthropic API key</span>
			<input
				type="password"
				autocomplete="off"
				placeholder="sk-ant-…"
				bind:value={keyDraft}
				onkeydown={(event) => {
					if (event.key === 'Enter') saveKey();
				}}
			/>
		</label>
		<div class="row end">
			<button class="primary" disabled={keyDraft.trim() === ''} onclick={saveKey}>Save key</button>
		</div>
	</section>
{:else}
	<section class="card stack">
		<label class="field">
			<span>What changed?</span>
			<textarea
				rows="3"
				placeholder="next 4 days I have 40 g less whey"
				bind:value={request}
			></textarea>
		</label>
		<div class="row spread">
			<div class="row examples">
				{#each examples as example (example)}
					<button class="ghost pill-button small" onclick={() => (request = example)}>
						{example}
					</button>
				{/each}
			</div>
			<button class="primary" disabled={busy || request.trim() === ''} onclick={ask}>
				{busy ? 'Thinking…' : 'Ask'}
			</button>
		</div>
	</section>
{/if}

{#if preview && summary}
	<section class="card stack" aria-label="Proposed change">
		<div class="row spread">
			<h2>Proposed change</h2>
			<span class="pill">Not applied</span>
		</div>
		<p class="request small muted">“{preview.proposal.request}”</p>
		{#if preview.proposal.rationale}
			<p>{preview.proposal.rationale}</p>
		{/if}

		{#if summary.isEmpty}
			<p class="muted small">
				{summary.label} Nothing will be written. Try naming the product or meal to change.
			</p>
		{:else}
			<ol class="list">
				{#each preview.days as day (day.date)}
					{@const delta = dayDelta(day)}
					<li class="day stack">
						<div class="row spread">
							<strong>{weekdayLabel(day.date)} {dayMonthLabel(day.date)}</strong>
							<span class="pill">{day.label ?? 'Unplanned'}</span>
						</div>
						<ul class="changes small">
							{#each day.changes as line, index (index)}
								<li>{line}</li>
							{/each}
						</ul>
						<MacroLine macros={day.after} />
						<p class="small muted">
							Change:
							{#each MACRO_KEYS as key (key)}
								<span class="figure">{MACRO_LABELS[key]} {formatMacro(key, delta[key])}</span>
							{/each}
						</p>
						{#if day.targetDelta}
							<p class="small muted">
								Against target:
								{#each MACRO_KEYS as key (key)}
									<span class="figure">
										{MACRO_LABELS[key]} {formatMacro(key, day.targetDelta[key])}
									</span>
								{/each}
							</p>
						{/if}
						{#if day.detaches}
							<p class="small muted">
								This day gets its own copy of {day.label}. Every other day on that plan — the
								days already past included — keeps the original.
							</p>
						{/if}
						{#if isPastDay(day, today)}
							<p class="small past">
								This day has already happened. Applying changes what the plan says you ate.
							</p>
						{/if}
					</li>
				{/each}
			</ol>

			<details class="small">
				<summary>The {preview.proposal.operations.length} typed operations</summary>
				<ul class="changes muted">
					{#each preview.proposal.operations as operation, index (index)}
						<li>{operationLabel(operation)}</li>
					{/each}
				</ul>
			</details>
		{/if}

		<div class="row spread">
			<span class="small muted">{summary.label}</span>
			<div class="row">
				<button onclick={discard}>Discard</button>
				{#if !summary.isEmpty}
					<button class="primary" disabled={busy} onclick={apply}>Apply</button>
				{/if}
			</div>
		</div>
	</section>
{/if}

<section class="stack" aria-label="What the assistant has done">
	<h2>History</h2>
	{#if data.history.length === 0}
		<p class="empty">Nothing applied yet.</p>
	{:else}
		<ul class="list">
			{#each data.history as record (record.id)}
				<li class="card stack">
					<div class="row spread">
						<strong>“{record.request}”</strong>
						<span class="small muted">{time.format(new Date(record.createdAt))}</span>
					</div>
					{#if record.rationale}
						<p class="small">{record.rationale}</p>
					{/if}
					<span class="small muted">{recordSummary(record)}</span>
					<details class="small">
						<summary>What it changed, and how to undo it</summary>
						<ul class="list">
							{#each record.changes as change (change.date)}
								<li class="stack tight">
									<strong class="small">{fullDateLabel(change.date)}</strong>
									<ul class="changes muted">
										{#each change.changes as line, index (index)}
											<li>{line}</li>
										{/each}
									</ul>
									<span class="muted">{reversalHint(change)}</span>
								</li>
							{/each}
						</ul>
					</details>
				</li>
			{/each}
		</ul>
	{/if}
</section>

{#if data.key.present}
	<section class="card stack" aria-label="API key">
		<div class="row spread">
			<h2>API key</h2>
			<span class="pill">…{data.key.hint}</span>
		</div>
		<p class="small muted">
			Stored on this device and sent only to <code>api.anthropic.com</code>. The app never shows
			it in full.
		</p>
		{#if editingKey}
			<label class="field">
				<span>Replace it</span>
				<input type="password" autocomplete="off" placeholder="sk-ant-…" bind:value={keyDraft} />
			</label>
			<div class="row end">
				<button onclick={() => ((editingKey = false), (keyDraft = ''))}>Cancel</button>
				<button class="primary" disabled={keyDraft.trim() === ''} onclick={saveKey}>Save</button>
			</div>
		{:else}
			<div class="row end">
				<button onclick={() => (editingKey = true)}>Replace</button>
				<button class="danger" onclick={forgetKey}>Forget</button>
			</div>
		{/if}
	</section>
{/if}

<style>
	textarea {
		resize: vertical;
		min-height: 4.5rem;
	}

	.examples {
		gap: 0.3rem;
		flex: 1 1 12rem;
		min-width: 0;
	}

	/* An example is a shortcut, not an action: it reads as a hint and fills the field. */
	.pill-button {
		min-height: 32px;
		padding: 0.1rem 0.55rem;
		border: 1px solid var(--border);
		border-radius: 999px;
		color: var(--muted);
	}

	.request {
		font-style: italic;
	}

	.day {
		gap: 0.3rem;
		padding: 0.6rem 0;
		border-top: 1px solid var(--border);
	}

	.day:first-child {
		border-top: none;
		padding-top: 0;
	}

	.changes {
		list-style: disc;
		margin: 0;
		padding-left: 1.1rem;
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
	}

	.figure {
		display: inline-block;
		margin-left: 0.5rem;
		font-variant-numeric: tabular-nums;
	}

	/* The one thing on this screen the user cannot undo by planning differently, so it is
	   the one thing that is not muted. Warning, not danger: nothing is broken. */
	.past {
		color: var(--warning);
	}

	.tight {
		gap: 0.2rem;
	}

	details > summary {
		cursor: pointer;
		color: var(--muted);
		min-height: 32px;
	}
</style>
