import { afterEach, describe, expect, it, vi } from 'vitest';
import { key } from './actions';

/** A host element standing in for a design-system sheet, with its `hide()`. */
function makeSheet(): HTMLElement & { hide: ReturnType<typeof vi.fn> } {
  const sheet = document.createElement('rr-test-sheet') as HTMLElement & { hide: ReturnType<typeof vi.fn> };
  sheet.hide = vi.fn();
  return sheet;
}

describe('a replayed Escape', () => {
  afterEach(() => {
    document.body.innerHTML = '';
  });

  it('closes the sheet it is pressed in, through the sheet itself', () => {
    const sheet = makeSheet();
    const dialog = document.createElement('dialog');
    dialog.setAttribute('open', '');
    sheet.attachShadow({ mode: 'open' }).appendChild(dialog);
    document.body.appendChild(sheet);
    key(dialog, 'Escape');
    expect(sheet.hide).toHaveBeenCalledOnce();
  });

  it('leaves it open when the page took the key, and for any other key', () => {
    const sheet = makeSheet();
    const dialog = document.createElement('dialog');
    dialog.setAttribute('open', '');
    sheet.attachShadow({ mode: 'open' }).appendChild(dialog);
    document.body.appendChild(sheet);
    key(dialog, 'Enter');
    dialog.addEventListener('keydown', (e) => e.preventDefault());
    key(dialog, 'Escape');
    expect(sheet.hide).not.toHaveBeenCalled();
  });
});
