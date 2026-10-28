<script lang="ts">
	import ArrowUp from '@lucide/svelte/icons/arrow-up';
	import { tick } from 'svelte';
	import { runMutation, setAssistantKey } from '$lib/api';
	import { assistant } from '$lib/assistant.svelte';
	import MacroLine from '$lib/components/MacroLine.svelte';
	import {
		type ChatMessage,
		entityHeading,
		entityReach,
		isPastDay,
		summarisePreview,
	} from '$lib/domain/assistant';
	import { formatSigned } from '$lib/domain/format';
	import { dayMonthLabel, todayIso, weekdayLabel } from '$lib/domain/week';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let keyDraft = $state('');
	let thread = $state<HTMLOListElement>();

	// Read once per visit rather than per render: a day that crosses midnight mid-review
	// would otherwise change category under the user's cursor.
	const today = todayIso();

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
		if (!latest) return;
		tick().then(() => thread?.lastElementChild?.scrollIntoView({ block: 'nearest' }));
	});

	async function saveKey() {
		const key = keyDraft.trim();
		if (key === '') return;
		const status = await runMutation(() => setAssistantKey(key), { success: 'Key saved' });
		if (status) keyDraft = '';
	}
</script>

<div class="chat">
	<header class="row spread">
		<h1>Assistant</h1>
		{#if data.key.present && assistant.messages.length > 0}
			<button class="ghost" disabled={assistant.asking} onclick={() => assistant.clear()}>
				New chat
			</button>
		{/if}
	</header>

	{#if !data.key.present}
		<section class="card stack">
			<p class="small">
				Chat about your plan: say what changed and confirm the edit, or ask a question. It needs
				your own Anthropic API key, stored on this device and sent only to
				<code>api.anthropic.com</code>.
			</p>
			<details class="small">
				<summary>What leaves the device</summary>
				<p class="muted">
					Your messages in the chat and the assistant's replies, and a view of your planning: the
					days around today, your plans, meals, products with their nutrients, categories and
					targets. No weigh-ins, no prices. Nothing is sent while there is no key.
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
		{#if assistant.messages.length === 0}
			<p class="empty small">Ask about your plan, or say what changed.</p>
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
													<p class="small past">This day has already happened.</p>
												{/if}
											</li>
										{/each}
									</ol>

									{#if message.status === 'proposed'}
										<div class="row end">
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
										<div class="row end">
											<span class="pill" class:applied={message.status === 'applied'}>{status}</span>
										</div>
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
	{/if}
</div>

<style>
	/* The whole screen is the chat: it fills the space above the tab bar, the thread takes
	   what is left, and the composer stays at the foot. */
	.chat {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		min-height: calc(
			100dvh - env(safe-area-inset-top) - var(--nav-height) - env(safe-area-inset-bottom) - 2.5rem
		);
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

	.empty {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
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

	details > summary {
		cursor: pointer;
		color: var(--muted);
		min-height: 32px;
	}
</style>
