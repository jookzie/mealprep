import { expect, test } from '@playwright/test';

// One run builds a product, a meal that uses it, a plan that groups the meal, and
// puts that plan on a date — the dependency order the requirements describe. Names
// carry a run id so repeated runs against the same database do not collide.
const run = Date.now().toString().slice(-6);
const PRODUCT = `E2E Oats ${run}`;
const MEAL = `E2E Porridge ${run}`;
const PLAN = `E2E Training day ${run}`;

test.describe.configure({ mode: 'serial' });

test('the backend is reachable', async ({ request }) => {
	const response = await request.get('http://localhost:8080/livez');
	expect(response.ok(), 'start the backend with `mise run backend:start`').toBeTruthy();
});

test('targets can be set and are shown back', async ({ page }) => {
	await page.goto('/targets');
	await page.getByLabel('Energy').fill('2400');
	await page.getByLabel('Protein').fill('160');
	await page.getByLabel('Fat').fill('70');
	await page.getByLabel('Carbs').fill('260');
	await page.getByRole('button', { name: /Set targets|Save targets/ }).click();

	await expect(page.getByText('Current targets')).toBeVisible();
	await expect(page.getByText('2,400 kcal').or(page.getByText('2400 kcal'))).toBeVisible();
});

test('a product can be created by hand', async ({ page }) => {
	await page.goto('/products/new');
	await page.getByLabel('Name').fill(PRODUCT);
	await page.getByLabel('Energy').fill('380');
	await page.getByLabel('Protein').fill('13');
	await page.getByLabel('Fat').fill('7');
	await page.getByLabel('Carbs').fill('60');
	await page.getByRole('button', { name: 'Create product' }).click();

	await expect(page.getByRole('heading', { name: PRODUCT })).toBeVisible();
	// A hand-created product carries no source, so nothing is credited (PR-5).
	await expect(page.getByText('Imported from')).toBeHidden();

	await page.goto('/products');
	await expect(page.getByRole('link', { name: PRODUCT })).toBeVisible();
});

test('a meal derives its macros from its servings', async ({ page }) => {
	await page.goto('/meals/new');
	await page.getByLabel('Label').fill(MEAL);
	await page.getByRole('button', { name: 'Add serving' }).click();
	await page.getByRole('combobox').click();
	await page.getByRole('option', { name: PRODUCT }).click();
	await page.locator('input[type="number"]').first().fill('50');
	await page.getByRole('button', { name: 'Create meal' }).click();

	await expect(page.getByRole('heading', { name: MEAL })).toBeVisible();
	// 50 g of a product listed per 100 g is half of it: 380 kcal -> 190 kcal.
	await expect(page.getByText('190 kcal').first()).toBeVisible();
});

test('a day plan sums its meals and is measured against the targets', async ({ page }) => {
	await page.goto('/day-plans/new');
	await page.getByLabel('Label').fill(PLAN);
	await page.getByRole('button', { name: 'Add meal' }).click();
	await page.getByRole('option', { name: new RegExp(MEAL) }).click();
	await page.keyboard.press('Escape');
	await page.getByRole('button', { name: 'Create day plan' }).click();

	await expect(page.getByRole('heading', { name: PLAN })).toBeVisible();
	await expect(page.getByText('190 kcal').first()).toBeVisible();
	// Targets are shown alongside, and nothing about being under them blocks anything.
	await expect(page.getByText(/of 2,?400 kcal/)).toBeVisible();
});

test('a plan can be put on a date, replaced and cleared', async ({ page }) => {
	await page.goto('/calendar');
	await expect(page.getByText('0 of 7 days planned')).toBeVisible();

	await page.getByRole('button', { name: 'Assign a plan' }).first().click();
	await page.getByRole('option', { name: new RegExp(PLAN) }).click();
	await expect(page.getByText('1 of 7 days planned')).toBeVisible();
	await expect(page.getByRole('link', { name: PLAN })).toBeVisible();

	// A day holds one plan, so assigning again replaces rather than adds (CL-3).
	await page.getByRole('button', { name: 'Replace' }).first().click();
	await page.getByRole('option', { name: new RegExp(PLAN) }).click();
	await expect(page.getByText('1 of 7 days planned')).toBeVisible();

	await page.getByRole('button', { name: 'Clear' }).first().click();
	await expect(page.getByText('0 of 7 days planned')).toBeVisible();
});

test('the calendar pages between weeks through the URL', async ({ page }) => {
	// Labels are rendered through Intl in the browser's locale, so the expected text
	// is derived the same way rather than hardcoded.
	const label = (iso: string) =>
		new Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short', timeZone: 'UTC' }).format(
			new Date(`${iso}T00:00:00Z`),
		);

	// A Monday well away from today, so the way back to the current week is offered.
	await page.goto('/calendar?week=2026-01-05');
	await expect(page.getByText(`${label('2026-01-05')} – ${label('2026-01-11')}`)).toBeVisible();
	await expect(page.getByRole('link', { name: 'This week' })).toBeVisible();

	await page.getByTitle('Next week').click();
	await expect(page).toHaveURL(/week=2026-01-12/);
	await expect(page.getByText(`${label('2026-01-12')} – ${label('2026-01-18')}`)).toBeVisible();
});

test('a meal whose product was deleted still renders', async ({ page }) => {
	// Products are soft-deleted and then never returned, so this join misses. The meal
	// must still open, with the gap named rather than swallowed.
	const doomed = `E2E Doomed ${run}`;
	const meal = `E2E Orphaned ${run}`;

	await page.goto('/products/new');
	await page.getByLabel('Name').fill(doomed);
	await page.getByLabel('Energy').fill('100');
	await page.getByLabel('Protein').fill('10');
	await page.getByLabel('Fat').fill('5');
	await page.getByLabel('Carbs').fill('5');
	await page.getByRole('button', { name: 'Create product' }).click();
	await expect(page.getByRole('heading', { name: doomed })).toBeVisible();
	const productUrl = page.url();

	await page.goto('/meals/new');
	await page.getByLabel('Label').fill(meal);
	await page.getByRole('button', { name: 'Add serving' }).click();
	await page.getByRole('combobox').click();
	await page.getByRole('option', { name: doomed }).click();
	await page.locator('input[type="number"]').first().fill('100');
	await page.getByRole('button', { name: 'Create meal' }).click();
	await expect(page.getByRole('heading', { name: meal })).toBeVisible();
	const mealUrl = page.url();

	await page.goto(productUrl);
	await page.getByRole('button', { name: 'Delete' }).first().click();
	await page.getByRole('button', { name: 'Delete', exact: true }).last().click();
	await expect(page).toHaveURL(/\/products$/);

	await page.goto(mealUrl);
	await expect(page.getByText('Product removed')).toBeVisible();
	await expect(page.getByText(/product has been deleted/)).toBeVisible();
});
