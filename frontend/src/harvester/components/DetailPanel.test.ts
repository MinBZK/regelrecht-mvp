import { mount } from '@vue/test-utils';
import { describe, it, expect } from 'vitest';
import DetailPanel from './DetailPanel.vue';

// The sheet teleports to body; render it in place so the rows are in w.html().
const global = {
  config: { compilerOptions: { isCustomElement: (tag: string) => tag.startsWith('nldd-') } },
  stubs: { Teleport: true },
};

describe('DetailPanel', () => {
  it('shows attempts as "n / max" when the job has them', () => {
    const w = mount(DetailPanel, {
      props: { job: { id: 'j1', status: 'failed', attempts: 2, max_attempts: 3 }, isOpen: true },
      global,
    });
    expect(w.html()).toContain('2 / 3');
  });

  // OverviewView opens the panel with a recent-failure row when fetching the
  // full job fails; that row has no attempts. A missing field is left out,
  // like every other row, instead of reading "undefined / undefined".
  it('leaves out the attempts row when the job has no attempts', () => {
    const w = mount(DetailPanel, {
      props: { job: { id: 'j1', status: 'failed', law_id: 'wet_a' }, isOpen: true },
      global,
    });
    expect(w.html()).toContain('wet_a');
    expect(w.html()).not.toContain('Attempts');
    expect(w.html()).not.toContain('undefined');
  });
});
