/**
 * Doing what the presenter did, on the element `locator.js` found again.
 *
 * Synthetic events, dispatched where a real one would land: on the innermost
 * element, bubbling and composed so they cross the shadow boundaries the way
 * a real click does. The demo's handlers (Vue's @click on a design-system
 * host, the components' own listeners inside) cannot tell the difference,
 * except through `isTrusted`, which the player uses the other way round: a
 * trusted event during playback is the viewer, not the replay.
 */

function point(el, fx = 0.5, fy = 0.5) {
  const r = el.getBoundingClientRect();
  return { clientX: r.left + r.width * fx, clientY: r.top + r.height * fy };
}

/** The innermost element a click at the centre of `el` would hit, inside its shadow roots. */
export function innermost(el, fx = 0.5, fy = 0.5) {
  const { clientX, clientY } = point(el, fx, fy);
  let cur = el;
  for (let i = 0; i < 8 && cur?.shadowRoot; i += 1) {
    const hit = cur.shadowRoot.elementFromPoint?.(clientX, clientY);
    if (!hit || hit === cur || !cur.contains(hit) && !cur.shadowRoot.contains(hit)) break;
    cur = hit;
  }
  return cur;
}

export function click(el, { fx = 0.5, fy = 0.5 } = {}) {
  const target = innermost(el, fx, fy);
  const at = point(el, fx, fy);
  const base = { bubbles: true, composed: true, cancelable: true, view: window, button: 0, ...at };
  target.dispatchEvent(new PointerEvent('pointerdown', { ...base, pointerId: 1, isPrimary: true, buttons: 1 }));
  target.dispatchEvent(new MouseEvent('mousedown', { ...base, buttons: 1 }));
  target.focus?.({ preventScroll: true });
  target.dispatchEvent(new PointerEvent('pointerup', { ...base, pointerId: 1, isPrimary: true }));
  target.dispatchEvent(new MouseEvent('mouseup', base));
  target.dispatchEvent(new MouseEvent('click', base));
}

/** The real text field behind a design-system field: its <input> or <textarea>. */
export function fieldOf(el) {
  if (/^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName)) return el;
  return el.shadowRoot?.querySelector('input, textarea, select') ?? el.querySelector?.('input, textarea, select') ?? el;
}

/**
 * Put `value` in a field the way typing does. Through the prototype's setter,
 * because a framework may shadow `value` on the instance; then an `input`
 * event, which is what both Vue and the design-system fields listen to.
 */
export function setValue(el, value, { change = false } = {}) {
  const field = fieldOf(el);
  const proto = field instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : field instanceof HTMLSelectElement ? HTMLSelectElement.prototype : HTMLInputElement.prototype;
  const setter = Object.getOwnPropertyDescriptor(proto, 'value')?.set;
  if (setter) setter.call(field, value);
  else field.value = value;
  field.dispatchEvent(new Event('input', { bubbles: true, composed: true }));
  if (change) field.dispatchEvent(new Event('change', { bubbles: true, composed: true }));
}

export function setChecked(el, checked) {
  const field = fieldOf(el);
  if (field.checked !== checked) click(el);
}

export function key(el, keyName) {
  const field = fieldOf(el);
  const init = { key: keyName, bubbles: true, composed: true, cancelable: true };
  const taken = !field.dispatchEvent(new KeyboardEvent('keydown', init));
  field.dispatchEvent(new KeyboardEvent('keyup', init));
  // Escape closes the sheet, dialog or popover it is pressed in. That is the
  // browser's doing, and a made-up key event does not set it off.
  if (keyName === 'Escape' && !taken) closeAround(el);
}

/** Close the innermost open sheet, dialog or popover around `el`, as Escape does. */
function closeAround(el) {
  for (let n = el; n; n = n.parentElement ?? n.getRootNode?.()?.host) {
    if (n.tagName === 'DIALOG' && n.open) {
      // A design-system sheet closes through its own method, so it knows.
      const host = n.getRootNode?.()?.host;
      if (typeof host?.hide === 'function') host.hide();
      else if (n.dispatchEvent(new Event('cancel', { cancelable: true }))) n.close();
      return;
    }
    if (n.matches?.('[popover]:popover-open')) {
      n.hidePopover();
      return;
    }
  }
}

export function scrollTo(el, top, left = 0) {
  el.scrollTo?.({ top, left, behavior: 'instant' });
}
