// The one module the app reaches the Rust side through. Components never call invoke;
// routes read through `read` in their load functions and write through `runMutation`.
import { type InvokeArgs, invoke, isTauri } from '@tauri-apps/api/core';
import type {
	CalendarDay,
	CatalogueEntry,
	Category,
	CategoryDraft,
	CategoryScope,
	DayPlan,
	DayPlanDraft,
	Macros,
	Meal,
	MealDraft,
	Product,
	ProductDraft,
	Targets,
	WeightEntry,
	WeightSeries,
} from './types';

export { errorKind, messageOf, read } from './errors';
export { type MutationOptions, runMutation } from './mutate';
export type * from './types';

const OUTSIDE_APP =
	'Mealprep keeps its data on the device, so it runs inside the app only. Start it with `bun tauri dev` or `bun tauri android dev`.';

function call<T>(command: string, args?: InvokeArgs): Promise<T> {
	if (!isTauri()) return Promise.reject(new Error(OUTSIDE_APP));
	return invoke<T>(command, args);
}

export const listProducts = () => call<Product[]>('list_products');
export const getProduct = (productId: string) => call<Product>('get_product', { productId });
export const createProduct = (draft: ProductDraft) => call<Product>('create_product', { draft });
export const updateProduct = (productId: string, draft: ProductDraft) =>
	call<Product>('update_product', { productId, draft });
export const deleteProduct = (productId: string) => call<void>('delete_product', { productId });
export const searchCatalogue = (query: string) =>
	call<CatalogueEntry[]>('search_catalogue', { query });
export const getCatalogueEntry = (code: string) =>
	call<CatalogueEntry>('get_catalogue_entry', { code });
/** Imports the entry with the given code as the user reviewed it, not as the catalogue has it. */
export const importProduct = (code: string, draft: ProductDraft) =>
	call<Product>('import_product', { code, draft });

export const listMeals = () => call<Meal[]>('list_meals');
export const getMeal = (mealId: string) => call<Meal>('get_meal', { mealId });
export const createMeal = (draft: MealDraft) => call<Meal>('create_meal', { draft });
export const updateMeal = (mealId: string, draft: MealDraft) =>
	call<Meal>('update_meal', { mealId, draft });
export const deleteMeal = (mealId: string) => call<void>('delete_meal', { mealId });

export const listCategories = (scope: CategoryScope) =>
	call<Category[]>('list_categories', { scope });
export const createCategory = (draft: CategoryDraft) =>
	call<Category>('create_category', { draft });
export const renameCategory = (categoryId: string, name: string) =>
	call<Category>('rename_category', { categoryId, name });
export const deleteCategory = (categoryId: string) =>
	call<void>('delete_category', { categoryId });

export const listDayPlans = () => call<DayPlan[]>('list_day_plans');
export const getDayPlan = (dayPlanId: string) => call<DayPlan>('get_day_plan', { dayPlanId });
export const createDayPlan = (draft: DayPlanDraft) => call<DayPlan>('create_day_plan', { draft });
export const updateDayPlan = (dayPlanId: string, draft: DayPlanDraft) =>
	call<DayPlan>('update_day_plan', { dayPlanId, draft });
export const deleteDayPlan = (dayPlanId: string) => call<void>('delete_day_plan', { dayPlanId });

export const listCalendarDays = (from: string, to: string) =>
	call<CalendarDay[]>('list_calendar_days', { from, to });
export const assignCalendarDay = (date: string, dayPlanId: string) =>
	call<CalendarDay>('assign_calendar_day', { date, dayPlanId });
export const unassignCalendarDay = (date: string) => call<void>('unassign_calendar_day', { date });

/** The targets, or null when the user has never set any: an absent singleton is not a failure. */
export const loadTargets = () => call<Targets | null>('get_targets');
export const setTargets = (macros: Macros) => call<Targets>('set_targets', { macros });

export const listWeightEntries = () => call<WeightEntry[]>('list_weight_entries');
export const loadWeightSeries = () => call<WeightSeries>('weight_series');
export const setWeightEntry = (date: string, kilograms: number) =>
	call<WeightEntry>('set_weight_entry', { date, kilograms });
export const deleteWeightEntry = (date: string) => call<void>('delete_weight_entry', { date });
