// RegelRijm in the Machine pane on two small fictional laws: every rule of
// the article as text in full, opening the action at a part, a tooltip on each
// name saying where it comes from, and following a name to the article or
// regelwerk that computes it.
import { describe, it, expect, vi } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';

const WET_B = {
  $id: 'wet_b',
  articles: [{ number: '8', machine_readable: { execution: { actions: [{ output: 'toetsingsinkomen', value: { operation: 'ADD', values: ['$loon', '$winst'] } }] } } }],
};
const ARTIKEL = {
  number: '2',
  machine_readable: {
    definitions: { drempel: { value: 100000 } },
    execution: {
      input: [{ name: 'inkomen', type: 'amount', type_spec: { unit: 'eurocent' }, source: { regulation: 'wet_b', output: 'toetsingsinkomen' } }],
      output: [{ name: 'hoogte', type: 'amount', type_spec: { unit: 'eurocent' } }],
      actions: [
        {
          output: 'hoogte',
          value: { operation: 'MAX', values: [0, { operation: 'MULTIPLY', values: [{ operation: 'SUBTRACT', values: ['$inkomen', '$drempel', '$korting'] }, '$percentage'] }] },
        },
        { output: 'dubbel', value: { operation: 'MULTIPLY', values: ['$hoogte', 2] } },
      ],
    },
  },
};
const WET_A = { $id: 'wet_a', articles: [ARTIKEL, { number: '3', machine_readable: { execution: { actions: [{ output: 'korting', value: 5 }] } } }] };

vi.mock('../composables/useLaw.js', () => ({
  fetchLaw: vi.fn(async (_traject, id) => {
    if (id === 'wet_c') return { law: { $id: 'wet_c', articles: [] }, lawName: 'Wet C' };
    if (id !== 'wet_b') throw new Error('404');
    return { law: WET_B, lawName: 'Wet B' };
  }),
}));

const { default: RegelRijmPane } = await import('./RegelRijmPane.vue');

async function pane(props = {}) {
  const wrapper = mount(RegelRijmPane, { props: { article: ARTIKEL, law: WET_A, lawId: 'wet_a', ...props } });
  await flushPromises();
  return wrapper;
}

// The reading as a reader sees it: a token shows its text attribute.
function ruleText(wrapper, output) {
  const el = wrapper.find(`[data-testid="regelrijm-pane-rule-${output}"] [data-testid="regelrijm-text"]`).element.cloneNode(true);
  for (const token of el.querySelectorAll('nldd-token')) token.replaceWith(token.getAttribute('text'));
  return el.textContent;
}
const hint = (wrapper, output, ref) => wrapper.find(`[data-testid="regelrijm-pane-rule-${output}"] [data-testid="regelrijm-hint-${ref}"]`).attributes('text');

describe('RegelRijmPane', () => {
  it('reads every rule of the article in full', async () => {
    const wrapper = await pane();
    expect(ruleText(wrapper, 'hoogte')).toBe('[hoogte] wordt: de grootste van ( € 0,00; ( inkomen min [drempel] min korting ) maal [percentage] )');
    expect(ruleText(wrapper, 'dubbel')).toBe('[dubbel] wordt: hoogte maal 2');
    expect(wrapper.text()).not.toContain('⟨');
  });

  it('shows only the heading and the add button when the article has no rules', async () => {
    const wrapper = await pane({ article: { number: '9' }, editable: true });
    expect(wrapper.findAll('[data-testid^="regelrijm-pane-rule-"]')).toHaveLength(0);
    await wrapper.find('[data-testid="add-action-btn"]').trigger('click');
    expect(wrapper.emitted('add-action')).toHaveLength(1);
  });

  it('opens the action at the top from its row', async () => {
    const wrapper = await pane();
    await wrapper.find('[data-testid="regelrijm-pane-open-dubbel"]').trigger('click');
    expect(wrapper.emitted('open-action')[0]).toEqual([ARTIKEL.machine_readable.execution.actions[1]]);
  });

  it('opens the action at the part whose operation word is clicked', async () => {
    const wrapper = await pane();
    await wrapper.find('[data-testid="regelrijm-pane-rule-hoogte"] [data-testid="regelrijm-select-value/values/1"]').trigger('click');
    const [action, path] = wrapper.emitted('open-action')[0];
    expect(action).toEqual(ARTIKEL.machine_readable.execution.actions[0]);
    expect(path).toEqual(['value', 'values', 1]);
  });

  it('deletes through the row menu when editable, without opening the row', async () => {
    const wrapper = await pane({ editable: true });
    await wrapper.find('[data-testid="action-dubbel-delete-btn"]').trigger('click');
    expect(wrapper.emitted('delete-action')[0]).toEqual([1]);
    expect(wrapper.emitted('open-action')).toBeUndefined();
  });

  it('says in a tooltip where each name comes from', async () => {
    const wrapper = await pane();
    expect(hint(wrapper, 'hoogte', '$korting')).toBe('korting: berekend in wet a, artikel 3');
    expect(hint(wrapper, 'hoogte', '$inkomen')).toBe('inkomen: berekend in Wet B, artikel 8');
    expect(hint(wrapper, 'hoogte', '$drempel')).toBe('Vaste waarde in dit artikel: 100000');
    expect(hint(wrapper, 'dubbel', '$hoogte')).toBe('hoogte: berekend in dit artikel');
  });

  it('reads a half-edited article rule by rule instead of failing as a whole', async () => {
    const article = structuredClone(ARTIKEL);
    article.machine_readable.execution.actions = [null, { output: 'k', value: { operation: 'IF', cases: 'foo', default: 1 } }, ...article.machine_readable.execution.actions];
    article.machine_readable.execution.parameters = {};
    const wrapper = await pane({ article });
    expect(wrapper.find('[data-testid="regelrijm-pane-rule-0"] h6').text()).toBe('Actie zonder uitkomst');
    expect(ruleText(wrapper, 'k')).toBe('[k] wordt: als ⟨ontbreekt: cases⟩');
    expect(ruleText(wrapper, 'dubbel')).toBe('[dubbel] wordt: hoogte maal 2');
  });

  it('stays in the pane for a rule of the same article, also when the route names the law differently', async () => {
    const wrapper = await pane({ lawId: 'wet-a-in-de-route' });
    await wrapper.find('[data-testid="regelrijm-follow-$hoogte"]').trigger('click');
    await flushPromises();
    expect(wrapper.emitted('navigate')).toBeUndefined();
    await wrapper.find('[data-testid="regelrijm-follow-$korting"]').trigger('click');
    await flushPromises();
    expect(wrapper.emitted('navigate')[0][0]).toEqual({ lawId: 'wet-a-in-de-route', articleNumber: '3' });
  });

  it('says why a name in another regelwerk cannot be followed', async () => {
    const article = structuredClone(ARTIKEL);
    article.machine_readable.execution.input.push(
      { name: 'weg', source: { regulation: 'wet_c', output: 'weg' } },
      { name: 'kwijt', source: { regulation: 'wet_x', output: 'kwijt' } },
    );
    article.machine_readable.execution.actions.push({ output: 'som', value: { operation: 'ADD', values: ['$weg', '$kwijt'] } });
    const wrapper = await pane({ article });
    await wrapper.find('[data-testid="regelrijm-follow-$weg"]').trigger('click');
    await flushPromises();
    expect(wrapper.find('nldd-banner[variant="warning"]').attributes('text')).toBe('weg is in wet c geen uitkomst van een rekenregel.');
    await wrapper.find('[data-testid="regelrijm-follow-$kwijt"]').trigger('click');
    await flushPromises();
    expect(wrapper.find('nldd-banner[variant="warning"]').attributes('text')).toBe('wet x is niet geladen: 404');
    expect(wrapper.emitted('navigate')).toBeUndefined();
  });

  it('follows a name to the article or regelwerk that computes it', async () => {
    const wrapper = await pane();
    await wrapper.find('[data-testid="regelrijm-follow-$korting"]').trigger('click');
    await flushPromises();
    expect(wrapper.emitted('navigate')[0][0]).toEqual({ lawId: 'wet_a', articleNumber: '3' });
    await wrapper.find('[data-testid="regelrijm-follow-$inkomen"]').trigger('click');
    await flushPromises();
    expect(wrapper.emitted('navigate')[1][0]).toEqual({ lawId: 'wet_b', articleNumber: '8' });
  });
});
