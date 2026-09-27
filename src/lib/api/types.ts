// The shapes the Tauri commands exchange. They mirror the serde derives in
// crates/core/src/domain, which is the source of truth.

export type Unit = 'g' | 'ml';

export type Macros = {
	energyKcal: number;
	fatG: number;
	proteinG: number;
	carbohydratesG: number;
};

export type Nutrients = Record<string, number>;

export type Cost = {
	amount: number;
	complete: boolean;
};

export type Product = {
	id: string;
	name: string;
	unit: Unit;
	macros: Macros;
	nutrients: Nutrients;
	cost?: number;
	brand?: string;
	sourceCode?: string;
	createdAt: string;
	updatedAt: string;
};

export type ProductDraft = {
	name: string;
	unit: Unit;
	macros: Macros;
	nutrients?: Nutrients;
	cost?: number;
	brand?: string;
};

/** Who supplied a catalogue entry, as far as the catalogue can tell. */
export type EntrySource = 'manufacturer' | 'community';

export type CatalogueEntry = {
	code: string;
	name: string;
	unit: Unit;
	macros: Macros;
	nutrients: Nutrients;
	brand?: string;
	imageUrl?: string;
	/** The macros the catalogue lacks; they read as zero in `macros`. */
	missingMacros: (keyof Macros)[];
	source: EntrySource;
	/** The catalogue's own complaints about the figures. */
	warnings: string[];
	modifiedAt?: string;
};

export type Serving = {
	productId: string;
	amount: number;
};

export type Meal = {
	id: string;
	label: string;
	categoryId?: string;
	servings: Serving[];
	macros: Macros;
	cost: Cost;
	createdAt: string;
	updatedAt: string;
};

export type MealDraft = {
	label: string;
	categoryId?: string;
	servings: Serving[];
};

export type CategoryScope = 'meal' | 'day-plan';

export type Category = {
	id: string;
	name: string;
	scope: CategoryScope;
	createdAt: string;
	updatedAt: string;
};

export type CategoryDraft = {
	name: string;
	scope: CategoryScope;
};

export type DayPlanProduct = {
	id: string;
	name?: string;
	unit?: Unit;
	amount: number;
	macros: Macros;
	cost: Cost;
};

export type DayPlanItem = { kind: 'meal'; meal: Meal } | { kind: 'product'; product: DayPlanProduct };

export type DayPlanItemDraft =
	| { kind: 'meal'; mealId: string }
	| { kind: 'product'; productId: string; amount: number };

export type DayPlan = {
	id: string;
	label: string;
	categoryId?: string;
	items: DayPlanItem[];
	macros: Macros;
	cost: Cost;
	createdAt: string;
	updatedAt: string;
};

export type DayPlanDraft = {
	label: string;
	categoryId?: string;
	items: DayPlanItemDraft[];
};

export type CalendarDay = {
	date: string;
	dayPlanId: string;
	createdAt: string;
	updatedAt: string;
};

export type Targets = {
	macros: Macros;
	createdAt: string;
	updatedAt: string;
};

export type WeightEntry = {
	date: string;
	kilograms: number;
	createdAt: string;
	updatedAt: string;
};

/** One day of the series: what the scale said, if anything, and what the trend says. */
export type WeightPoint = {
	date: string;
	kilograms?: number;
	trend: number;
};

export type WeightSeries = {
	points: WeightPoint[];
	trend?: number;
	ratePerWeek?: number;
};

export type MeasurementKind =
	| 'waist'
	| 'hips'
	| 'chest'
	| 'neck'
	| 'arm'
	| 'thigh'
	| 'height'
	| 'body-fat';

/** One measurement of one kind; a date holds at most one per kind. */
export type Measurement = {
	date: string;
	kind: MeasurementKind;
	value: number;
	createdAt: string;
	updatedAt: string;
};

export type HealthAvailability = 'available' | 'update-required' | 'unsupported';

export type HealthStatus = {
	availability: HealthAvailability;
	connected: boolean;
	syncedAt?: string;
};

/** The night that ended on a morning. Minutes count from that morning's midnight. */
export type NightSleep = {
	bedMinute: number;
	wakeMinute: number;
	asleepMinutes: number;
	lightMinutes?: number;
	deepMinutes?: number;
	remMinutes?: number;
	awakeMinutes?: number;
};

export type SleepNight = NightSleep & { date: string };

export type SleepSummary = {
	nights: SleepNight[];
	averageMinutes?: number;
	typicalBedMinute?: number;
	typicalWakeMinute?: number;
	bedtimeSpreadMinutes?: number;
	wakeSpreadMinutes?: number;
};

export type Marker = 'hrv' | 'resting-heart-rate';

export type Band = { low: number; high: number };

export type BaselineStatus = 'below' | 'within' | 'above';

export type MarkerPoint = {
	date: string;
	value?: number;
	average?: number;
	band?: Band;
};

export type MarkerSeries = {
	marker: Marker;
	points: MarkerPoint[];
	average?: number;
	band?: Band;
	status?: BaselineStatus;
};

export type Behaviour =
	| 'slept-seven-hours'
	| 'earlier-bedtime'
	| 'trained-day-before'
	| 'energy-over-target-day-before'
	| 'protein-at-target-day-before';

export type Impact = {
	behaviour: Behaviour;
	withDays: number;
	withoutDays: number;
	withMean: number;
	withoutMean: number;
	changePercent: number;
	clear: boolean;
};

export type Recovery = {
	hrv?: MarkerSeries;
	restingHeartRate?: MarkerSeries;
	impactMarker?: Marker;
	impacts: Impact[];
};

export type EnergyBalance = {
	from: string;
	to: string;
	windowDays: number;
	plannedDays: number;
	plannedKcal?: number;
	trendChangeKg?: number;
	storedKcal?: number;
	expenditureKcal?: number;
	measuredDays: number;
	measuredKcal?: number;
};

export type CommandErrorKind =
	| 'catalogue-unavailable'
	| 'conflict'
	| 'health-unavailable'
	| 'internal'
	| 'invalid'
	| 'not-found';

export type CommandError = {
	kind: CommandErrorKind;
	message: string;
};
