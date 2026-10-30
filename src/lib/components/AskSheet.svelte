<script lang="ts">
	import ArrowUp from '@lucide/svelte/icons/arrow-up';
	import Check from '@lucide/svelte/icons/check';
	import { tick } from 'svelte';
	import type { AssistantKeyStatus } from '$lib/api';
	import { assistant } from '$lib/assistant.svelte';
	import {
		type ChatMessage,
		entityHeading,
		entityReach,
		isPastDay,
		summarisePreview,
	} from '$lib/domain/assistant';
	import { formatSigned } from '$lib/domain/format';
	import { dayMonthLabel, todayIso, weekdayLabel } from '$lib/domain/week';
	import Dialog from './Dialog.svelte';
	import MacroLine from './MacroLine.svelte';

	let {
		open = $bindable(false),
		keyStatus,
		onclose,
		onSaveKey,
	}: {
		open: boolean;
		keyStatus: AssistantKeyStatus;
		onclose?: () => void;
		/** Answers something only when the key was stored. */
		onSaveKey: (key: string) => Promise<unknown>;
	} = $props();

	let keyDraft = $state('');
	let thread = $state<HTMLOListElement>();

	// Read once per opening rather than per render: a day that crosses midnight mid-review
	// would otherwise change category under the user's thumb.
	const today = $derived.by(() => {
		void open;
		return todayIso();
	});

	const STATUS_LABELS: Record<ChatMessage['status'], string | null> = {
		asking: null,
		proposed: null,
		answered: null,
		applied: 'Applied',
		discarded: 'Discarded',
	};

	// The newest turn is the one being read, so a question sent or an answer arriving
	// brings it into view. Keyed on the last message's status, which changes on both.
	const latest = $derived(assistant.messages.at(-1)?.status);
	$effect(() => {
		if (!latest || !open) return;
		tick().then(() => thread?.lastElementChild?.scrollIntoView({ block: 'nearest' }));
	});

	async function saveKey() {
		const key = keyDraft.trim();
		if (key === '') return;
		const saved = await onSaveKey(key);
		if (saved !== undefined) keyDraft = '';
	}
</script>

<Dialog bind:open title="Ask" tall {onclose}>
	{#if !keyStatus.present}
		<div class="stack setup">
			<p class="display intro">Change the plan by saying what changed</p>
			<div class="example stack">
				<span class="bubble user">next 4 days I have 40 g less whey</span>
				<span class="small">
					You see which days change and by how much, then apply or discard. Nothing is written
					until you say so.
				</span>
			</div>
			<div class="grid-2">
				<div class="facts sent">
					<strong>Sent per message</strong>
					<ul>
						<li>What you type</li>
						<li>The chat so far</li>
						<li>Your plan around today: plans, meals, products and their nutrients</li>
						<li>Categories and targets</li>
					</ul>
				</div>
				<div class="facts">
					<strong>Never sent</strong>
					<ul>
						<li>Weigh-ins</li>
						<li>Prices</li>
						<li>Your database</li>
						<li>Anything while no key is stored</li>
					</ul>
				</div>
			</div>
			<label class="field">
				<span>Your Anthropic API key</span>
				<input
					type="password"
					autocomplete="off"
					placeholder="sk-ant-…"
					bind:value={keyDraft}
					onkeydown={(event) => {
						if (event.key === 'Enter') saveKey();
					}}
				/>
				<span class="muted small hint">
					Kept on this phone and sent only to <code>api.anthropic.com</code>.
				</span>
			</label>
			<button class="primary wide" disabled={keyDraft.trim() === ''} onclick={saveKey}>
				Turn on Ask
			</button>
		</div>
	{:else}
		<div class="chat">
			{#if assistant.messages.length > 0}
				<div class="row end">
					<button class="outline small-button" disabled={assistant.asking} onclick={() => assistant.clear()}>
						Start over
					</button>
				</div>
			{/if}

			{#if assistant.messages.length === 0}
				<p class="empty">Ask about your plan, or say what changed.</p>
			{:else}
				<ol class="thread" bind:this={thread}>
					{#each assistant.messages as message (message.id)}
						<li class="turn">
							<p class="bubble user">{message.request}</p>
							{#if !message.preview}
								<p class="bubble reply thinking" aria-live="polite">Thinking…</p>
							{:else}
								{@const preview = message.preview}
								{@const summary = summarisePreview(preview, today)}
								{@const status = STATUS_LABELS[message.status]}
								<div class="stack reply-block" aria-label={summary.isAnswer ? 'Answer' : 'Proposed change'}>
									{#if preview.proposal.rationale}
										<p class="bubble reply rationale">{preview.proposal.rationale}</p>
									{/if}

									{#if summary.isAnswer}
										<!-- The prose is the reply; a hint about naming a product would misread a question. -->
									{:else if summary.isEmpty}
										<p class="muted small">
											{summary.label} Try naming the product, meal or day to change.
										</p>
									{:else}
										<ol class="changes-list" class:settled={message.status !== 'proposed'}>
											{#each preview.entities as entity, index (index)}
												{@const reach = entityReach(entity)}
												<li class="change">
													<div class="row spread">
														<strong class="grow truncate">{entity.label}</strong>
														<span class="pill">{entityHeading(entity)}</span>
													</div>
													<ul class="lines small">
														{#each entity.changes as line, lineIndex (lineIndex)}
															<li>{line}</li>
														{/each}
													</ul>
													<MacroLine macros={entity.macros} />
													{#if reach}
														<p class="small">{reach}</p>
													{/if}
												</li>
											{/each}
											{#each preview.days as day (day.date)}
												<li class="change">
													<div class="row spread">
														<span class="display date">{weekdayLabel(day.date)} {dayMonthLabel(day.date)}</span>
														<span class="pill">{day.label ?? 'Unplanned'}</span>
													</div>
													<ul class="lines small">
														{#each day.changes as line, index (index)}
															<li>{line}</li>
														{/each}
													</ul>
													<MacroLine macros={day.after} />
													{#if day.targetDelta}
														<p class="small">
															{formatSigned(day.targetDelta.energyKcal, 'kcal')} against your target
														</p>
													{/if}
													{#if message.status === 'proposed' && isPastDay(day, today)}
														<p class="small notice">This day has already happened.</p>
													{/if}
												</li>
											{/each}
										</ol>

										{#if message.status === 'proposed'}
											<div class="grid-2">
												<button class="outline" onclick={() => assistant.discard(message.id)}>Discard</button>
												<button
													class="primary"
													disabled={assistant.applying}
													onclick={() => assistant.apply(message.id)}
												>
													Apply
												</button>
											</div>
										{:else if status}
											<span class="pill outcome" class:applied={message.status === 'applied'}>
												{#if message.status === 'applied'}<Check size={14} strokeWidth={3} aria-hidden="true" />{/if}
												{status}
											</span>
										{/if}
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
					placeholder={assistant.messages.length === 0 ? 'Change something, or ask' : 'Reply'}
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
					<ArrowUp size={20} strokeWidth={2.6} aria-hidden="true" />
				</button>
			</form>
		</div>
	{/if}
</Dialog>

<style>
	.intro {
		font-size: 1.6rem;
		line-height: 1.1;
	}

	.example {
		background: var(--carbs);
		color: var(--on-macro);
		border-radius: var(--radius);
		padding: 0.9rem 1rem;
		gap: 0.5rem;
	}

	.example .user {
		align-self: flex-end;
	}

	.facts {
		background: var(--surface);
		border-radius: var(--radius);
		padding: 0.75rem 0.9rem;
		font-size: 0.85rem;
	}

	.facts.sent {
		background: var(--protein);
		color: var(--on-macro);
	}

	.facts ul {
		margin: 0.35rem 0 0;
		padding-left: 1rem;
		list-style: disc;
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
	}

	.hint {
		font-weight: 500;
	}

	.wide {
		width: 100%;
		min-height: 52px;
	}

	/* The sheet is the chat: the thread takes what is left and the composer stays at the foot. */
	.chat {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		min-height: 0;
	}

	.small-button {
		min-height: 36px;
		font-size: 0.9rem;
	}

	.empty {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.thread {
		flex: 1;
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
		gap: 0.6rem;
		scroll-margin-bottom: 6rem;
	}

	.bubble {
		margin: 0;
		padding: 0.7rem 1rem;
		min-width: 0;
		overflow-wrap: anywhere;
		max-width: 85%;
	}

	/* The user's side reads as what was typed: kept to its own line breaks, on the right. */
	.user {
		align-self: flex-end;
		white-space: pre-line;
		background: var(--accent);
		color: var(--accent-text);
		border-radius: 24px 24px 6px 24px;
		font-weight: 500;
	}

	.reply {
		align-self: flex-start;
		background: var(--surface);
		border-radius: 24px 24px 24px 6px;
	}

	.thinking {
		animation: pulse 1.4s ease-in-out infinite;
	}

	@keyframes pulse {
		50% {
			opacity: 0.45;
		}
	}

	/* An answer comes as short lines and "- " lists, which only read as such kept apart. */
	.rationale {
		white-space: pre-line;
	}

	.changes-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: 0.6rem;
	}

	/* A proposed change is a protein-orange card: the colour of the edit most requests make,
	   and loud enough that Apply is never pressed without reading it. */
	.change {
		background: var(--protein);
		color: var(--on-macro);
		border-radius: var(--radius);
		padding: 0.85rem 1rem;
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}

	.change .pill {
		background: rgb(255 255 255 / 0.6);
		color: var(--on-macro);
	}

	.date {
		font-size: 1.25rem;
	}

	.lines {
		margin: 0;
		padding-left: 1.1rem;
		list-style: disc;
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
		font-weight: 600;
	}

	/* What was applied or discarded stays readable, but no longer asks for a decision. */
	.settled .change {
		opacity: 0.6;
	}

	.outcome {
		align-self: flex-start;
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
	}

	.outcome.applied {
		background: var(--carbs);
		color: var(--on-macro);
	}

	.composer {
		position: sticky;
		bottom: 0;
		display: flex;
		align-items: flex-end;
		gap: 0.5rem;
		padding: 0.35rem 0.35rem 0.35rem 1rem;
		background: var(--surface);
		border-radius: 28px;
		margin-top: auto;
	}

	.composer textarea {
		flex: 1;
		min-width: 0;
		min-height: 44px;
		max-height: 10rem;
		resize: none;
		field-sizing: content;
		border: none;
		background: transparent;
		padding: 0.6rem 0;
	}

	.send {
		flex-shrink: 0;
	}
</style>
