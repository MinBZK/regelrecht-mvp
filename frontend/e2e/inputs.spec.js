import { test, expect } from '@playwright/test';
import { interceptLaw, gotoEditor, selectArticle, readYamlPane } from './helpers.js';

async function waitForSheet(page) {
  await page.waitForFunction(() => {
    const sheet = document.querySelector('rr-sheet');
    if (!sheet) return false;
    const dialog = sheet.shadowRoot?.querySelector('dialog');
    return dialog?.open ?? false;
  }, { timeout: 5000 });
  await page.waitForTimeout(200);
}

async function fillSheetTextField(page, labelText, value) {
  const sheet = page.locator('rr-sheet');
  const listItem = sheet.locator(`rr-list-item:has(rr-text-cell:has-text("${labelText}"))`);
  const input = listItem.locator('rr-text-field input');
  await input.evaluate((el, val) => {
    el.value = val;
    el.dispatchEvent(new Event('input', { bubbles: true }));
  }, value);
}

async function selectSheetDropdown(page, labelText, value) {
  const sheet = page.locator('rr-sheet');
  const listItem = sheet.locator(`rr-list-item:has(rr-text-cell:has-text("${labelText}"))`);
  const select = listItem.locator('select');
  await select.evaluate((el, val) => {
    el.value = val;
    el.dispatchEvent(new Event('change', { bubbles: true }));
  }, value);
}

async function saveSheet(page) {
  const sheet = page.locator('rr-sheet');
  const btn = sheet.locator('rr-button:has-text("Opslaan")');
  await btn.evaluate(el => el.click());
  await page.waitForTimeout(300);
}

test.describe('Inputs with sources', () => {
  test.beforeEach(async ({ page }) => {
    await interceptLaw(page, 'zorgtoeslagwet', 'zorgtoeslag-stripped.yaml');
    await gotoEditor(page);
  });

  test('add input with source reference', async ({ page }) => {
    await selectArticle(page, '2');

    // Init machine_readable
    await page.locator('[data-testid="init-mr-btn"]').click();
    await page.waitForTimeout(300);

    // Add input: leeftijd from wet_basisregistratie_personen
    await page.locator('rr-button:has-text("Nieuwe input")').click();
    await waitForSheet(page);

    await fillSheetTextField(page, 'Naam', 'leeftijd');
    await selectSheetDropdown(page, 'Type', 'number');
    await fillSheetTextField(page, 'Bron regelgeving', 'wet_basisregistratie_personen');
    await fillSheetTextField(page, 'Bron output', 'leeftijd');
    await saveSheet(page);

    // Verify YAML
    const yaml = await readYamlPane(page);
    expect(yaml.execution.input).toHaveLength(1);
    expect(yaml.execution.input[0].name).toBe('leeftijd');
    expect(yaml.execution.input[0].type).toBe('number');
    expect(yaml.execution.input[0].source.regulation).toBe('wet_basisregistratie_personen');
    expect(yaml.execution.input[0].source.output).toBe('leeftijd');
  });
});
