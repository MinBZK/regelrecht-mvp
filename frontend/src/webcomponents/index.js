/**
 * Local RegelRecht Web Components
 *
 * These are local components for the machine action sheet UI,
 * complementing the @minbzk/storybook design system.
 *
 * Import this file to register all local web components:
 *
 *   import './src/webcomponents';
 *
 * Components:
 * - <rr-form-row>     - Form row layout with label
 * - <rr-select-field> - Styled dropdown select
 * - <rr-text-field>   - Styled text input
 * - <rr-help-text>    - Summary text with action link
 * - <rr-list-item>    - Clickable list item
 */

// Base class (not registered, just exported for extension)
export { RRLocalBase } from './base/rr-local-base.js';

// Components (self-registering on import)
export { RRFormRow } from './form-row/rr-form-row.js';
export { RRSelectField } from './select-field/rr-select-field.js';
export { RRTextField } from './text-field/rr-text-field.js';
export { RRHelpText } from './help-text/rr-help-text.js';
export { RRListItem } from './list-item/rr-list-item.js';
