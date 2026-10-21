import { mount } from '@vue/test-utils';
import { describe, it, expect } from 'vitest';
import ScenarioForm from './ScenarioForm.vue';
import DataSourceTable from './DataSourceTable.vue';

const baseSetup = () => ({
  calculationDate: '2025-01-01',
  parameters: [],
  dataSources: [
    { sourceName: 'insurance', keyField: 'bsn', headers: ['bsn', 'verdragsinschrijving', 'onbekend'], rows: [['1', 'false', 'x']] },
  ],
});

type FieldMetaMap = Map<string, { type: string; unit: string | null; nullable?: boolean }>;

const mountForm = (externalFieldTypeMap: FieldMetaMap) =>
  mount(ScenarioForm, {
    props: { scenario: { assertions: [] }, setup: baseSetup(), lawId: 'l', externalFieldTypeMap },
  });

// The editable DataSourceTable only renders once a source is drilled into.
type FormWrapper = ReturnType<typeof mountForm>;
const drillIn = (w: FormWrapper) => w.find('[data-testid="ds-row-0"]').trigger('click');
const fieldsOf = (w: FormWrapper) => w.findComponent(DataSourceTable).props('fields');
// The column named `name`; fails the test when there is none.
const fieldNamed = (w: FormWrapper, name: string) => {
  const field = fieldsOf(w).find((c) => c.name === name);
  if (!field) throw new Error(`no column ${name}`);
  return field;
};

describe('ScenarioForm data-source column typing', () => {
  it('types data-source columns from the map (fallback string, key excluded)', async () => {
    const w = mountForm(new Map([['verdragsinschrijving', { type: 'boolean', unit: null }]]));
    await drillIn(w);
    expect(fieldNamed(w, 'verdragsinschrijving').type).toBe('boolean');
    expect(fieldNamed(w, 'onbekend').type).toBe('string'); // not in map -> fallback
    expect(fieldsOf(w).some((c) => c.name === 'bsn')).toBe(false); // key field excluded
  });

  it('re-types columns in place when the map arrives asynchronously', async () => {
    const w = mountForm(new Map()); // empty at mount -> all string
    await drillIn(w);
    expect(fieldNamed(w, 'verdragsinschrijving').type).toBe('string');
    await w.setProps({ externalFieldTypeMap: new Map([['verdragsinschrijving', { type: 'boolean', unit: null }]]) });
    expect(fieldNamed(w, 'verdragsinschrijving').type).toBe('boolean');
  });
});
