import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import TrajectCreateForm from './TrajectCreateForm.vue';

const apiFetchJson = vi.fn();
vi.mock('../lib/apiFetch.js', () => ({
  apiFetchJson: (...a) => apiFetchJson(...a),
}));

async function mountForm(localTargets) {
  apiFetchJson.mockResolvedValue(localTargets);
  const wrapper = mount(TrajectCreateForm);
  await flushPromises();
  return wrapper;
}

function setName(wrapper, name) {
  const field = wrapper.find('nldd-text-field');
  field.element.value = name;
  field.trigger('input');
}

beforeEach(() => apiFetchJson.mockReset());

describe('TrajectCreateForm', () => {
  it('asks the backend for local targets', async () => {
    await mountForm([]);
    expect(apiFetchJson).toHaveBeenCalledWith('/api/trajects/local-targets');
  });

  it('defaults to the first local target and sends only its id', async () => {
    const wrapper = await mountForm([{ id: 'lokaal', name: 'Lokale checkout' }]);
    setName(wrapper, 'Lokaal werk');

    const { payload } = wrapper.vm.buildPayload();
    expect(payload).toEqual({
      name: 'Lokaal werk',
      description: '',
      local_source_id: 'lokaal',
    });
    expect(wrapper.find('nldd-radio-button-group').exists()).toBe(true);
    // The GitHub switch belongs to the GitHub choice.
    expect(wrapper.find('nldd-switch-field').exists()).toBe(false);
  });

  it('sends no local target once GitHub is chosen', async () => {
    const wrapper = await mountForm([{ id: 'lokaal', name: 'Lokale checkout' }]);
    setName(wrapper, 'Op GitHub');
    await wrapper
      .find('nldd-radio-button-group')
      .trigger('change', { detail: { checked: true, value: 'github' } });

    const { payload } = wrapper.vm.buildPayload();
    expect(payload).toEqual({ name: 'Op GitHub', description: '' });
    expect(wrapper.find('nldd-switch-field').exists()).toBe(true);
  });

  it('looks as before without local targets', async () => {
    const wrapper = await mountForm([]);
    setName(wrapper, 'Gewoon');

    expect(wrapper.find('nldd-radio-button-group').exists()).toBe(false);
    expect(wrapper.find('nldd-switch-field').exists()).toBe(true);
    expect(wrapper.vm.buildPayload().payload).toEqual({ name: 'Gewoon', description: '' });
  });
});
