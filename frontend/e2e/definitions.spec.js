import { test, expect } from '@playwright/test';
import { interceptLaw, gotoEditor, selectArticle, readYamlPane } from './helpers.js';

/**
 * Helper to wait for rr-sheet to be open (the component uses a dialog element internally).
 */
async function waitForSheet(page) {
  // The rr-sheet opens a <dialog> in its shadow DOM. Wait for the sheet to show.
  await page.waitForFunction(() => {
    const sheet = document.querySelector('rr-sheet');
    if (!sheet) return false;
    // Check if the sheet's internal dialog is open
    const dialog = sheet.shadowRoot?.querySelector('dialog');
    return dialog?.open ?? false;
  }, { timeout: 5000 });
  await page.waitForTimeout(200);
}

/**
 * Fill an rr-text-field's internal input, forcing interaction even if Playwright considers it hidden.
 */
async function fillSheetTextField(page, labelText, value) {
  const sheet = page.locator('rr-sheet');
  const listItem = sheet.locator(`rr-list-item:has(rr-text-cell:has-text("${labelText}"))`);
  const input = listItem.locator('rr-text-field input');
  // Use evaluate to directly set value and dispatch event
  await input.evaluate((el, val) => {
    el.value = val;
    el.dispatchEvent(new Event('input', { bubbles: true }));
  }, value);
}

/**
 * Fill an rr-number-field's internal input.
 */
async function fillSheetNumberField(page, labelText, value) {
  const sheet = page.locator('rr-sheet');
  const listItem = sheet.locator(`rr-list-item:has(rr-text-cell:has-text("${labelText}"))`);
  // The rr-number-field dispatches a custom 'change' event with detail.value
  // We need to set the value on the component and trigger the change
  const numberField = listItem.locator('rr-number-field');
  await numberField.evaluate((el, val) => {
    // Set internal input value
    const input = el.shadowRoot?.querySelector('input') ?? el.querySelector('input');
    if (input) {
      input.value = String(val);
      input.dispatchEvent(new Event('input', { bubbles: true }));
      input.dispatchEvent(new Event('change', { bubbles: true }));
    }
    // Also dispatch the custom event that Vue listens for
    el.dispatchEvent(new CustomEvent('change', { detail: { value: Number(val) }, bubbles: true }));
  }, value);
}

/**
 * Select option in rr-dropdown within sheet.
 */
async function selectSheetDropdown(page, labelText, value) {
  const sheet = page.locator('rr-sheet');
  const listItem = sheet.locator(`rr-list-item:has(rr-text-cell:has-text("${labelText}"))`);
  const select = listItem.locator('select');
  await select.evaluate((el, val) => {
    el.value = val;
    el.dispatchEvent(new Event('change', { bubbles: true }));
  }, value);
}

/**
 * Click Save button in sheet.
 */
async function saveSheet(page) {
  const sheet = page.locator('rr-sheet');
  const btn = sheet.locator('rr-button:has-text("Opslaan")');
  await btn.evaluate(el => el.click());
  await page.waitForTimeout(300);
}

test.describe('Definitions', () => {
  test.beforeEach(async ({ page }) => {
    await interceptLaw(page, 'zorgtoeslagwet', 'zorgtoeslag-stripped.yaml');
    await gotoEditor(page);
  });

  test('add a numeric definition to article 1a', async ({ page }) => {
    await selectArticle(page, '1a');

    // Init machine_readable
    await page.locator('[data-testid="init-mr-btn"]').click();
    await page.waitForTimeout(300);

    // Click "Nieuwe definitie" button
    await page.locator('rr-button:has-text("Nieuwe definitie")').click();
    await waitForSheet(page);

    // Set name
    await fillSheetTextField(page, 'Naam', 'verantwoordelijke_autoriteit');

    // Set value (new definitions default to number)
    await fillSheetNumberField(page, 'Waarde', 42);

    // Save
    await saveSheet(page);

    // Verify the definition appears in the machine readable view
    const mrPane = page.locator('[data-testid="machine-readable"]');
    await expect(mrPane).toContainText('verantwoordelijke_autoriteit');

    // Verify YAML
    const yaml = await readYamlPane(page);
    expect(yaml.definitions).toHaveProperty('verantwoordelijke_autoriteit');
    // New definitions from EditSheet always store as {value: N} format
    expect(yaml.definitions.verantwoordelijke_autoriteit).toEqual({ value: 42 });
  });

  test('add numeric definitions to article 2', async ({ page }) => {
    await selectArticle(page, '2');

    // Init machine_readable
    await page.locator('[data-testid="init-mr-btn"]').click();
    await page.waitForTimeout(300);

    // Add definition: drempelinkomen_alleenstaande = 3971900
    await page.locator('rr-button:has-text("Nieuwe definitie")').click();
    await waitForSheet(page);

    await fillSheetTextField(page, 'Naam', 'drempelinkomen_alleenstaande');
    await fillSheetNumberField(page, 'Waarde', 3971900);
    await saveSheet(page);

    // Verify definition appears
    const mrPane = page.locator('[data-testid="machine-readable"]');
    await expect(mrPane).toContainText('drempelinkomen_alleenstaande');

    // Verify YAML
    const yaml = await readYamlPane(page);
    expect(yaml.definitions.drempelinkomen_alleenstaande).toEqual({ value: 3971900 });
  });
});
