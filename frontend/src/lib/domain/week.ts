/** A calendar date as the API writes it: YYYY-MM-DD. */
export type IsoDate = string;

/**
 * Every calendar date is handled in UTC. `new Date('2026-09-07')` parses as UTC but
 * `getDate()` reads local, which lands a day early west of UTC; going through
 * Date.UTC on the way in and toISOString on the way out keeps the two consistent.
 */
function parse(date: IsoDate): Date {
	const [year, month, day] = date.split('-').map(Number);
	return new Date(Date.UTC(year, month - 1, day));
}

function format(date: Date): IsoDate {
	return date.toISOString().slice(0, 10);
}

function shiftDays(date: IsoDate, days: number): IsoDate {
	const d = parse(date);
	d.setUTCDate(d.getUTCDate() + days);
	return format(d);
}

/** Today as the user's clock reads it, expressed as a UTC-anchored ISO date. */
export function todayIso(): IsoDate {
	const now = new Date();
	return format(new Date(Date.UTC(now.getFullYear(), now.getMonth(), now.getDate())));
}

/** The Monday of the week the date falls in. */
export function startOfIsoWeek(date: IsoDate): IsoDate {
	const d = parse(date);
	// getUTCDay is 0 on Sunday; shift so Monday is 0.
	return shiftDays(date, -((d.getUTCDay() + 6) % 7));
}

export function weekDates(weekStart: IsoDate): IsoDate[] {
	return Array.from({ length: 7 }, (_, i) => shiftDays(weekStart, i));
}

export function weekRange(weekStart: IsoDate): { from: IsoDate; to: IsoDate } {
	return { from: weekStart, to: shiftDays(weekStart, 6) };
}

/** Monday is 0, matching the order weekDates returns and the grid draws. */
export function weekdayIndex(date: IsoDate): number {
	return (parse(date).getUTCDay() + 6) % 7;
}

/** Every date from `from` to `to` inclusive; empty when the range runs backwards. */
export function datesInRange(from: IsoDate, to: IsoDate): IsoDate[] {
	const dates: IsoDate[] = [];
	for (let date = from; date <= to; date = shiftDays(date, 1)) {
		dates.push(date);
	}
	return dates;
}

/**
 * The dates a bulk assignment would touch: every day in the range whose weekday was
 * chosen. Assigning one plan to Mon/Wed/Fri is otherwise a dialog per day.
 */
export function datesMatchingWeekdays(
	from: IsoDate,
	to: IsoDate,
	weekdays: readonly number[],
): IsoDate[] {
	const chosen = new Set(weekdays);
	return datesInRange(from, to).filter((date) => chosen.has(weekdayIndex(date)));
}

export function shiftWeeks(weekStart: IsoDate, delta: number): IsoDate {
	return shiftDays(weekStart, delta * 7);
}

const weekday = new Intl.DateTimeFormat(undefined, { weekday: 'short', timeZone: 'UTC' });
const dayMonth = new Intl.DateTimeFormat(undefined, {
	day: 'numeric',
	month: 'short',
	timeZone: 'UTC',
});
const fullDate = new Intl.DateTimeFormat(undefined, {
	weekday: 'long',
	day: 'numeric',
	month: 'long',
	year: 'numeric',
	timeZone: 'UTC',
});

export function weekdayLabel(date: IsoDate): string {
	return weekday.format(parse(date));
}

export function dayMonthLabel(date: IsoDate): string {
	return dayMonth.format(parse(date));
}

export function fullDateLabel(date: IsoDate): string {
	return fullDate.format(parse(date));
}

export function weekLabel(weekStart: IsoDate): string {
	const { from, to } = weekRange(weekStart);
	return `${dayMonthLabel(from)} – ${dayMonthLabel(to)}`;
}

export function isToday(date: IsoDate): boolean {
	return date === todayIso();
}

/** True for a well-formed YYYY-MM-DD that names a real day. */
export function isIsoDate(value: string | null | undefined): value is IsoDate {
	if (!value || !/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
	return format(parse(value)) === value;
}
