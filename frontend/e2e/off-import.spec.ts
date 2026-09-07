import { expect, test } from '@playwright/test';

// Tagged @network and excluded from the default run: a smoke suite should not fail
// because a third party is unreachable. Run it with `mise run frontend:e2e:network`.
test('@network importing a catalog entry credits and links to the source', async ({ page }) => {
	await page.goto('/products/search?q=oats');
	await expect(page.getByRole('button', { name: 'Import' }).first()).toBeVisible({
		timeout: 30_000,
	});
	await page.getByRole('button', { name: 'Import' }).first().click();

	// The import lands in the product's own editor: the catalog's names are messy and
	// this snapshot is never re-read from the source afterwards (PR-8).
	await expect(page).toHaveURL(/\/products\/[^/]+\/edit$/);
	await expect(page.getByLabel('Name', { exact: true })).not.toHaveValue('');

	// Requirements §6: imported data is credited and links to the source wherever it is
	// shown, the editor included.
	await expect(page.getByText('Imported from')).toBeVisible();
	await expect(page.getByRole('link', { name: /Open Food Facts/ })).toHaveAttribute(
		'href',
		/world\.openfoodfacts\.org\/product\/.+/,
	);

	await page.getByRole('button', { name: 'Save product' }).click();
	await expect(page).toHaveURL(/\/products\/[^/]+$/);
	await expect(page.getByText('Imported from')).toBeVisible();
});
