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
	/**
	 * Set when this plan is one date's own copy of another, made so that editing that
	 * date could not move every other date sharing the original. A copy is absent from
	 * listDayPlans and from every picker; only the calendar read resolves one.
	 */
	sourcePlanId?: string;
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

/** A span of dates an operation acts on, both ends included. */
export type DateRange = {
	from: string;
	to: string;
};

/** Which entry of a day's order an operation names. */
export type ItemRef = { kind: 'meal'; mealId: string } | { kind: 'product'; productId: string };

/**
 * One typed edit the assistant may propose. The model answers with these and never
 * writes: the rules check every one against the real data, and the user confirms.
 */
export type Operation =
	| { kind: 'adjust-product-amount'; dates: DateRange; productId: string; deltaAmount: number }
	| { kind: 'add-item'; dates: DateRange; item: DayPlanItemDraft }
	| { kind: 'remove-item'; dates: DateRange; item: ItemRef }
	| { kind: 'swap-meal'; dates: DateRange; fromMealId: string; toMealId: string }
	| { kind: 'assign-plan'; dates: DateRange; dayPlanId: string }
	| { kind: 'clear-plan'; dates: DateRange }
	| { kind: 'create-product'; newId: string; product: ProductDraft }
	| { kind: 'update-product'; productId: string; changes: ProductPatch }
	| { kind: 'create-meal'; newId: string; meal: MealDraft }
	| { kind: 'update-meal'; mealId: string; changes: MealPatch }
	| { kind: 'create-day-plan'; newId: string; dayPlan: DayPlanDraft }
	| { kind: 'update-day-plan'; dayPlanId: string; changes: DayPlanPatch }
	| { kind: 'sync-product'; productId: string; code?: string }
	| { kind: 'update-targets'; changes: TargetsPatch };

/**
 * The fields of a product an update changes; what is left out keeps its value, and `null`
 * removes a field a product can lack.
 */
export type ProductPatch = {
	name?: string;
	brand?: string | null;
	unit?: Unit;
	energyKcal?: number;
	proteinG?: number;
	fatG?: number;
	carbohydratesG?: number;
	cost?: number | null;
	/** Grams per 100 of the unit; `null` removes one. */
	nutrients?: Record<string, number | null>;
};

/** The parts of a meal an update changes; servings, when given, replace them all. */
export type MealPatch = {
	label?: string;
	/** `null` takes the meal out of its category. */
	categoryId?: string | null;
	servings?: Serving[];
};

/** The parts of a day plan an update changes; items, when given, replace them all. */
export type DayPlanPatch = {
	label?: string;
	/** `null` takes the plan out of its category. */
	categoryId?: string | null;
	items?: DayPlanItemDraft[];
};

/** The daily targets an update changes; what is left out keeps its value. */
export type TargetsPatch = {
	energyKcal?: number;
	proteinG?: number;
	fatG?: number;
	carbohydratesG?: number;
};

export type EntityKind = 'product' | 'meal' | 'day-plan' | 'targets';
export type EntityAction = 'create' | 'update';

export type OperationKind = Operation['kind'];

/** What the assistant answered. Nothing in it has been checked yet. */
export type Proposal = {
	request: string;
	rationale: string;
	operations: Operation[];
};

/** What became of an earlier reply: nothing to apply, applied, or discarded. */
export type TurnOutcome = 'answered' | 'applied' | 'discarded';

/** One earlier exchange of the conversation a request continues. Only the words travel. */
export type ChatTurn = {
	request: string;
	reply: string;
	outcome: TurnOutcome;
};

/** What one date would become. */
export type PreviewDay = {
	date: string;
	/** Absent when the date would be left unplanned. */
	label?: string;
	changes: string[];
	before: Macros;
	after: Macros;
	/** `after` minus the targets; absent when no targets are set. */
	targetDelta?: Macros;
	/** Whether applying detaches this date onto its own copy of the plan. */
	detaches: boolean;
};

/** What one product, meal, day plan or the targets would become. */
export type PreviewEntity = {
	kind: EntityKind;
	action: EntityAction;
	label: string;
	changes: string[];
	/** Per 100 g or ml for a product; the whole meal or day for a meal or plan; per day for the targets. */
	macros: Macros;
	/** How many dates in view already use it, and so follow an update. */
	datesUsing: number;
};

/** A checked proposal, as the user reads it before confirming. */
export type Preview = {
	proposal: Proposal;
	/** The products, meals and day plans it creates or changes. */
	entities: PreviewEntity[];
	days: PreviewDay[];
	targets?: Macros;
};

/** What an applied request did to one date, and what it takes to put it back. */
export type AppliedChange = {
	date: string;
	previousDayPlanId?: string;
	previousLabel?: string;
	dayPlanId?: string;
	label?: string;
	detached: boolean;
	changes: string[];
};

/** What an applied request did to one product, meal or day plan. */
export type AppliedEntityChange = {
	kind: EntityKind;
	action: EntityAction;
	id: string;
	label: string;
	changes: string[];
};

export type AssistantRecord = {
	id: string;
	request: string;
	rationale: string;
	operations: Operation[];
	changes: AppliedChange[];
	/** Empty in records from before the assistant could create or change these. */
	entityChanges: AppliedEntityChange[];
	createdAt: string;
};

/** Whether the assistant has a key. The key itself is never returned in full. */
export type AssistantKeyStatus = {
	present: boolean;
	/** The last four characters of the stored key. */
	hint?: string;
};

export type CommandErrorKind =
	| 'assistant-unavailable'
	| 'catalogue-unavailable'
	| 'conflict'
	| 'internal'
	| 'invalid'
	| 'not-found';

export type CommandError = {
	kind: CommandErrorKind;
	message: string;
};
