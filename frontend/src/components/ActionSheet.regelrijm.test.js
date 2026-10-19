// The action sheet behind the flag panel.regelrijm: it reads the selected part
// as RegelRijm text, and opens at the part a reading in the Machine pane
// pointed at.
import { describe, it, expect, vi, beforeAll } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';

vi.mock('../composables/useFeatureFlags.js', () => ({ useFeatureFlags: () => ({ isEnabled: () => true }) }));
vi.mock('../composables/useLaw.js', () => ({ fetchLaw: vi.fn(async () => null) }));
const { default: ActionSheet } = await import('./ActionSheet.vue');

const ACTION = { output: 'hoogte', value: { operation: 'MAX', values: [0, { operation: 'MULTIPLY', values: ['$inkomen', '$percentage'] }] } };
const ARTICLE = { number: '2', machine_readable: { execution: { actions: [ACTION] } } };

beforeAll(() => {
  // jsdom does not know nldd-sheet's show() and hide().
  HTMLElement.prototype.show ??= () => {};
  HTMLElement.prototype.hide ??= () => {};
});

const reading = (wrapper) => wrapper.find('[data-testid="action-sheet-regelrijm"] [data-testid="regelrijm-text"]').text();

describe('ActionSheet with RegelRijm', () => {
  it('reads the whole action when it opens at the top', async () => {
    const wrapper = mount(ActionSheet, { props: { action: ACTION, article: ARTICLE } });
    await flushPromises();
    expect(reading(wrapper)).toBe('[hoogte] wordt: de grootste van ( 0; [inkomen] maal [percentage] )');
  });

  it('opens at the part a reading pointed at', async () => {
    const wrapper = mount(ActionSheet, { props: { action: ACTION, article: ARTICLE, focusPath: ['value', 'values', 1] } });
    await flushPromises();
    expect(reading(wrapper)).toBe('[inkomen] maal [percentage]');
  });
});
