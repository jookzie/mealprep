<script lang="ts">
	import ArrowUp from '@lucide/svelte/icons/arrow-up';
	import { tick } from 'svelte';
	import { clearAssistantKey, runMutation, setAssistantKey } from '$lib/api';
	import { assistant } from '$lib/assistant.svelte';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import {
		appliedEntityLine,
		type ChatMessage,
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
	let thread = $state<HTMLOListElement>();

	// Read once per visit rather than per render: a day that crosses midnight mid-review
	// would otherwise change category under the user's cursor.
	const today = todayIso();

	const STATUS_LABELS: Record<ChatMessage['status'], string | null> = {
		asking: null,
		proposed: 'Not applied',
		answered: null,
		applied: 'Applied',
		discarded: 'Discarded',
	};

	// The newest turn is the one being read, so a question sent or an answer arriving
	// brings it into view. Keyed on the last message's status, which changes on both.
	const latest = $derived(assistant.messages.at(-1)?.status);
	$effect(() => {
		if (!latest) return;
		tick().then(() => thread?.lastElementChild?.scrollIntoView({ block: 'nearest' }));
	});

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
		assistant.clear();
	}

	const time = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' });
</script>

<PageHeader title="Assistant" subtitle="Ask about your plan or say what changed">
	{#snippet actions()}
		{#if data.key.present && assistant.messages.length > 0}
			<button disabled={assistant.asking} onclick={() => assistant.clear()}>New chat</button>
		{/if}
	{/snippet}
</PageHeader>

{#if !data.key.present}
	<section class="card stack">
		<h2>Turn the assistant on</h2>
		<p class="small">
			The assistant is a chat about your plan. Say what changed and it proposes the edit, or ask
			a question, such as how your micronutrients compare with the reference intakes. It never
			writes on its own: you see what would change and confirm.
		</p>
		<p class="small">
			It needs your own Anthropic API key. The key is stored on this device and is sent nowhere
			but to <code>api.anthropic.com</code>, as the header of each message you send.
		</p>
		<details class="small">
			<summary>What leaves the device when you send a message</summary>
			<p class="muted">
				Your message and the earlier messages of the chat with their replies, and a view of your
				planning: the days around today with what is on them and their nutrient totals, your
				plans, meals and products with what they hold, their macros and nutrients, your
				categories, and your targets. Nothing else — no weigh-ins, no prices, no history. Nothing
				is sent until you send, and nothing at all while there is no key. A sync asks Open Food
				Facts for the product's barcode, as a scan does.
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
	<section class="chat stack" aria-label="Chat">
		{#if assistant.messages.length === 0}
			<p class="empty small">
				Ask about your plan, or say what changed. Nothing is applied until you confirm it.
			</p>
		{:else}
			<ol class="thread" bind:this={thread}>
				{#each assistant.messages as message (message.id)}
					<li class="turn">
						<p class="bubble user">{message.request}</p>
						{#if !message.preview}
							<p class="bubble reply muted thinking" aria-live="polite">Thinking…</p>
						{:else}
							{@const preview = message.preview}
							{@const summary = summarisePreview(preview, today)}
							{@const status = STATUS_LABELS[message.status]}
							<div
								class="bubble reply stack"
								aria-label={summary.isAnswer ? 'Answer' : 'Proposed change'}
							>
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
									<ol class="list" class:settled={message.status !== 'proposed'}>
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
												{#if message.status === 'proposed' && isPastDay(day, today)}
													<p class="small past">
														This day has already happened. Applying changes what the plan says you
														ate.
													</p>
												{/if}
											</li>
										{/each}
									</ol>

									<div class="row spread">
										<span class="small muted">{summary.label}</span>
										{#if message.status === 'proposed'}
											<div class="row">
												<button onclick={() => assistant.discard(message.id)}>Discard</button>
												<button
													class="primary"
													disabled={assistant.applying}
													onclick={() => assistant.apply(message.id)}
												>
													Apply
												</button>
											</div>
										{:else if status}
											<span class="pill" class:applied={message.status === 'applied'}>{status}</span>
										{/if}
									</div>
								{/if}
							</div>
						{/if}
					</li>
				{/each}
			</ol>
		{/if}

		<form
			class="composer"
			onsubmit={(event) => {
				event.preventDefault();
				assistant.send();
			}}
		>
			<textarea
				rows="1"
				enterkeyhint="send"
				aria-label="Message the assistant"
				placeholder={assistant.messages.length === 0 ? 'Ask or say what changed' : 'Reply'}
				bind:value={assistant.draft}
				onkeydown={(event) => {
					// Enter sends and Shift+Enter breaks the line, as in any chat.
					if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
						event.preventDefault();
						assistant.send();
					}
				}}
			></textarea>
			<button
				type="submit"
				class="primary icon send"
				aria-label="Send"
				disabled={assistant.asking || assistant.draft.trim() === ''}
			>
				<ArrowUp size={20} aria-hidden="true" />
			</button>
		</form>
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
	.thread {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.turn {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		/* Scrolled into view with room for the composer that sits over the page's foot. */
		scroll-margin-bottom: 6rem;
	}

	.bubble {
		margin: 0;
		border-radius: var(--radius);
		padding: 0.6rem 0.85rem;
		min-width: 0;
		overflow-wrap: anywhere;
	}

	/* The user's side reads as what was typed: kept to its own line breaks, on the right. */
	.user {
		align-self: flex-end;
		max-width: 85%;
		white-space: pre-line;
		background: var(--accent);
		color: var(--accent-text);
		border-bottom-right-radius: 4px;
	}

	/* A reply can carry a whole proposal, so it takes the full width a card would. */
	.reply {
		align-self: stretch;
		background: var(--surface);
		border: 1px solid var(--border);
		border-bottom-left-radius: 4px;
	}

	.thinking {
		align-self: flex-start;
		animation: pulse 1.4s ease-in-out infinite;
	}

	@keyframes pulse {
		50% {
			opacity: 0.45;
		}
	}

	/* An answer comes as short lines and "- " lists, which only read as such kept apart. */
	.rationale {
		margin: 0;
		white-space: pre-line;
	}

	/* Above the tab bar while the thread scrolls, so a reply is always one tap away. */
	.composer {
		position: sticky;
		bottom: calc(var(--nav-height) + env(safe-area-inset-bottom) + 0.5rem);
		z-index: 5;
		display: flex;
		align-items: flex-end;
		gap: 0.5rem;
		padding: 0.5rem;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius);
	}

	.composer textarea {
		flex: 1;
		min-width: 0;
		min-height: 44px;
		max-height: 10rem;
		resize: none;
		field-sizing: content;
	}

	.send {
		flex-shrink: 0;
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

	/* What was applied or discarded stays readable, but no longer asks for a decision. */
	.settled {
		opacity: 0.75;
	}

	.applied {
		color: var(--accent);
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
