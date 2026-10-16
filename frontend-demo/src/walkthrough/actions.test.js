import { afterEach, describe, expect, it, vi } from 'vitest';
import { key } from './actions.js';

describe('a replayed Escape', () => {
  afterEach(() => {
    document.body.innerHTML = '';
  });

  it('closes the sheet it is pressed in, through the sheet itself', () => {
    const sheet = document.createElement('rr-test-sheet');
    sheet.hide = vi.fn();
    const dialog = document.createElement('dialog');
    dialog.setAttribute('open', '');
    sheet.attachShadow({ mode: 'open' }).appendChild(dialog);
    document.body.appendChild(sheet);
    key(dialog, 'Escape');
    expect(sheet.hide).toHaveBeenCalledOnce();
  });

  it('leaves it open when the page took the key, and for any other key', () => {
    const sheet = document.createElement('rr-test-sheet');
    sheet.hide = vi.fn();
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
