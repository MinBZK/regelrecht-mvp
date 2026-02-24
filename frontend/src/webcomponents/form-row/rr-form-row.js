/**
 * RegelRecht Form Row Component
 *
 * A form row layout with label and control slot.
 *
 * @element rr-form-row
 * @attr {string} label - Label text displayed on the left
 * @attr {string} background - Background color (default: #f1f5f9)
 * @attr {boolean} divider - Show border-bottom divider
 *
 * @slot - Default slot for control element
 */

import { RRLocalBase } from '../base/rr-local-base.js';

export class RRFormRow extends RRLocalBase {
  static componentName = 'rr-form-row';

  static get observedAttributes() {
    return ['label', 'background', 'divider'];
  }

  get label() {
    return this.getAttribute('label') || '';
  }

  set label(value) {
    this.setAttribute('label', value);
  }

  get background() {
    return this.getAttribute('background') || '#f1f5f9';
  }

  set background(value) {
    this.setAttribute('background', value);
  }

  get divider() {
    return this.getBooleanAttribute('divider');
  }

  set divider(value) {
    if (value) {
      this.setAttribute('divider', '');
    } else {
      this.removeAttribute('divider');
    }
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

      .form-row {
        display: flex;
        align-items: center;
        padding: var(--spacing-3, 12px) var(--spacing-4, 16px);
        background-color: var(--form-row-background, #f1f5f9);
      }

      :host([divider]) .form-row {
        border-bottom: 1px solid #e2e8f0;
      }

      .form-row__label {
        flex: 0 0 80px;
        font-size: var(--font-size-sm, 0.875rem);
        color: var(--color-slate-700, #334155);
      }

      .form-row__control {
        flex: 1;
        min-width: 0;
      }
    `;
  }

  render() {
    const background = this.background;
    const label = this.escapeHtml(this.label);

    this.shadowRoot.innerHTML = `
      <div class="form-row" style="--form-row-background: ${background}">
        ${label ? `<label class="form-row__label">${label}</label>` : ''}
        <div class="form-row__control">
          <slot></slot>
        </div>
      </div>
    `;
  }
}

// Register the element
customElements.define('rr-form-row', RRFormRow);
