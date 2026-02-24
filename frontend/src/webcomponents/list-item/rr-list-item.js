/**
 * RegelRecht List Item Component
 *
 * A clickable list item with title, summary, and optional chevron.
 *
 * @element rr-list-item
 * @attr {string} title - Item title
 * @attr {string} summary - Item summary text
 * @attr {boolean} clickable - Whether item is clickable (shows chevron)
 *
 * @fires click - When clickable item is clicked
 */

import { RRLocalBase } from '../base/rr-local-base.js';

export class RRListItem extends RRLocalBase {
  static componentName = 'rr-list-item';

  static get observedAttributes() {
    return ['title', 'summary', 'clickable'];
  }

  constructor() {
    super();
    this._onClick = this._onClick.bind(this);
  }

  connectedCallback() {
    super.connectedCallback();
    if (this.clickable) {
      this.addEventListener('click', this._onClick);
      this.setAttribute('role', 'button');
      this.setAttribute('tabindex', '0');
    }
  }

  disconnectedCallback() {
    this.removeEventListener('click', this._onClick);
  }

  _onClick(event) {
    if (this.clickable) {
      this.dispatchEvent(new CustomEvent('item-click', {
        bubbles: true,
        composed: true
      }));
    }
  }

  get title() {
    return this.getAttribute('title') || '';
  }

  set title(value) {
    this.setAttribute('title', value);
  }

  get summary() {
    return this.getAttribute('summary') || '';
  }

  set summary(value) {
    this.setAttribute('summary', value);
  }

  get clickable() {
    return this.getBooleanAttribute('clickable');
  }

  set clickable(value) {
    if (value) {
      this.setAttribute('clickable', '');
      this.setAttribute('role', 'button');
      this.setAttribute('tabindex', '0');
      this.addEventListener('click', this._onClick);
    } else {
      this.removeAttribute('clickable');
      this.removeAttribute('role');
      this.removeAttribute('tabindex');
      this.removeEventListener('click', this._onClick);
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

      :host([clickable]) {
        cursor: pointer;
      }

      :host([clickable]:hover) .list-item {
        background-color: var(--color-slate-100, #f1f5f9);
      }

      :host([clickable]:focus-visible) {
        outline: 2px solid var(--color-primary, #154273);
        outline-offset: -2px;
        border-radius: var(--border-radius-md, 7px);
      }

      .list-item {
        display: flex;
        align-items: center;
        padding: var(--spacing-3, 12px) var(--spacing-4, 16px);
        gap: var(--spacing-3, 12px);
        transition: background-color 0.15s ease;
      }

      .list-item__content {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: var(--spacing-1, 4px);
      }

      .list-item__title {
        font-size: var(--font-size-base, 1rem);
        font-weight: var(--font-weight-medium, 500);
        color: var(--color-slate-900, #0f172a);
        margin: 0;
      }

      .list-item__summary {
        font-size: var(--font-size-sm, 0.875rem);
        color: var(--color-slate-600, #475569);
        margin: 0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }

      .list-item__chevron {
        flex-shrink: 0;
        width: 20px;
        height: 20px;
        color: var(--color-slate-400, #94a3b8);
      }

      .list-item__chevron svg {
        width: 20px;
        height: 20px;
      }
    `;
  }

  render() {
    const title = this.escapeHtml(this.title);
    const summary = this.escapeHtml(this.summary);
    const clickable = this.clickable;

    this.shadowRoot.innerHTML = `
      <div class="list-item">
        <div class="list-item__content">
          ${title ? `<h4 class="list-item__title">${title}</h4>` : ''}
          ${summary ? `<p class="list-item__summary">${summary}</p>` : ''}
        </div>
        ${clickable ? `
          <span class="list-item__chevron">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 20 20" fill="none">
              <path d="M7.5 5L12.5 10L7.5 15" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </span>
        ` : ''}
      </div>
    `;
  }
}

// Register the element
customElements.define('rr-list-item', RRListItem);
