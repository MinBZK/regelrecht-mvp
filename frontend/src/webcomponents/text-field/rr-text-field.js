/**
 * RegelRecht Text Field Component
 *
 * A styled text input with border.
 *
 * @element rr-text-field
 * @attr {string} value - Input value
 * @attr {string} placeholder - Placeholder text
 * @attr {boolean} disabled - Disabled state
 * @attr {string} type - Input type (default: text)
 *
 * @fires input - When input value changes (on each keystroke)
 * @fires change - When input value changes (on blur)
 */

import { RRLocalBase } from '../base/rr-local-base.js';

export class RRTextField extends RRLocalBase {
  static componentName = 'rr-text-field';

  static get observedAttributes() {
    return ['value', 'placeholder', 'disabled', 'type'];
  }

  constructor() {
    super();
    this._onInput = this._onInput.bind(this);
    this._onChange = this._onChange.bind(this);
  }

  connectedCallback() {
    super.connectedCallback();
    requestAnimationFrame(() => {
      this._attachListeners();
    });
  }

  disconnectedCallback() {
    this._detachListeners();
  }

  _attachListeners() {
    const input = this.shadowRoot.querySelector('input');
    if (input) {
      input.addEventListener('input', this._onInput);
      input.addEventListener('change', this._onChange);
    }
  }

  _detachListeners() {
    const input = this.shadowRoot.querySelector('input');
    if (input) {
      input.removeEventListener('input', this._onInput);
      input.removeEventListener('change', this._onChange);
    }
  }

  _onInput(event) {
    const newValue = event.target.value;
    this.dispatchEvent(new CustomEvent('input', {
      detail: { value: newValue },
      bubbles: true,
      composed: true
    }));
  }

  _onChange(event) {
    const newValue = event.target.value;
    this.setAttribute('value', newValue);
    this.dispatchEvent(new CustomEvent('change', {
      detail: { value: newValue },
      bubbles: true,
      composed: true
    }));
  }

  get value() {
    return this.getAttribute('value') || '';
  }

  set value(val) {
    this.setAttribute('value', val);
    const input = this.shadowRoot?.querySelector('input');
    if (input) {
      input.value = val;
    }
  }

  get placeholder() {
    return this.getAttribute('placeholder') || '';
  }

  set placeholder(value) {
    this.setAttribute('placeholder', value);
  }

  get disabled() {
    return this.getBooleanAttribute('disabled');
  }

  set disabled(value) {
    if (value) {
      this.setAttribute('disabled', '');
    } else {
      this.removeAttribute('disabled');
    }
  }

  get type() {
    return this.getAttribute('type') || 'text';
  }

  set type(value) {
    this.setAttribute('type', value);
  }

  _getStyles() {
    return `
      :host {
        display: block;
        font-family: var(--rr-font-family-sans, 'RijksSansVF', system-ui, sans-serif);
      }

      :host([hidden]) {
        display: none;
      }

      .input {
        width: 100%;
        padding: var(--spacing-2, 8px) var(--spacing-3, 12px);
        border: 2px solid var(--color-slate-600, #475569);
        border-radius: var(--border-radius-md, 7px);
        font-size: var(--font-size-sm, 0.875rem);
        font-family: inherit;
        background: var(--color-white, #fff);
        color: var(--color-slate-900, #0f172a);
        box-sizing: border-box;
      }

      .input:focus {
        outline: 2px solid var(--color-primary, #154273);
        outline-offset: -2px;
      }

      .input:disabled {
        opacity: 0.6;
        cursor: not-allowed;
        background: var(--color-slate-100, #f1f5f9);
      }

      .input::placeholder {
        color: var(--color-slate-400, #94a3b8);
      }
    `;
  }

  render() {
    const value = this.escapeHtml(this.value);
    const placeholder = this.escapeHtml(this.placeholder);
    const disabled = this.disabled;
    const type = this.type;

    this.shadowRoot.innerHTML = `
      <input
        class="input"
        type="${type}"
        value="${value}"
        placeholder="${placeholder}"
        ${disabled ? 'disabled' : ''}
      >
    `;

    // Re-attach listeners after render
    this._attachListeners();
  }
}

// Register the element
customElements.define('rr-text-field', RRTextField);
