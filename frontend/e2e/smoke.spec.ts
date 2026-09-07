import { type APIRequestContext, expect, test } from '@playwright/test';

// One run builds a product, a meal that uses it, a plan that groups the meal, and
// puts that plan on a date — the dependency order the requirements describe. Names
// carry a run id so repeated runs against the same database do not collide.
const run = Date.now().toString().slice(-6);
const PRODUCT = `E2E Oats ${run}`;
const MEAL = `E2E Porridge ${run}`;
const PLAN = `E2E Training day ${run}`;

/** The Monday of the current week, the same way the app computes it. */
function thisMonday(): string {
	const now = new Date();
	const today = new Date(Date.UTC(now.getFullYear(), now.getMonth(), now.getDate()));
	today.setUTCDate(today.getUTCDate() - ((today.getUTCDay() + 6) % 7));
	return today.toISOString().slice(0, 10);
}

test.describe.configure({ mode: 'serial' });

const API = 'http://localhost:8080/v1';

/** The calendar shows four weeks at a time and pages by the whole block. */
const DAYS_IN_VIEW = 28;

/**
 * The suite runs against a real, persistent database, so a block a previous run left
 * assignments in is not empty. Anything asserting "0 of 28" clears its own block first.
 */
async function clearBlock(request: APIRequestContext, monday: string) {
	const start = new Date(`${monday}T00:00:00Z`);
	for (let day = 0; day < DAYS_IN_VIEW; day++) {
		const date = new Date(start.getTime() + day * 86_400_000).toISOString().slice(0, 10);
		await request.delete(`${API}/calendar/${date}`);
	}
}

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

test('a decimal typed with a comma is kept, not silently dropped', async ({ page }) => {
	await page.goto('/products/new');
	await page.getByLabel('Name').fill(`E2E Comma ${run}`);
	await page.getByLabel('Energy').fill('380');
	await page.getByLabel('Protein').fill('13');
	// A European keyboard writes this; Number('7,5') is NaN, which used to empty the field.
	await page.getByLabel('Fat').fill('7,5');
	await page.getByLabel('Carbs').fill('60');
	await page.getByRole('button', { name: 'Create product' }).click();

	await expect(page.getByRole('heading', { name: `E2E Comma ${run}` })).toBeVisible();
	await expect(page.getByText('7.5 g')).toBeVisible();
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

test('the product list sorts by a macro, and says so', async ({ page }) => {
	await page.goto('/products');
	await page.getByRole('button', { name: /^Energy/ }).click();

	await expect(page).toHaveURL(/sort=energyKcal%3Adesc/);
	// The ordering is announced, not only drawn.
	await expect(page.locator('th', { hasText: 'Energy' })).toHaveAttribute(
		'aria-sort',
		'descending',
	);
});

test('the meal list is a table that sorts like the product list', async ({ page }) => {
	await page.goto('/meals');
	await page.getByRole('button', { name: /^Protein/ }).click();

	await expect(page).toHaveURL(/sort=proteinG%3Adesc/);
	await expect(page.locator('th', { hasText: 'Protein' })).toHaveAttribute(
		'aria-sort',
		'descending',
	);
});

test('a meal derives its macros from its servings', async ({ page }) => {
	await page.goto('/meals/new');
	await page.getByLabel('Label').fill(MEAL);

	// Picking the product is what creates the row: there is no empty row to fill first,
	// and focus lands in the amount so the run of typing continues.
	await page.getByRole('combobox', { name: 'Add a product…' }).click();
	await page.getByRole('option', { name: PRODUCT }).click();
	const amount = page.getByLabel(`Amount of ${PRODUCT}`);
	await expect(amount).toBeFocused();

	// 50 g of a product listed per 100 g is half of it: 380 kcal -> 190 kcal.
	await amount.fill('50');
	await expect(page.getByText('190 kcal').first()).toBeVisible();

	await page.getByRole('button', { name: 'Create meal' }).click();
	await expect(page.getByRole('heading', { name: MEAL })).toBeVisible();
	await expect(page.getByText('190 kcal').first()).toBeVisible();
});

test('a day plan sums its meals and is measured against the targets', async ({ page }) => {
	await page.goto('/day-plans/new');
	await page.getByLabel('Label').fill(PLAN);
	await page.getByRole('button', { name: 'Add meal' }).click();
	await page.getByRole('option', { name: new RegExp(MEAL) }).click();
	await page.getByRole('button', { name: 'Create day plan' }).click();

	await expect(page.getByRole('heading', { name: PLAN })).toBeVisible();
	await expect(page.getByText('190 kcal').first()).toBeVisible();
	// Targets are shown alongside, and nothing about being under them blocks anything.
	await expect(page.getByText(/of 2,?400 kcal/).first()).toBeVisible();
});

test('a plan can be put on a date, replaced and cleared', async ({ page, request }) => {
	await clearBlock(request, thisMonday());
	await page.goto('/calendar');
	await expect(page.getByText('0 of 28 days planned')).toBeVisible();

	await page.getByRole('button', { name: 'Assign' }).first().click();
	await page.getByRole('option', { name: new RegExp(PLAN) }).click();
	await expect(page.getByText('1 of 28 days planned')).toBeVisible();
	await expect(page.getByRole('link', { name: PLAN })).toBeVisible();

	// A day holds one plan, so assigning again replaces rather than adds (CL-3).
	await page
		.getByRole('button', { name: /^Change / })
		.first()
		.click();
	await page.getByRole('menuitem', { name: 'Replace plan' }).click();
	await page.getByRole('option', { name: new RegExp(PLAN) }).click();
	await expect(page.getByText('1 of 28 days planned')).toBeVisible();

	await page
		.getByRole('button', { name: /^Change / })
		.first()
		.click();
	await page.getByRole('menuitem', { name: 'Clear day' }).click();
	await expect(page.getByText('0 of 28 days planned')).toBeVisible();
});

test('an unplanned day reads as unplanned and stays out of the denominator', async ({
	page,
	request,
}) => {
	// The app's best idea, and the thing no shipped planner gets right: an empty day
	// must never read as 0 kcal, which would look like a catastrophic shortfall.
	await clearBlock(request, '2026-02-02');
	await page.goto('/calendar?week=2026-02-02');
	await expect(page.getByText('0 of 28 days planned')).toBeVisible();
	await expect(page.getByText('Unplanned').first()).toBeVisible();

	await page.getByRole('button', { name: 'Assign' }).first().click();
	await page.getByRole('option', { name: new RegExp(PLAN) }).click();
	await expect(page.getByText('1 of 28 days planned')).toBeVisible();

	// The per-day average divides by the one day planned, not by seven.
	await expect(page.getByText('190 kcal').first()).toBeVisible();
});

test('one plan can be applied to several weekdays at once', async ({ page, request }) => {
	await clearBlock(request, '2026-03-02');
	await page.goto('/calendar?week=2026-03-02');
	await expect(page.getByText('0 of 28 days planned')).toBeVisible();

	await page.getByRole('button', { name: 'Apply a plan' }).click();
	await page.getByRole('button', { name: 'Day plan' }).click();
	await page.getByRole('option', { name: new RegExp(PLAN) }).click();

	// Scoped to one week of the four on screen, so the count is checkable.
	await page.getByRole('button', { name: 'Weeks' }).click();
	await page.getByRole('option').nth(1).click();

	await page.getByRole('button', { name: 'Mon' }).click();
	await page.getByRole('button', { name: 'Wed' }).click();
	await page.getByRole('button', { name: 'Fri' }).click();

	// The dates it will touch are named before anything is written.
	await expect(page.getByText('3 days will be assigned.')).toBeVisible();
	await page.getByRole('button', { name: 'Apply plan' }).click();

	await expect(page.getByText('3 of 28 days planned')).toBeVisible();
});

test('the calendar pages between weeks through the URL', async ({ page }) => {
	// Labels are rendered through Intl in the browser's locale, so the expected text
	// is derived the same way rather than hardcoded.
	const label = (iso: string) =>
		new Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short', timeZone: 'UTC' }).format(
			new Date(`${iso}T00:00:00Z`),
		);

	// A Monday well away from today, so the way back to the current week is offered.
	// The block spans four weeks, so 5 Jan runs to 1 Feb.
	await page.goto('/calendar?week=2026-01-05');
	await expect(page.getByText(`${label('2026-01-05')} – ${label('2026-02-01')}`)).toBeVisible();
	await expect(page.getByRole('link', { name: 'This week' })).toBeVisible();

	// Paging moves the whole block: four weeks on, not one.
	await page.getByTitle('Next 4 weeks').click();
	await expect(page).toHaveURL(/week=2026-02-02/);
	await expect(page.getByText(`${label('2026-02-02')} – ${label('2026-03-01')}`)).toBeVisible();

	await page.getByTitle('Previous 4 weeks').click();
	await expect(page).toHaveURL(/week=2026-01-05/);
});

test('the command palette goes to a screen without touching the mouse', async ({ page }) => {
	await page.goto('/calendar');
	// The shortcut is a window listener, so it only exists once the page has hydrated.
	await expect(page.getByText(/of 28 days planned/)).toBeVisible();
	await page.keyboard.press('ControlOrMeta+k');
	await expect(page.getByPlaceholder('Go to a screen, or start something new…')).toBeVisible();

	await page.getByRole('option', { name: 'Products' }).click();
	await expect(page).toHaveURL(/\/products$/);
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
	await page.getByRole('combobox', { name: 'Add a product…' }).click();
	await page.getByRole('option', { name: doomed }).click();
	await page.getByLabel(`Amount of ${doomed}`).fill('100');
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
