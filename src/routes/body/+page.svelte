<script lang="ts">
	import { onMount } from 'svelte';
	import { connectHealth, installHealthConnect, runMutation, syncHealth } from '$lib/api';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { formatKcal } from '$lib/domain/format';
	import {
		formatDuration,
		formatMarker,
		formatSpread,
		markerName,
		shouldSync,
		statusPhrase,
		syncedLabel,
	} from '$lib/domain/health';
	import { formatMeasurement, ofKind, waistToHeight } from '$lib/domain/measurement';
	import { todayIso } from '$lib/domain/week';
	import { formatKilograms, formatRate } from '$lib/domain/weight';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let syncing = $state(false);
	let now = $state(new Date());

	const health = $derived(data.health);
	const connected = $derived(health.availability === 'available' && health.connected);
	/** HRV when the source writes it; resting heart rate otherwise. */
	const marker = $derived(data.recovery.hrv ?? data.recovery.restingHeartRate);
	const waist = $derived(ofKind(data.measurements, 'waist').at(-1));
	const ratio = $derived(waistToHeight(data.measurements));

	async function sync() {
		syncing = true;
		await runMutation(() => syncHealth(todayIso()));
		syncing = false;
		now = new Date();
	}

	async function connect() {
		const granted = await runMutation(() => connectHealth(), { invalidate: false });
		if (granted) await sync();
	}

	// Opening the screen is when the user wants current figures, so a stale import is
	// refreshed here rather than on a timer nobody sees.
	onMount(() => {
		if (connected && shouldSync(health.syncedAt, new Date())) sync();
	});
</script>

<PageHeader title="Body" subtitle="What the plan is measured against" />

<ul class="list">
	{#if connected}
		<li>
			<a class="card list-link" href="/body/recovery">
				<div class="row spread">
					<strong>Recovery</strong>
					<span class="small muted">→</span>
				</div>
				<p class="small muted">
					{#if marker?.average !== undefined}
						{markerName(marker.marker)} {formatMarker(marker.marker, marker.average)}
						{#if marker.status}· {statusPhrase(marker.status)}{:else}· a baseline takes two weeks{/if}
					{:else}
						A week of readings gives a first figure
					{/if}
				</p>
			</a>
		</li>
		<li>
			<a class="card list-link" href="/body/sleep">
				<div class="row spread">
					<strong>Sleep</strong>
					<span class="small muted">→</span>
				</div>
				<p class="small muted">
					{#if data.sleep.averageMinutes !== undefined}
						{formatDuration(data.sleep.averageMinutes)} a night
						{#if data.sleep.bedtimeSpreadMinutes !== undefined}
							· bedtime {formatSpread(data.sleep.bedtimeSpreadMinutes)}
						{/if}
					{:else}
						Three nights give a first average
					{/if}
				</p>
			</a>
		</li>
	{/if}

	<li>
		<a class="card list-link" href="/weight">
			<div class="row spread">
				<strong>Weight</strong>
				<span class="small muted">
					{data.weight.ratePerWeek === undefined ? '' : formatRate(data.weight.ratePerWeek)} →
				</span>
			</div>
			<p class="small muted">
				{data.weight.trend === undefined
					? 'Nothing logged yet'
					: `Trend ${formatKilograms(data.weight.trend)}`}
			</p>
		</a>
	</li>

	<li>
		<a class="card list-link" href="/body/measurements">
			<div class="row spread">
				<strong>Measurements</strong>
				<span class="small muted">→</span>
			</div>
			<p class="small muted">
				{#if waist}
					Waist {formatMeasurement('waist', waist.value)}
					{#if ratio}· waist-to-height {ratio.ratio.toFixed(2)}{/if}
				{:else if data.measurements.length > 0}
					{data.measurements.length} recorded · add a waist to track it
				{:else}
					Waist, hips, body fat and more
				{/if}
			</p>
		</a>
	</li>

	<li>
		<a class="card list-link" href="/body/energy">
			<div class="row spread">
				<strong>Energy balance</strong>
				<span class="small muted">→</span>
			</div>
			<p class="small muted">
				{#if data.energy.expenditureKcal !== undefined}
					About {formatKcal(data.energy.expenditureKcal)} a day burned, by your weight trend
				{:else}
					Needs two weeks of planned days and weigh-ins
				{/if}
			</p>
		</a>
	</li>

	<li>
		<a class="card list-link" href="/targets">
			<div class="row spread">
				<strong>Daily targets</strong>
				<span class="small muted">→</span>
			</div>
			<p class="small muted">
				{data.targets
					? `${formatKcal(data.targets.macros.energyKcal)} · ${Math.round(data.targets.macros.proteinG)} g protein`
					: 'Not set yet'}
			</p>
		</a>
	</li>
</ul>

<!--
	Health Connect gets exactly one row: an offer before access, a sync line after, and
	nothing on a device that has none.
-->
{#if health.availability === 'update-required'}
	<section class="card stack">
		<p class="small">
			Install or update Health Connect to see recovery and sleep from your wearable.
		</p>
		<div class="row end">
			<button onclick={() => runMutation(() => installHealthConnect(), { invalidate: false })}
				>Open Play Store</button
			>
		</div>
	</section>
{:else if health.availability === 'available' && !health.connected}
	<section class="card stack">
		<p class="small">
			Connect Health Connect to see recovery and sleep from your wearable, and which habits go
			with better mornings. Data is read, never written, and stays on this phone.
		</p>
		<div class="row end">
			<button class="primary" onclick={connect}>Connect</button>
		</div>
	</section>
{:else if connected}
	<div class="row spread small muted sync">
		<span>Health Connect · {syncing ? 'syncing…' : syncedLabel(health.syncedAt, now)}</span>
		<button class="ghost" onclick={sync} disabled={syncing}>Sync</button>
	</div>
{/if}

<style>
	.list-link p {
		margin-top: 0.15rem;
	}

	.sync {
		padding: 0 0.25rem;
	}
</style>
