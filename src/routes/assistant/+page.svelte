<script lang="ts">
	import { clearAssistantKey, runMutation, setAssistantKey } from '$lib/api';
	import { assistant } from '$lib/assistant.svelte';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import {
		appliedEntityLine,
		entityHeading,
		entityReach,
		isPastDay,
		recordSummary,
		reversalHint,
		summarisePreview,
	} from '$lib/domain/assistant';
	import { formatSigned } from '$lib/domain/format';
	import { dayMonthLabel, fullDateLabel, todayIso, weekdayLabel } from '$lib/domain/week';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let keyDraft = $state('');
	let editingKey = $state(false);

	// The request and the proposal live in the session, so they outlast this screen.
	const preview = $derived(assistant.preview);

	// Read once per visit rather than per render: a day that crosses midnight mid-review
	// would otherwise change category under the user's cursor.
	const today = todayIso();
	const summary = $derived(preview ? summarisePreview(preview, today) : null);

	const examples = [
		'next 4 days I have 40 g less whey',
		'swap tomorrow for a rest day',
		'sync my whey from Open Food Facts',
		'how is my vitamin D against the RDI?',
	];

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
		assistant.discard();
	}

	const time = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' });
</script>

<PageHeader title="Assistant" subtitle="Say what changed and review the edit before it lands" />

{#if !data.key.present}
	<section class="card stack">
		<h2>Turn the assistant on</h2>
		<p class="small">
			The assistant reads a plain-language request — "next 4 days I have 40 g less whey", "make a
			meal of oats and whey", "sync my whey from Open Food Facts" — and proposes the change, or
			answers a question about your plan such as how your micronutrients compare with the
			reference intakes. It never writes on its own: you see what would change and confirm.
		</p>
		<p class="small">
			It needs your own Anthropic API key. The key is stored on this device and is sent nowhere
			but to <code>api.anthropic.com</code>, as the header of the one request that asks for a
			proposal.
		</p>
		<details class="small">
			<summary>What leaves the device when you ask</summary>
			<p class="muted">
				Your request, and a view of your planning: the days around today with what is on them and
				their nutrient totals, your plans, meals and products with what they hold, their macros
				and nutrients, your categories, and your targets. Nothing else — no weigh-ins, no prices,
				no history. Nothing is sent until you ask, and nothing at all while there is no key. A sync
				asks Open Food Facts for the product's barcode, as a scan does.
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
			<span>What should change, or what would you like to know?</span>
			<textarea
				rows="3"
				placeholder="next 4 days I have 40 g less whey"
				bind:value={assistant.request}
			></textarea>
		</label>
		<div class="row spread">
			<div class="row examples">
				{#each examples as example (example)}
					<button class="ghost pill-button small" onclick={() => (assistant.request = example)}>
						{example}
					</button>
				{/each}
			</div>
			<button
				class="primary"
				disabled={assistant.asking || assistant.request.trim() === ''}
				onclick={() => assistant.ask()}
			>
				{assistant.asking ? 'Thinking…' : 'Ask'}
			</button>
		</div>
	</section>
{/if}

{#if preview && summary}
	<section class="card stack" aria-label={summary.isAnswer ? 'Answer' : 'Proposed change'}>
		<div class="row spread">
			<h2>{summary.isAnswer ? 'Answer' : 'Proposed change'}</h2>
			{#if !summary.isEmpty}
				<span class="pill">Not applied</span>
			{/if}
		</div>
		<p class="request small muted">“{preview.proposal.request}”</p>
		{#if preview.proposal.rationale}
			<p class="rationale">{preview.proposal.rationale}</p>
		{/if}

		{#if summary.isAnswer}
			<!-- The prose is the reply; a hint about naming a product would misread a question. -->
		{:else if summary.isEmpty}
			<p class="muted small">
				{summary.label} Try naming the product, meal or day to change.
			</p>
		{:else}
			<ol class="list">
				{#each preview.entities as entity, index (index)}
					{@const reach = entityReach(entity)}
					<li class="entry stack">
						<div class="row spread">
							<strong class="grow truncate">{entity.label}</strong>
							<span class="pill">{entityHeading(entity)}</span>
						</div>
						<ul class="changes small">
							{#each entity.changes as line, lineIndex (lineIndex)}
								<li>{line}</li>
							{/each}
						</ul>
						<MacroLine macros={entity.macros} />
						{#if reach}
							<p class="small muted">{reach}</p>
						{/if}
					</li>
				{/each}
				{#each preview.days as day (day.date)}
					<li class="entry stack">
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
						{#if day.targetDelta}
							<p class="small muted">
								{formatSigned(day.targetDelta.energyKcal, 'kcal')} against your target
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
		{/if}

		<div class="row spread">
			<span class="small muted">{summary.isAnswer ? 'Nothing to apply.' : summary.label}</span>
			<div class="row">
				<button onclick={() => assistant.discard()}>{summary.isEmpty ? 'Done' : 'Discard'}</button>
				{#if !summary.isEmpty}
					<button class="primary" disabled={assistant.applying} onclick={() => assistant.apply()}>
						Apply
					</button>
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
						<summary>What it changed</summary>
						<ul class="list">
							{#each record.entityChanges ?? [] as change, index (index)}
								<li class="stack tight">
									<strong class="small">{appliedEntityLine(change)}</strong>
									<ul class="changes muted">
										{#each change.changes as line, lineIndex (lineIndex)}
											<li>{line}</li>
										{/each}
									</ul>
								</li>
							{/each}
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

	/* An answer comes as short lines and "- " lists, which only read as such kept apart. */
	.rationale {
		white-space: pre-line;
	}

	.entry {
		gap: 0.3rem;
		padding: 0.6rem 0;
		border-top: 1px solid var(--border);
	}

	.entry:first-child {
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
