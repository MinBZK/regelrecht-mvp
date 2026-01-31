/**
 * RegelRecht Help Text Component
 *
 * Summary text with optional action link.
 *
 * @element rr-help-text
 * @attr {string} summary - Summary text to display
 * @attr {string} action-label - Action link text (default: "Bewerk")
 * @attr {boolean} show-action - Whether to show the action link
 *
 * @fires action-click - When action link is clicked
 */

import { RRLocalBase } from '../base/rr-local-base.js';

export class RRHelpText extends RRLocalBase {
  static componentName = 'rr-help-text';

  static get observedAttributes() {
    return ['summary', 'action-label', 'show-action'];
  }

  constructor() {
    super();
    this._onActionClick = this._onActionClick.bind(this);
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
    const link = this.shadowRoot.querySelector('.help-text__action');
    if (link) {
      link.addEventListener('click', this._onActionClick);
    }
  }

  _detachListeners() {
    const link = this.shadowRoot.querySelector('.help-text__action');
    if (link) {
      link.removeEventListener('click', this._onActionClick);
    }
  }

  _onActionClick(event) {
    event.preventDefault();
    this.dispatchEvent(new CustomEvent('action-click', {
      bubbles: true,
      composed: true
    }));
  }

  get summary() {
    return this.getAttribute('summary') || '';
  }

  set summary(value) {
    this.setAttribute('summary', value);
  }

  get actionLabel() {
    return this.getAttribute('action-label') || 'Bewerk';
  }

  set actionLabel(value) {
    this.setAttribute('action-label', value);
  }

  get showAction() {
    return this.getBooleanAttribute('show-action');
  }

  set showAction(value) {
    if (value) {
      this.setAttribute('show-action', '');
    } else {
      this.removeAttribute('show-action');
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

      .help-text {
        display: flex;
        align-items: center;
        gap: var(--spacing-2, 8px);
        font-size: var(--font-size-sm, 0.875rem);
        color: var(--color-slate-800, #1e293b);
        line-height: 1.25;
      }

      .help-text__summary {
        flex: 1;
      }

      .help-text__action {
        color: var(--color-primary, #154273);
        text-decoration: none;
        font-weight: var(--font-weight-medium, 500);
        cursor: pointer;
        flex-shrink: 0;
      }

      .help-text__action:hover {
        text-decoration: underline;
      }

      .help-text__action:focus-visible {
        outline: 2px solid var(--color-primary, #154273);
        outline-offset: 2px;
        border-radius: 2px;
      }
    `;
  }

  render() {
    const summary = this.escapeHtml(this.summary);
    const actionLabel = this.escapeHtml(this.actionLabel);
    const showAction = this.showAction;

    this.shadowRoot.innerHTML = `
      <div class="help-text">
        <span class="help-text__summary">${summary}</span>
        ${showAction ? `<a href="#" class="help-text__action">${actionLabel}</a>` : ''}
      </div>
    `;

    // Re-attach listeners after render
    if (showAction) {
      this._attachListeners();
    }
  }
}

// Register the element
customElements.define('rr-help-text', RRHelpText);
