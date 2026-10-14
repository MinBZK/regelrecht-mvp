// The RegelRijm pane of the editor on two small fictional laws: every rule
// of the article as text, opening a fold in place, and following a name to
// the article or regelwerk that computes it.
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

async function setDepth(wrapper, d) {
  const control = wrapper.find('nldd-segmented-control');
  control.element.value = d;
  await control.trigger('change');
  await flushPromises();
}

const ruleText = (wrapper, output) => wrapper.find(`[data-testid="regelrijm-pane-rule-${output}"] [data-testid="regelrijm-text"]`).text();

describe('RegelRijmPane', () => {
  it('reads every rule of the article', async () => {
    const wrapper = await pane();
    expect(ruleText(wrapper, 'hoogte')).toContain('[hoogte] wordt:');
    expect(ruleText(wrapper, 'dubbel')).toBe('[dubbel] wordt: [hoogte] maal 2');
  });

  it('says so when the article has no rules', async () => {
    const wrapper = await pane({ article: { number: '9' } });
    expect(wrapper.text()).toBe('');
    expect(wrapper.find('nldd-banner').attributes('text')).toBe('Geen rekenregels');
  });

  it('opens a folded part in place, one level at a time', async () => {
    const wrapper = await pane();
    await setDepth(wrapper, '1');
    expect(ruleText(wrapper, 'hoogte')).toContain('⟨bedrag uit [inkomen], [korting] en [percentage]⟩');
    await wrapper.find('[data-testid="regelrijm-open-1.1"]').trigger('click');
    await flushPromises();
    expect(ruleText(wrapper, 'hoogte')).toContain('⟨bedrag uit [inkomen] en [korting]⟩ maal [percentage]');
  });

  it('emits open-action for the action sheet', async () => {
    const wrapper = await pane();
    await wrapper.find('[data-testid="regelrijm-pane-open-dubbel"]').trigger('click');
    expect(wrapper.emitted('open-action')[0][0]).toEqual(ARTIKEL.machine_readable.execution.actions[1]);
  });

  it('keeps opened folds through an edit, and folds again in another article', async () => {
    const wrapper = await pane();
    await setDepth(wrapper, '1');
    await wrapper.find('[data-testid="regelrijm-open-1.1"]').trigger('click');
    await flushPromises();
    // EditorView passes a new article object on every keystroke.
    await wrapper.setProps({ article: structuredClone(ARTIKEL) });
    await flushPromises();
    expect(ruleText(wrapper, 'hoogte')).toContain('maal [percentage]');
    await wrapper.setProps({ article: { ...structuredClone(ARTIKEL), number: '4' } });
    await flushPromises();
    expect(ruleText(wrapper, 'hoogte')).toContain('⟨bedrag uit [inkomen], [korting] en [percentage]⟩');
  });

  it('reads a half-edited article rule by rule instead of failing as a whole', async () => {
    const article = structuredClone(ARTIKEL);
    article.machine_readable.execution.actions = [null, { output: 'k', value: { operation: 'IF', cases: 'foo', default: 1 } }, ...article.machine_readable.execution.actions];
    article.machine_readable.execution.parameters = {};
    const wrapper = await pane({ article });
    expect(wrapper.find('[data-testid="regelrijm-pane-rule-0"]').text()).toContain('Actie zonder uitkomst');
    expect(ruleText(wrapper, 'k')).toBe('[k] wordt: als ⟨ontbreekt: cases⟩');
    expect(ruleText(wrapper, 'dubbel')).toBe('[dubbel] wordt: [hoogte] maal 2');
  });

  it('stays in the pane for a rule of the same article, also when the route names the law differently', async () => {
    const wrapper = await pane({ lawId: 'wet-a-in-de-route' });
    await wrapper.find('[data-testid="regelrijm-follow-$hoogte"]').trigger('click');
    await flushPromises();
    expect(wrapper.emitted('navigate')).toBeUndefined();
    await setDepth(wrapper, 'alles');
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
    await setDepth(wrapper, 'alles');
    await wrapper.find('[data-testid="regelrijm-follow-$korting"]').trigger('click');
    await flushPromises();
    expect(wrapper.emitted('navigate')[0][0]).toEqual({ lawId: 'wet_a', articleNumber: '3' });
    await wrapper.find('[data-testid="regelrijm-follow-$inkomen"]').trigger('click');
    await flushPromises();
    expect(wrapper.emitted('navigate')[1][0]).toEqual({ lawId: 'wet_b', articleNumber: '8' });
  });
});
