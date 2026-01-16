/**
 * RegelRecht Select Field Component
 *
 * A styled dropdown select with custom chevron icon.
 *
 * @element rr-select-field
 * @attr {string} value - Currently selected value
 * @attr {boolean} disabled - Disabled state
 * @attr {string} background - Background color (default: #e2e8f0)
 *
 * @slot - Default slot for option elements
 *
 * @fires change - When selection changes
 */

import { RRLocalBase } from '../base/rr-local-base.js';

export class RRSelectField extends RRLocalBase {
  static componentName = 'rr-select-field';

  static get observedAttributes() {
    return ['value', 'disabled', 'background'];
  }

  constructor() {
    super();
    this._onChange = this._onChange.bind(this);
  }

  connectedCallback() {
    super.connectedCallback();
    // Defer event listener setup to after render
    requestAnimationFrame(() => {
      this._setupSelect();
      this._syncOptions();
    });

    // Observe changes to light DOM children (options)
    this._mutationObserver = new MutationObserver(() => this._syncOptions());
    this._mutationObserver.observe(this, { childList: true, subtree: true, characterData: true });
  }

  disconnectedCallback() {
    const select = this.shadowRoot.querySelector('select');
    if (select) {
      select.removeEventListener('change', this._onChange);
    }
    if (this._mutationObserver) {
      this._mutationObserver.disconnect();
    }
  }

  _setupSelect() {
    const select = this.shadowRoot.querySelector('select');
    if (select) {
      select.addEventListener('change', this._onChange);
    }
  }

  _syncOptions() {
    const select = this.shadowRoot.querySelector('select');
    if (!select) return;

    // Get options from light DOM
    const lightOptions = this.querySelectorAll('option');

    // Clear and rebuild options in shadow select
    select.innerHTML = '';
    lightOptions.forEach(opt => {
      const newOpt = document.createElement('option');
      newOpt.value = opt.value;
      newOpt.textContent = opt.textContent;
      newOpt.selected = opt.selected || opt.hasAttribute('selected');
      newOpt.disabled = opt.disabled;
      select.appendChild(newOpt);
    });

    // Set value if specified
    const value = this.value;
    if (value) {
      select.value = value;
    }
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

  get background() {
    return this.getAttribute('background') || '#e2e8f0';
  }

  set background(value) {
    this.setAttribute('background', value);
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

      .select-wrapper {
        position: relative;
        width: 100%;
      }

      .select {
        width: 100%;
        height: 44px;
        padding: 8px 40px 8px 12px;
        border: none;
        border-radius: 7px;
        font-size: 1rem;
        font-family: inherit;
        background-color: var(--select-background, #e2e8f0);
        color: #0f172a;
        cursor: pointer;
        appearance: none;
        -webkit-appearance: none;
        -moz-appearance: none;
      }

      .select:focus {
        outline: 2px solid var(--color-primary, #154273);
        outline-offset: -2px;
      }

      .select:disabled {
        opacity: 0.6;
        cursor: not-allowed;
      }

      .chevron {
        position: absolute;
        right: 12px;
        top: 50%;
        transform: translateY(-50%);
        pointer-events: none;
        width: 16px;
        height: 16px;
      }

      .chevron svg {
        width: 16px;
        height: 16px;
      }
    `;
  }

  render() {
    const background = this.background;
    const disabled = this.disabled;

    this.shadowRoot.innerHTML = `
      <div class="select-wrapper">
        <select
          class="select"
          style="--select-background: ${background}"
          ${disabled ? 'disabled' : ''}
        >
        </select>
        <span class="chevron">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M4 6L8 10L12 6" stroke="#334155" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </span>
      </div>
      <slot style="display: none;"></slot>
    `;

    // Setup event listener and sync options
    this._setupSelect();
    this._syncOptions();
  }
}

// Register the element
customElements.define('rr-select-field', RRSelectField);
