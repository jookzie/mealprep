import type {
	BaselineStatus,
	Behaviour,
	Impact,
	Marker,
	MarkerSeries,
	SleepNight,
} from '$lib/api/types';
import type { ChartPoint } from './trend-chart';
import { datesInRange, type IsoDate } from './week';
import type { Domain } from './weight';

/** How long a sync stays fresh before opening the Body screen reads Health Connect again. */
export const SYNC_EVERY_MS = 60 * 60 * 1000;

export function shouldSync(syncedAt: string | undefined, now: Date): boolean {
	if (syncedAt === undefined) return true;
	return now.getTime() - new Date(syncedAt).getTime() >= SYNC_EVERY_MS;
}

export function syncedLabel(syncedAt: string | undefined, now: Date): string {
	if (syncedAt === undefined) return 'not synced yet';
	const minutes = Math.floor((now.getTime() - new Date(syncedAt).getTime()) / 60_000);
	if (minutes < 1) return 'synced just now';
	if (minutes < 60) return `synced ${minutes} min ago`;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return `synced ${hours} h ago`;
	return `synced ${Math.floor(hours / 24)} d ago`;
}

/** A duration in minutes as hours and minutes: 440 is "7h 20m". */
export function formatDuration(minutes: number): string {
	const rounded = Math.round(minutes);
	const hours = Math.floor(rounded / 60);
	const rest = rounded % 60;
	if (hours === 0) return `${rest}m`;
	return rest === 0 ? `${hours}h` : `${hours}h ${rest}m`;
}

/** A minute counted from a morning's midnight as a clock time: -30 is "23:30". */
export function formatClock(minute: number): string {
	const wrapped = ((Math.round(minute) % 1440) + 1440) % 1440;
	const hours = Math.floor(wrapped / 60);
	return `${String(hours).padStart(2, '0')}:${String(wrapped % 60).padStart(2, '0')}`;
}

/** A spread of clock times, which reads as "give or take". */
export function formatSpread(minutes: number): string {
	return `±${Math.round(minutes)} min`;
}

export function markerName(marker: Marker): string {
	return marker === 'hrv' ? 'HRV' : 'Resting heart rate';
}

export function formatMarker(marker: Marker, value: number): string {
	return marker === 'hrv' ? `${Math.round(value)} ms` : `${Math.round(value)} bpm`;
}

/**
 * Where the week sits, in words. Neutral on purpose: the words are the same for both
 * markers, and which side is good is said once, in `statusMeaning`, rather than coloured.
 */
export function statusPhrase(status: BaselineStatus): string {
	switch (status) {
		case 'below':
			return 'below your normal range';
		case 'within':
			return 'within your normal range';
		case 'above':
			return 'above your normal range';
	}
}

/**
 * What a week outside the range usually means. HRV falls and resting heart rate rises
 * under the same load, so the two readings mirror each other.
 */
export function statusMeaning(marker: Marker, status: BaselineStatus): string {
	const strained = marker === 'hrv' ? status === 'below' : status === 'above';
	const fresh = marker === 'hrv' ? status === 'above' : status === 'below';
	if (strained) {
		return 'Often a sign of accumulated training load, short sleep, illness coming on or stress.';
	}
	if (fresh) return 'Usually a sign of recovery or improving fitness.';
	return 'Your body is handling what you are doing.';
}

/**
 * The series as the chart draws it: the 7-day average is the line, because it is the
 * figure the verdict is read from, and each morning's reading is a dot.
 */
export function markerChart(series: MarkerSeries): ChartPoint[] {
	return series.points.map((point) => ({
		date: point.date,
		value: point.value,
		line: point.average,
		band: point.band,
	}));
}

const BEHAVIOURS: Record<Behaviour, string> = {
	'slept-seven-hours': 'Slept 7 hours or more',
	'earlier-bedtime': 'Went to bed earlier than usual',
	'trained-day-before': 'Worked out the day before',
	'energy-over-target-day-before': 'Planned over the energy target the day before',
	'protein-at-target-day-before': 'Planned protein at target the day before',
};

export function behaviourLabel(behaviour: Behaviour): string {
	return BEHAVIOURS[behaviour];
}

/** The comparison in words, or that there is none worth reading. */
export function impactPhrase(impact: Impact, marker: Marker): string {
	if (!impact.clear) return 'no clear difference';
	const direction = impact.changePercent > 0 ? 'higher' : 'lower';
	const name = marker === 'hrv' ? 'HRV' : 'resting heart rate';
	return `${name} ${Math.abs(Math.round(impact.changePercent))}% ${direction}`;
}

/** Whether a clear impact went with better recovery, which depends on the marker. */
export function impactHelps(impact: Impact, marker: Marker): boolean {
	return marker === 'hrv' ? impact.changePercent > 0 : impact.changePercent < 0;
}

/** Clear differences first, the largest leading; the rest keep the order the rules gave. */
export function rankImpacts(impacts: readonly Impact[]): Impact[] {
	return impacts.toSorted((a, b) => {
		if (a.clear !== b.clear) return a.clear ? -1 : 1;
		return a.clear ? Math.abs(b.changePercent) - Math.abs(a.changePercent) : 0;
	});
}

export type SleepDay = { date: IsoDate; night: SleepNight | null };

/**
 * Every morning from the first night to the last, with the nights that were recorded.
 * A missing night stays in as an empty column, so a week the strap was off reads as a gap
 * rather than as the neighbouring nights moving closer together.
 */
export function sleepDays(nights: readonly SleepNight[]): SleepDay[] {
	const first = nights.at(0);
	const last = nights.at(-1);
	if (!first || !last) return [];
	const byDate = new Map(nights.map((night) => [night.date, night]));
	return datesInRange(first.date, last.date).map((date) => ({
		date,
		night: byDate.get(date) ?? null,
	}));
}

/** The clock span the sleep chart covers: from the earliest bedtime to the latest wake, to the hour. */
export function sleepDomain(days: readonly SleepDay[]): Domain {
	const nights = days.flatMap((day) => (day.night ? [day.night] : []));
	if (nights.length === 0) return { min: -120, max: 480 };
	const earliest = Math.min(...nights.map((night) => night.bedMinute));
	const latest = Math.max(...nights.map((night) => night.wakeMinute));
	return { min: Math.floor(earliest / 60) * 60, max: Math.ceil(latest / 60) * 60 };
}

/** Whole hours to label the clock axis with, every two hours, or every four on a wide span. */
export function hourTicks({ min, max }: Domain): number[] {
	const step = max - min > 12 * 60 ? 240 : 120;
	const found: number[] = [];
	for (let minute = Math.ceil(min / step) * step; minute <= max; minute += step) {
		found.push(minute);
	}
	return found;
}
