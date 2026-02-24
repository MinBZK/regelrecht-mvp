/**
 * Text Annotator - Simpele, interactieve annotatie tool
 *
 * Features:
 * - Highlight variabelen, inputs, outputs in wettekst
 * - Link logica (actions) aan tekstfragmenten
 * - Interactief toevoegen van open normen (opslaan naar YAML)
 */

const TextAnnotator = {
  // Kleuren
  colors: {
    definition: { bg: 'rgba(253, 224, 71, 0.35)', border: '#eab308' },
    input: { bg: 'rgba(147, 197, 253, 0.35)', border: '#3b82f6' },
    output: { bg: 'rgba(134, 239, 172, 0.35)', border: '#22c55e' },
    openNorm: { bg: 'rgba(249, 115, 22, 0.15)', border: '#f97316' },
    logic: { bg: 'rgba(168, 85, 247, 0.2)', border: '#a855f7' },
    lawReference: { bg: 'rgba(14, 165, 233, 0.2)', border: '#0ea5e9' }
  },

  // W3C Motivation vocabulary
  motivations: {
    commenting: { label: 'Commenting', description: 'Open norm / uitleg' },
    linking: { label: 'Linking', description: 'Referentie naar andere wet' },
    tagging: { label: 'Tagging', description: 'Machine-readable label' },
    classifying: { label: 'Classifying', description: 'Definition/Input/Output classificatie' }
  },

  // Classification types
  classifications: ['definition', 'input', 'output', 'logic', 'open_norm'],

  // Data types
  dataTypes: ['boolean', 'integer', 'number', 'string', 'date', 'money', 'amount'],

  // BWB resolver cache (populated from server)
  bwbMappings: {},

  /**
   * Initialize BWB mappings from server
   */
  async initBwbMappings() {
    try {
      const res = await fetch('/api/bwb');
      if (res.ok) {
        this.bwbMappings = await res.json();
      }
    } catch (e) {
      console.warn('Failed to load BWB mappings:', e);
    }
  },

  /**
   * Get law_id from BWB ID
   */
  bwbToLawId(bwbId) {
    const mapping = this.bwbMappings[bwbId];
    return mapping ? mapping.law_id : null;
  },

  /**
   * Convert wetten.overheid.nl URL to regelrecht:// URI
   */
  urlToRegelrechtUri(url) {
    const match = url.match(/wetten\.overheid\.nl\/(BWBR[0-9]{7})/);
    if (match) {
      const lawId = this.bwbToLawId(match[1]);
      if (lawId) {
        return `regelrecht://${lawId}`;
      }
    }
    return null;
  },

  /**
   * Get existing tags from article (from YAML and annotations)
   * @param {Object} article - Article data with definitions, inputs, outputs, annotations
   * @returns {Array} Array of existing tag objects
   */
  getExistingTags(article) {
    const tags = [];

    // From machine_readable definitions
    if (article.definitions) {
      Object.keys(article.definitions).forEach(name => {
        tags.push({
          name,
          type: 'definition',
          source: 'yaml',
          value: article.definitions[name].value,
          description: article.definitions[name].description
        });
      });
    }

    // From execution.input
    if (article.input) {
      article.input.forEach(inp => {
        tags.push({
          name: inp.name,
          type: 'input',
          source: 'yaml',
          dataType: inp.type,
          regulation: inp.source?.regulation
        });
      });
    }

    // From execution.output
    if (article.output) {
      article.output.forEach(out => {
        tags.push({
          name: out.name,
          type: 'output',
          source: 'yaml',
          dataType: out.type
        });
      });
    }

    // From execution.parameters
    if (article.parameters) {
      article.parameters.forEach(param => {
        tags.push({
          name: param.name,
          type: 'parameter',
          source: 'yaml',
          dataType: param.type,
          required: param.required
        });
      });
    }

    // From open_norms
    const machineReadable = article.machine_readable || {};
    if (machineReadable.open_norms) {
      machineReadable.open_norms.forEach(norm => {
        tags.push({
          name: norm.term,
          type: 'open_norm',
          source: 'yaml',
          description: norm.description
        });
      });
    }

    // From W3C annotations (not yet promoted)
    if (article.annotations) {
      article.annotations.forEach(ann => {
        if (ann.status !== 'promoted') {
          const name = ann.body?.name || ann.target?.selector?.exact;
          if (name) {
            tags.push({
              name: name,
              type: ann.body?.classification || 'open_norm',
              source: 'annotation',
              status: ann.status,
              dataType: ann.body?.data_type,
              description: ann.body?.description || ann.body?.value
            });
          }
        }
      });
    }

    return tags;
  },

  /**
   * Convert text to a valid variable name
   * @param {string} text - Text to convert
   * @returns {string} Valid variable name
   */
  _textToVariableName(text) {
    if (!text) return '';
    return text.toLowerCase()
      .replace(/\s+/g, '_')
      .replace(/[^a-z0-9_]/g, '')
      .replace(/^[0-9_]+/, '')
      .replace(/_+/g, '_')
      .substring(0, 50);
  },

  /**
   * Genereer TextQuoteSelector van selectie
   * @param {string} text - Volledige tekst
   * @param {string} selectedText - Geselecteerde tekst
   * @param {number} startOffset - Start positie in tekst
   * @returns {Object} TextQuoteSelector object
   */
  createSelector(text, selectedText, startOffset) {
    const prefixStart = Math.max(0, startOffset - 30);
    const suffixEnd = Math.min(text.length, startOffset + selectedText.length + 30);
    return {
      type: 'TextQuoteSelector',
      exact: selectedText,
      prefix: text.slice(prefixStart, startOffset),
      suffix: text.slice(startOffset + selectedText.length, suffixEnd)
    };
  },

  /**
   * Resolve TextQuoteSelector naar match positie
   * @param {string} text - Tekst om in te zoeken
   * @param {Object} selector - TextQuoteSelector object
   * @returns {Object|null} { start, end } of null als niet gevonden
   */
  resolveSelector(text, selector) {
    if (!selector || !selector.exact) return null;

    // Probeer exact match met prefix + suffix context
    if (selector.prefix && selector.suffix) {
      const fullPattern = selector.prefix + selector.exact + selector.suffix;
      const idx = text.indexOf(fullPattern);
      if (idx !== -1) {
        return {
          start: idx + selector.prefix.length,
          end: idx + selector.prefix.length + selector.exact.length
        };
      }
    }

    // Probeer exact match met alleen prefix context
    if (selector.prefix) {
      const patternWithPrefix = selector.prefix + selector.exact;
      const idx = text.indexOf(patternWithPrefix);
      if (idx !== -1) {
        return {
          start: idx + selector.prefix.length,
          end: idx + selector.prefix.length + selector.exact.length
        };
      }
    }

    // Probeer exact match met alleen suffix context
    if (selector.suffix) {
      const patternWithSuffix = selector.exact + selector.suffix;
      const idx = text.indexOf(patternWithSuffix);
      if (idx !== -1) {
        return {
          start: idx,
          end: idx + selector.exact.length
        };
      }
    }

    // Fallback: zoek alleen de exact tekst
    const idx = text.indexOf(selector.exact);
    if (idx !== -1) {
      return {
        start: idx,
        end: idx + selector.exact.length
      };
    }

    // Fuzzy fallback: case-insensitive zoeken
    const lowerText = text.toLowerCase();
    const lowerExact = selector.exact.toLowerCase();
    const fuzzyIdx = lowerText.indexOf(lowerExact);
    if (fuzzyIdx !== -1) {
      return {
        start: fuzzyIdx,
        end: fuzzyIdx + selector.exact.length
      };
    }

    return null;
  },

  /**
   * Extract law references uit markdown tekst
   * @param {string} text - Tekst met mogelijke markdown links
   * @returns {Array} Array van law reference objecten
   */
  extractLawReferences(text) {
    const results = [];
    // Markdown links naar wetten.overheid.nl
    const pattern = /\[([^\]]+)\]\((https:\/\/wetten\.overheid\.nl\/([A-Z0-9]+)[^)]*)\)/g;
    let match;
    while ((match = pattern.exec(text)) !== null) {
      results.push({
        name: match[1],
        url: match[2],
        bwb_id: match[3],
        fullMatch: match[0],
        start: match.index,
        end: match.index + match[0].length
      });
    }
    return results;
  },

  /**
   * Zoek alle annoteerbare termen in de tekst
   */
  findAnnotations(text, article) {
    if (!text || !article) return [];

    const annotations = [];
    const textLower = text.toLowerCase();

    // 1. Definities uit machine_readable.definitions
    if (article.definitions) {
      Object.entries(article.definitions).forEach(([name, def]) => {
        const searchTerms = this.termToSearchWords(name);
        searchTerms.forEach(term => {
          this.findTermInText(textLower, text, term, {
            type: 'definition',
            name: name,
            value: def.value !== undefined ? def.value : def,
            description: def.description
          }, annotations);
        });
      });
    }

    // 2. Inputs
    if (article.input) {
      article.input.forEach(inp => {
        const searchTerms = this.termToSearchWords(inp.name);
        searchTerms.forEach(term => {
          this.findTermInText(textLower, text, term, {
            type: 'input',
            name: inp.name,
            dataType: inp.type,
            source: inp.source,
            description: inp.description
          }, annotations);
        });
      });
    }

    // 3. Outputs
    if (article.output) {
      article.output.forEach(out => {
        const searchTerms = this.termToSearchWords(out.name);
        searchTerms.forEach(term => {
          this.findTermInText(textLower, text, term, {
            type: 'output',
            name: out.name,
            dataType: out.type,
            description: out.description
          }, annotations);
        });
      });
    }

    // 4. Open normen uit YAML
    const machineReadable = article.machine_readable || {};
    if (machineReadable.open_norms) {
      machineReadable.open_norms.forEach(norm => {
        const searchTerms = this.termToSearchWords(norm.term);
        searchTerms.forEach(term => {
          this.findTermInText(textLower, text, term, {
            type: 'openNorm',
            name: norm.term,
            description: norm.description,
            fromYaml: true
          }, annotations);
        });
      });
    }

    // 5. Logica - zoek variabelen die in actions gebruikt worden
    if (article.actions) {
      article.actions.forEach(action => {
        const vars = this.extractVariablesFromAction(action);
        vars.forEach(varName => {
          const searchTerms = this.termToSearchWords(varName);
          searchTerms.forEach(term => {
            this.findTermInText(textLower, text, term, {
              type: 'logic',
              name: varName,
              action: action.output,
              description: `Gebruikt in logica voor: ${action.output}`
            }, annotations);
          });
        });
      });
    }

    // 6. Law references uit markdown links
    const lawRefs = this.extractLawReferences(text);
    lawRefs.forEach(ref => {
      annotations.push({
        start: ref.start,
        end: ref.end,
        text: ref.fullMatch,
        type: 'lawReference',
        name: ref.name,
        url: ref.url,
        bwb_id: ref.bwb_id,
        description: `Referentie naar ${ref.name}`
      });
    });

    // 7. W3C annotations from server
    if (article.annotations) {
      article.annotations.forEach(ann => {
        const selector = ann.target?.selector;
        const resolved = this.resolveSelector(text, selector);
        if (resolved) {
          const classification = ann.body?.classification || 'unknown';
          annotations.push({
            start: resolved.start,
            end: resolved.end,
            text: text.substring(resolved.start, resolved.end),
            type: classification === 'open_norm' ? 'openNorm' : classification,
            name: ann.body?.name || selector?.exact || 'annotatie',
            description: ann.body?.description || ann.body?.value,
            fromAnnotation: true,
            status: ann.status
          });
        }
      });
    }

    // Sorteer en verwijder duplicaten
    return this.deduplicateAnnotations(annotations);
  },

  /**
   * Converteer variabele naam naar zoektermen
   * bijv: "is_gezamenlijke_huishouding" -> ["gezamenlijke huishouding"]
   */
  termToSearchWords(name) {
    if (!name) return [];
    const terms = [];

    // Vervang underscores door spaties
    let spaced = name.replace(/_/g, ' ').toLowerCase();

    // Verwijder common prefixes
    const prefixes = ['is ', 'heeft ', 'wordt ', 'kan ', 'mag ', 'moet ', 'voldoet aan '];
    prefixes.forEach(prefix => {
      if (spaced.startsWith(prefix)) {
        terms.push(spaced.substring(prefix.length));
      }
    });

    // Voeg ook volledige term toe
    terms.push(spaced);

    // Voeg varianten zonder lidwoorden toe
    const withoutArticles = spaced.replace(/\b(de|het|een|van|in|op|aan|voor|met)\b/g, '').replace(/\s+/g, ' ').trim();
    if (withoutArticles !== spaced && withoutArticles.length > 3) {
      terms.push(withoutArticles);
    }

    return [...new Set(terms)].filter(t => t.length > 2);
  },

  /**
   * Zoek een term in de tekst en voeg annotaties toe
   */
  findTermInText(textLower, text, term, data, annotations) {
    const termLower = term.toLowerCase();
    if (termLower.length < 3) return;

    let pos = 0;
    while (pos < textLower.length) {
      const idx = textLower.indexOf(termLower, pos);
      if (idx === -1) break;

      // Check woordgrenzen
      const before = idx > 0 ? textLower[idx - 1] : ' ';
      const after = idx + termLower.length < textLower.length ? textLower[idx + termLower.length] : ' ';

      if (this.isWordBoundary(before) && this.isWordBoundary(after)) {
        annotations.push({
          start: idx,
          end: idx + termLower.length,
          text: text.substring(idx, idx + termLower.length),
          ...data
        });
      }
      pos = idx + 1;
    }
  },

  isWordBoundary(char) {
    return /[\s.,;:!?()\[\]{}'"<>\-\/]/.test(char);
  },

  /**
   * Extract variabelen uit action logica
   */
  extractVariablesFromAction(action) {
    const vars = [];
    const extract = (obj) => {
      if (!obj || typeof obj !== 'object') return;
      if (typeof obj === 'string' && obj.startsWith('$')) {
        vars.push(obj.substring(1));
      }
      Object.values(obj).forEach(v => {
        if (typeof v === 'string' && v.startsWith('$')) {
          vars.push(v.substring(1));
        } else if (typeof v === 'object') {
          extract(v);
        }
      });
    };
    extract(action);
    return [...new Set(vars)];
  },

  /**
   * Verwijder overlappende annotaties, houd meest specifieke
   */
  deduplicateAnnotations(annotations) {
    annotations.sort((a, b) => a.start - b.start || b.end - a.end);

    const result = [];
    for (const ann of annotations) {
      const overlapping = result.findIndex(r =>
        (ann.start >= r.start && ann.start < r.end) ||
        (ann.end > r.start && ann.end <= r.end)
      );

      if (overlapping === -1) {
        result.push(ann);
      } else {
        // Houd de langere of meest specifieke (lawReference > openNorm > definition > input > output > logic)
        const priority = { lawReference: 6, openNorm: 5, definition: 4, input: 3, output: 2, logic: 1 };
        const existing = result[overlapping];
        if (priority[ann.type] > priority[existing.type] ||
            (priority[ann.type] === priority[existing.type] && ann.end - ann.start > existing.end - existing.start)) {
          result[overlapping] = ann;
        }
      }
    }
    return result.sort((a, b) => a.start - b.start);
  },

  /**
   * Render geannoteerde tekst als HTML
   */
  render(text, annotations) {
    if (!text) return '';
    if (!annotations || annotations.length === 0) return this.escapeHtml(text);

    let html = '';
    let lastIdx = 0;

    for (const ann of annotations) {
      // Tekst voor annotatie
      if (ann.start > lastIdx) {
        html += this.escapeHtml(text.substring(lastIdx, ann.start));
      }
      if (ann.start < lastIdx) continue;

      // Geannoteerde span
      const color = this.colors[ann.type] || this.colors.input;
      const dataAttrs = `data-type="${ann.type}" data-name="${this.escapeHtml(ann.name)}"`;

      // Law references worden als klikbare links gerenderd
      if (ann.type === 'lawReference' && ann.url) {
        html += `<a href="${this.escapeHtml(ann.url)}" target="_blank" rel="noopener noreferrer"
          class="annotation annotation--${ann.type}" ${dataAttrs}
          style="background:${color.bg}; border-bottom:2px solid ${color.border}; cursor:pointer; padding:1px 2px; border-radius:2px; text-decoration:none; color:inherit;"
          title="${this.escapeHtml(ann.description || ann.name)}">${this.escapeHtml(ann.name)}</a>`;
      } else {
        html += `<span class="annotation annotation--${ann.type}" ${dataAttrs}
          style="background:${color.bg}; border-bottom:2px solid ${color.border}; cursor:pointer; padding:1px 2px; border-radius:2px;"
          title="${this.escapeHtml(ann.description || ann.name)}">${this.escapeHtml(ann.text)}</span>`;
      }

      lastIdx = ann.end;
    }

    // Rest van tekst
    if (lastIdx < text.length) {
      html += this.escapeHtml(text.substring(lastIdx));
    }

    return html;
  },

  /**
   * Render als paragrafen met markdown support en Nederlandse wettekst structuur
   */
  renderFormatted(text, annotations) {
    const annotatedHtml = this.render(text, annotations);

    // Normaliseer tekst: vervang enkele newlines binnen paragrafen door spaties,
    // maar behoud newlines voor lijstitems (a., b., 1°., etc.)
    let normalized = annotatedHtml
      // Behoud newlines voor letter-items
      .replace(/\n\s*((?:[a-z]\.)\s+)/g, '\n\n$1')
      // Behoud newlines voor sub-items (1°, 2°)
      .replace(/\n\s*((?:\d+[°º])\.?\s*)/g, '\n\n$1')
      // Behoud newlines voor genummerde leden
      .replace(/\n\s*((?:\d+\.)\s+)/g, '\n\n$1');

    // Split op dubbele newlines voor paragrafen
    const paragraphs = normalized.split(/\n\n+/);

    let html = '';
    let inList = false;
    let listType = null; // 'letter' of 'number'

    paragraphs.forEach((para, idx) => {
      // Verwijder overtollige witruimte
      para = para.replace(/\s+/g, ' ').trim();
      if (!para) return;

      // Bold **text**
      para = para.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');

      // Check voor onderdeel-nummering (a., b., etc.)
      const onderdeelMatch = para.match(/^([a-z])\.\s+/);
      if (onderdeelMatch) {
        const letter = onderdeelMatch[1];
        const content = para.substring(onderdeelMatch[0].length);
        html += `<div class="onderdeel" style="display:flex; gap:8px; margin-left:24px; margin-bottom:6px;">
          <span style="font-weight:500; color:#64748b; min-width:20px;">${letter}.</span>
          <span style="flex:1;">${content}</span>
        </div>`;
        return;
      }

      // Check voor sub-onderdeel nummering (1°, 2°, etc.)
      const subMatch = para.match(/^(\d+)[°º]\.?\s*/);
      if (subMatch) {
        const num = subMatch[1];
        const content = para.substring(subMatch[0].length);
        html += `<div class="sub-onderdeel" style="display:flex; gap:8px; margin-left:48px; margin-bottom:4px;">
          <span style="color:#94a3b8; min-width:24px;">${num}°.</span>
          <span style="flex:1;">${content}</span>
        </div>`;
        return;
      }

      // Check voor lid-nummering (1., 2., etc. aan begin)
      const lidMatch = para.match(/^(\d+)\.\s+/);
      if (lidMatch) {
        const lidNum = lidMatch[1];
        const content = para.substring(lidMatch[0].length);
        html += `<div class="lid" style="display:flex; gap:8px; margin-bottom:10px;">
          <span style="font-weight:600; color:#475569; min-width:24px;">${lidNum}.</span>
          <span style="flex:1;">${content}</span>
        </div>`;
        return;
      }

      // Reguliere paragraaf
      html += `<p style="margin-bottom:10px; line-height:1.6;">${para}</p>`;
    });

    return html;
  },

  escapeHtml(str) {
    if (!str) return '';
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  },

  /**
   * Setup interactieve functies
   * @param {HTMLElement} container - Container element
   * @param {Object} article - Artikel data
   * @param {Function} onSave - Callback voor legacy open norm opslaan
   * @param {Function} onSaveAnnotation - Callback voor W3C annotation opslaan
   */
  setup(container, article, onSave, onSaveAnnotation) {
    // Tooltip
    let tooltip = document.getElementById('ann-tooltip');
    if (!tooltip) {
      tooltip = document.createElement('div');
      tooltip.id = 'ann-tooltip';
      tooltip.className = 'ann-tooltip';
      document.body.appendChild(tooltip);
    }

    // Hover op annotaties
    container.addEventListener('mouseover', e => {
      const ann = e.target.closest('.annotation');
      if (ann) {
        const type = ann.dataset.type;
        const name = ann.dataset.name;
        const desc = ann.title;

        tooltip.innerHTML = `
          <div style="font-weight:600; margin-bottom:4px;">${name}</div>
          <div style="font-size:0.75rem; color:#64748b; text-transform:uppercase; margin-bottom:4px;">${this.typeLabel(type)}</div>
          ${desc ? `<div style="font-size:0.8125rem;">${this.escapeHtml(desc)}</div>` : ''}
        `;

        const rect = ann.getBoundingClientRect();
        tooltip.style.left = `${rect.left}px`;
        tooltip.style.top = `${rect.bottom + 6}px`;
        tooltip.style.display = 'block';
      }
    });

    container.addEventListener('mouseout', e => {
      if (e.target.closest('.annotation')) {
        tooltip.style.display = 'none';
      }
    });

    // Text selectie voor annotatie tagging
    container.addEventListener('mouseup', e => {
      const selection = window.getSelection();
      const selectedText = selection?.toString().trim();

      if (selectedText && selectedText.length > 2) {
        this.showTagPopup(e, selectedText, article, onSave, onSaveAnnotation);
      }
    });
  },

  typeLabel(type) {
    return {
      definition: 'Definitie',
      input: 'Input',
      output: 'Output',
      openNorm: 'Open Norm',
      logic: 'Logica',
      lawReference: 'Wet Referentie'
    }[type] || type;
  },

  /**
   * Toon popup om geselecteerde tekst te taggen (W3C annotation)
   * @param {Event} event - Mouse event
   * @param {string} selectedText - Geselecteerde tekst
   * @param {Object} article - Artikel data
   * @param {Function} onSave - Callback voor opslaan (legacy open norm)
   * @param {Function} onSaveAnnotation - Callback voor W3C annotation opslaan
   */
  showTagPopup(event, selectedText, article, onSave, onSaveAnnotation) {
    // Verwijder bestaande popup
    document.getElementById('tag-popup')?.remove();

    // Get selection position for TextQuoteSelector
    const selection = window.getSelection();
    const range = selection.getRangeAt(0);
    // Clone de range zodat we die later kunnen gebruiken voor highlighting
    const savedRange = range.cloneRange();
    const container = range.startContainer.parentElement?.closest('.article-text');
    const fullText = article.text || '';

    // Calculate offset in full text using DOM Range for accurate positioning
    let startOffset = 0;

    if (container) {
      // Walk through text nodes to find actual offset
      const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT);
      let currentOffset = 0;
      let node;

      while ((node = walker.nextNode())) {
        if (node === range.startContainer) {
          startOffset = currentOffset + range.startOffset;
          break;
        }
        currentOffset += node.textContent.length;
      }
    }

    // Fallback: zoek in plain text met context
    if (startOffset === 0) {
      const idx = fullText.indexOf(selectedText);
      if (idx !== -1) {
        startOffset = idx;
      } else {
        // Fuzzy match
        startOffset = fullText.toLowerCase().indexOf(selectedText.toLowerCase());
      }
    }

    // Get existing tags
    const existingTags = this.getExistingTags(article);

    const popup = document.createElement('div');
    popup.id = 'tag-popup';
    popup.innerHTML = `
      <div style="background:white; border:1px solid #e2e8f0; border-radius:8px; box-shadow:0 4px 12px rgba(0,0,0,0.15); padding:16px; min-width:360px; max-width:450px; max-height:80vh; overflow-y:auto;">
        <div style="font-weight:600; margin-bottom:8px;">Annotatie toevoegen</div>
        <div style="font-size:0.8125rem; color:#64748b; margin-bottom:12px; padding:8px; background:#f8fafc; border-radius:4px;">
          "${this.escapeHtml(selectedText.substring(0, 60))}${selectedText.length > 60 ? '...' : ''}"
        </div>

        <!-- Existing tags section -->
        ${existingTags.length > 0 ? `
        <div style="margin-bottom:12px;">
          <label style="font-size:0.8125rem; font-weight:500; color:#475569; display:block; margin-bottom:6px;">Bestaande tags in dit artikel:</label>
          <div style="max-height:120px; overflow-y:auto; border:1px solid #e2e8f0; border-radius:4px; padding:8px;">
            ${existingTags.map((tag, idx) => `
              <label style="display:flex; align-items:center; gap:8px; font-size:0.8125rem; cursor:pointer; padding:4px 0;">
                <input type="radio" name="existingTag" value="${idx}" style="margin:0;">
                <span style="font-family:monospace;">${this.escapeHtml(tag.name)}</span>
                <span style="font-size:0.6875rem; padding:1px 4px; background:${this.colors[tag.type]?.bg || '#f1f5f9'}; border-radius:2px;">${tag.type}</span>
                ${tag.status ? `<span style="font-size:0.6875rem; color:#64748b;">(${tag.status})</span>` : ''}
              </label>
            `).join('')}
            <label style="display:flex; align-items:center; gap:8px; font-size:0.8125rem; cursor:pointer; padding:4px 0; border-top:1px solid #e2e8f0; margin-top:4px; padding-top:8px;">
              <input type="radio" name="existingTag" value="new" checked style="margin:0;">
              <span style="font-weight:500;">Nieuwe tag aanmaken...</span>
            </label>
          </div>
        </div>
        ` : ''}

        <!-- New tag section -->
        <div id="new-tag-section">
          <!-- W3C Motivation Type -->
          <div style="margin-bottom:12px;">
            <label style="font-size:0.8125rem; font-weight:500; color:#475569; display:block; margin-bottom:6px;">Type (W3C motivation):</label>
            <div style="display:flex; flex-direction:column; gap:6px;">
              <label style="display:flex; align-items:center; gap:8px; font-size:0.8125rem; cursor:pointer;">
                <input type="radio" name="motivation" value="commenting" checked style="margin:0;">
                <span><strong>Commenting</strong> - Open norm / uitleg</span>
              </label>
              <label style="display:flex; align-items:center; gap:8px; font-size:0.8125rem; cursor:pointer;">
                <input type="radio" name="motivation" value="linking" style="margin:0;">
                <span><strong>Linking</strong> - Referentie naar andere wet</span>
              </label>
              <label style="display:flex; align-items:center; gap:8px; font-size:0.8125rem; cursor:pointer;">
                <input type="radio" name="motivation" value="classifying" style="margin:0;">
                <span><strong>Classifying</strong> - Definition/Input/Output</span>
              </label>
              <label style="display:flex; align-items:center; gap:8px; font-size:0.8125rem; cursor:pointer;">
                <input type="radio" name="motivation" value="tagging" style="margin:0;">
                <span><strong>Tagging</strong> - Machine-readable label</span>
              </label>
            </div>
          </div>

          <!-- Conditional fields container -->
          <div id="conditional-fields">
            <!-- Commenting fields (default) -->
            <div id="fields-commenting">
              <textarea id="tag-desc" placeholder="Beschrijving (waarom is dit een open norm?)" rows="2"
                style="width:100%; padding:8px; border:1px solid #cbd5e1; border-radius:4px; font-size:0.875rem; resize:none; margin-bottom:8px;"></textarea>
            </div>

            <!-- Classifying fields (hidden by default) -->
            <div id="fields-classifying" style="display:none;">
              <div style="margin-bottom:8px;">
                <label style="font-size:0.75rem; color:#64748b;">Variabele naam:</label>
                <input type="text" id="varName" placeholder="bijv: is_gezamenlijke_huishouding"
                  style="width:100%; padding:6px; border:1px solid #cbd5e1; border-radius:4px; font-size:0.8125rem; font-family:monospace;">
              </div>
              <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px; margin-bottom:8px;">
                <div>
                  <label style="font-size:0.75rem; color:#64748b;">Classificatie:</label>
                  <select id="classification" style="width:100%; padding:6px; border:1px solid #cbd5e1; border-radius:4px; font-size:0.8125rem;">
                    <option value="definition">Definition</option>
                    <option value="input">Input</option>
                    <option value="output">Output</option>
                    <option value="parameter">Parameter</option>
                    <option value="open_norm">Open Norm</option>
                  </select>
                </div>
                <div>
                  <label style="font-size:0.75rem; color:#64748b;">Data type:</label>
                  <select id="dataType" style="width:100%; padding:6px; border:1px solid #cbd5e1; border-radius:4px; font-size:0.8125rem;">
                    <option value="boolean">Boolean</option>
                    <option value="string">String</option>
                    <option value="integer">Integer</option>
                    <option value="number">Number</option>
                    <option value="date">Date</option>
                    <option value="amount">Amount (money)</option>
                  </select>
                </div>
              </div>

              <!-- Source fields for input -->
              <div id="fields-source" style="display:none; margin-bottom:8px; padding:8px; background:#f8fafc; border-radius:4px;">
                <label style="font-size:0.75rem; color:#64748b; display:block; margin-bottom:4px;">Bron (voor inputs):</label>
                <div style="display:grid; grid-template-columns:1fr 1fr; gap:8px;">
                  <input type="text" id="sourceRegulation" placeholder="regulation ID"
                    style="padding:6px; border:1px solid #cbd5e1; border-radius:4px; font-size:0.8125rem;">
                  <input type="text" id="sourceOutput" placeholder="output naam"
                    style="padding:6px; border:1px solid #cbd5e1; border-radius:4px; font-size:0.8125rem;">
                </div>
                <label style="display:flex; align-items:center; gap:6px; margin-top:6px; font-size:0.8125rem;">
                  <input type="checkbox" id="humanInput" style="margin:0;">
                  <span>Vereist menselijke beoordeling</span>
                </label>
              </div>

              <!-- Parameters section -->
              <div id="fields-parameters" style="margin-bottom:8px;">
                <label style="font-size:0.75rem; color:#64748b; display:block; margin-bottom:4px;">Parameters (optioneel):</label>
                <div id="param-list" style="margin-bottom:6px;"></div>
                <button type="button" id="add-param" style="font-size:0.75rem; padding:4px 8px; border:1px dashed #cbd5e1; border-radius:4px; background:white; cursor:pointer;">+ Parameter</button>
              </div>

              <textarea id="classify-desc" placeholder="Beschrijving" rows="2"
                style="width:100%; padding:8px; border:1px solid #cbd5e1; border-radius:4px; font-size:0.875rem; resize:none; margin-bottom:8px;"></textarea>
            </div>

            <!-- Linking fields (hidden by default) -->
            <div id="fields-linking" style="display:none;">
              <div style="margin-bottom:8px;">
                <label style="font-size:0.75rem; color:#64748b;">Bron wet (regelrecht:// URI of BWB ID):</label>
                <input type="text" id="linkTarget" placeholder="bijv: participatiewet of BWBR0015703"
                  style="width:100%; padding:6px; border:1px solid #cbd5e1; border-radius:4px; font-size:0.8125rem;">
              </div>
            </div>

            <!-- Tagging fields (hidden by default) -->
            <div id="fields-tagging" style="display:none;">
              <div style="margin-bottom:8px;">
                <label style="font-size:0.75rem; color:#64748b;">Classificatie:</label>
                <select id="tagClassification" style="width:100%; padding:6px; border:1px solid #cbd5e1; border-radius:4px; font-size:0.8125rem;">
                  <option value="definition">Definition</option>
                  <option value="input">Input</option>
                  <option value="output">Output</option>
                  <option value="parameter">Parameter</option>
                  <option value="open_norm">Open Norm</option>
                </select>
              </div>
              <div style="margin-bottom:8px;">
                <label style="font-size:0.75rem; color:#64748b;">Variabele naam:</label>
                <input type="text" id="tagName" placeholder="bijv: is_alleenstaande"
                  style="width:100%; padding:6px; border:1px solid #cbd5e1; border-radius:4px; font-size:0.8125rem; font-family:monospace;">
              </div>
              <div style="margin-bottom:8px;">
                <label style="font-size:0.75rem; color:#64748b;">Data type:</label>
                <select id="tagDataType" style="width:100%; padding:6px; border:1px solid #cbd5e1; border-radius:4px; font-size:0.8125rem;">
                  <option value="boolean">Boolean</option>
                  <option value="string">String</option>
                  <option value="integer">Integer</option>
                  <option value="number">Number</option>
                  <option value="date">Date</option>
                  <option value="amount">Amount (money)</option>
                </select>
              </div>
              <textarea id="tagDescription" placeholder="Beschrijving (optioneel)" rows="2"
                style="width:100%; padding:8px; border:1px solid #cbd5e1; border-radius:4px; font-size:0.875rem; resize:none; margin-bottom:8px;"></textarea>
            </div>
          </div>
        </div>

        <div style="display:flex; gap:8px; justify-content:flex-end; margin-top:12px;">
          <button id="tag-cancel" style="padding:6px 12px; border:1px solid #e2e8f0; border-radius:4px; background:white; cursor:pointer; font-size:0.8125rem;">Annuleren</button>
          <button id="tag-save" style="padding:6px 12px; border:none; border-radius:4px; background:#154273; color:white; cursor:pointer; font-size:0.8125rem;">Opslaan</button>
        </div>
      </div>
    `;
    popup.style.cssText = `position:fixed; z-index:10000; left:${Math.min(event.clientX, window.innerWidth - 470)}px; top:${Math.min(event.clientY + 10, window.innerHeight - 550)}px;`;

    document.body.appendChild(popup);

    // Track existing tags for reference
    const existingTagsData = existingTags;

    // Toggle new tag section based on existing tag selection
    const existingTagRadios = popup.querySelectorAll('input[name="existingTag"]');
    const newTagSection = popup.querySelector('#new-tag-section');
    existingTagRadios.forEach(radio => {
      radio.addEventListener('change', () => {
        if (newTagSection) {
          newTagSection.style.display = radio.value === 'new' ? 'block' : 'none';
        }
      });
    });

    // Toggle conditional fields based on motivation selection
    const motivationRadios = popup.querySelectorAll('input[name="motivation"]');
    motivationRadios.forEach(radio => {
      radio.addEventListener('change', () => {
        // Hide all field groups
        popup.querySelectorAll('#conditional-fields > div').forEach(el => el.style.display = 'none');
        // Show selected field group
        const fieldsEl = popup.querySelector(`#fields-${radio.value}`);
        if (fieldsEl) fieldsEl.style.display = 'block';
      });
    });

    // Toggle source fields based on classification
    const classificationSelect = popup.querySelector('#classification');
    const sourceFields = popup.querySelector('#fields-source');
    if (classificationSelect && sourceFields) {
      classificationSelect.addEventListener('change', () => {
        sourceFields.style.display = classificationSelect.value === 'input' ? 'block' : 'none';
      });
    }

    // Auto-generate variable name from selected text
    const varNameInput = popup.querySelector('#varName');
    if (varNameInput) {
      varNameInput.value = TextAnnotator._textToVariableName(selectedText);
    }

    // Parameter add/remove functionality
    const paramList = popup.querySelector('#param-list');
    const addParamBtn = popup.querySelector('#add-param');
    let paramCounter = 0;

    const addParameterRow = () => {
      paramCounter++;
      const row = document.createElement('div');
      row.className = 'param-row';
      row.style.cssText = 'display:grid; grid-template-columns:1fr 80px 60px 24px; gap:4px; margin-bottom:4px; align-items:center;';
      row.innerHTML = `
        <input type="text" placeholder="naam" class="param-name" style="padding:4px; border:1px solid #cbd5e1; border-radius:3px; font-size:0.75rem;">
        <select class="param-type" style="padding:4px; border:1px solid #cbd5e1; border-radius:3px; font-size:0.75rem;">
          <option value="string">string</option>
          <option value="number">number</option>
          <option value="boolean">boolean</option>
          <option value="date">date</option>
        </select>
        <label style="display:flex; align-items:center; gap:2px; font-size:0.7rem;">
          <input type="checkbox" class="param-required" style="margin:0;">req
        </label>
        <button type="button" class="remove-param" style="width:20px; height:20px; border:none; background:#fee2e2; color:#dc2626; border-radius:3px; cursor:pointer; font-size:0.75rem;">x</button>
      `;
      row.querySelector('.remove-param').onclick = () => row.remove();
      paramList.appendChild(row);
    };

    if (addParamBtn) {
      addParamBtn.onclick = addParameterRow;
    }

    // Event handlers
    popup.querySelector('#tag-cancel').onclick = () => popup.remove();
    popup.querySelector('#tag-save').onclick = async () => {
      // Check if using existing tag
      const selectedExisting = popup.querySelector('input[name="existingTag"]:checked');
      if (selectedExisting && selectedExisting.value !== 'new') {
        const tagIdx = parseInt(selectedExisting.value);
        const existingTag = existingTagsData[tagIdx];
        if (existingTag && onSaveAnnotation) {
          // Create annotation linking to existing tag (RFC-005 compliant)
          const selector = TextAnnotator.createSelector(fullText, selectedText, startOffset >= 0 ? startOffset : 0);
          const annotation = {
            type: 'Annotation',
            motivation: 'classifying',
            resolution: 'found',  // RFC-005
            target: {
              source: `regelrecht://${article.regulationId || 'unknown'}`,
              article: article.number,
              selector: selector
            },
            body: {
              type: 'TextualBody',
              purpose: 'classifying',  // RFC-005: purpose matches motivation
              // regelrecht extensions
              name: existingTag.name,
              classification: existingTag.type,
              data_type: existingTag.dataType || 'boolean',
              existing_reference: true
            }
          };
          try {
            await onSaveAnnotation(annotation);
            TextAnnotator.highlightSelection(savedRange, annotation);
            TextAnnotator.showToast(`Gelinkt aan ${existingTag.name}`, 'success');
          } catch (e) {
            TextAnnotator.showToast('Fout bij opslaan', 'error');
          }
          popup.remove();
          return;
        }
      }

      const motivation = popup.querySelector('input[name="motivation"]:checked').value;

      // Create TextQuoteSelector (RFC-005)
      const selector = TextAnnotator.createSelector(fullText, selectedText, startOffset >= 0 ? startOffset : 0);

      // Build W3C annotation (RFC-005 compliant)
      const annotation = {
        type: 'Annotation',
        motivation: motivation,
        resolution: 'found',  // RFC-005: text was just selected, so it's found
        target: {
          source: `regelrecht://${article.regulationId || 'unknown'}`,
          article: article.number,
          selector: selector
        },
        body: {
          type: 'TextualBody',
          purpose: motivation  // RFC-005: purpose matches motivation by default
        }
      };

      // Add workflow for motivations that need review (RFC-005)
      if (['commenting', 'questioning', 'editing'].includes(motivation)) {
        annotation.workflow = 'open';
      }

      // Add motivation-specific fields
      if (motivation === 'commenting') {
        const description = popup.querySelector('#tag-desc').value;
        annotation.body.value = description || `Open norm: ${selectedText}`;
        annotation.body.purpose = 'commenting';
        // regelrecht extension: classification for open norms
        annotation.body.classification = 'open_norm';
      } else if (motivation === 'classifying') {
        const varName = popup.querySelector('#varName')?.value || TextAnnotator._textToVariableName(selectedText);
        annotation.body.purpose = 'classifying';
        // regelrecht extensions for machine-readable metadata
        annotation.body.name = varName;
        annotation.body.classification = popup.querySelector('#classification').value;
        annotation.body.data_type = popup.querySelector('#dataType').value;
        annotation.body.description = popup.querySelector('#classify-desc').value;

        // Add source for inputs
        if (annotation.body.classification === 'input') {
          const sourceReg = popup.querySelector('#sourceRegulation')?.value;
          const sourceOut = popup.querySelector('#sourceOutput')?.value;
          const humanInput = popup.querySelector('#humanInput')?.checked;
          if (sourceReg || sourceOut || humanInput) {
            annotation.body.source = {};
            if (sourceReg) annotation.body.source.regulation = sourceReg;
            if (sourceOut) annotation.body.source.output = sourceOut;
            if (humanInput) annotation.body.source.human_input = true;
          }
        }

        // Collect parameters
        const paramRows = paramList?.querySelectorAll('.param-row') || [];
        if (paramRows.length > 0) {
          annotation.body.parameters = [];
          paramRows.forEach(row => {
            const name = row.querySelector('.param-name')?.value?.trim();
            const type = row.querySelector('.param-type')?.value || 'string';
            const required = row.querySelector('.param-required')?.checked || false;
            if (name) {
              annotation.body.parameters.push({ name, type, required });
            }
          });
        }
      } else if (motivation === 'linking') {
        let linkTarget = popup.querySelector('#linkTarget').value;
        // Convert BWB ID to law_id if possible
        if (linkTarget.match(/^BWBR[0-9]{7}$/)) {
          const lawId = TextAnnotator.bwbToLawId(linkTarget);
          if (lawId) linkTarget = lawId;
        }
        // RFC-005: SpecificResource for links
        annotation.body.type = 'SpecificResource';
        annotation.body.source = linkTarget.startsWith('regelrecht://') ? linkTarget : `regelrecht://${linkTarget}`;
        annotation.body.purpose = 'linking';
      } else if (motivation === 'tagging') {
        const tagName = popup.querySelector('#tagName')?.value || TextAnnotator._textToVariableName(selectedText);
        annotation.body.purpose = 'tagging';
        // regelrecht extensions for machine-readable metadata
        annotation.body.name = tagName;
        annotation.body.classification = popup.querySelector('#tagClassification').value;
        annotation.body.data_type = popup.querySelector('#tagDataType').value;
        annotation.body.description = popup.querySelector('#tagDescription')?.value || '';
        annotation.body.value = selectedText;
      }

      // Call the W3C annotation save callback if provided
      if (onSaveAnnotation) {
        try {
          await onSaveAnnotation(annotation);
          // Highlight de selectie direct in de DOM
          TextAnnotator.highlightSelection(savedRange, annotation);
          TextAnnotator.showToast('Annotatie opgeslagen', 'success');
        } catch (e) {
          TextAnnotator.showToast('Fout bij opslaan', 'error');
          console.error('Failed to save annotation:', e);
        }
      }
      // Also call legacy save for backwards compatibility (commenting = open norm)
      else if (onSave && motivation === 'commenting') {
        const term = selectedText.toLowerCase().replace(/\s+/g, '_').replace(/[^a-z0-9_]/g, '');
        await onSave({
          term: term,
          description: annotation.body.value
        });
        TextAnnotator.highlightSelection(savedRange, annotation);
        TextAnnotator.showToast('Open norm opgeslagen', 'success');
      }
      popup.remove();
    };

    // Sluit bij klik buiten
    setTimeout(() => {
      document.addEventListener('click', function close(e) {
        if (!popup.contains(e.target)) {
          popup.remove();
          document.removeEventListener('click', close);
        }
      });
    }, 100);
  },

  /**
   * Highlight de huidige selectie direct in de DOM (zonder refresh)
   * @param {Range} range - De DOM range van de selectie
   * @param {Object} annotation - De opgeslagen annotatie
   */
  highlightSelection(range, annotation) {
    if (!range || range.collapsed) return;

    const classification = annotation.body?.classification || 'input';
    const type = classification === 'open_norm' ? 'openNorm' : classification;
    const color = this.colors[type] || this.colors.input;
    const name = annotation.body?.name || annotation.target?.selector?.exact || 'annotatie';

    // Maak een span element voor de highlight
    const span = document.createElement('span');
    span.className = `annotation annotation--${type}`;
    span.dataset.type = type;
    span.dataset.name = name;
    span.style.cssText = `background:${color.bg}; border-bottom:2px solid ${color.border}; cursor:pointer; padding:1px 2px; border-radius:2px;`;
    span.title = annotation.body?.description || name;

    // Wrap de selectie in de span
    try {
      range.surroundContents(span);

      // Animatie voor feedback
      span.style.transition = 'all 0.3s ease';
      span.style.boxShadow = '0 0 0 3px rgba(34, 197, 94, 0.5)';
      setTimeout(() => {
        span.style.boxShadow = 'none';
      }, 1000);
    } catch (e) {
      // Als surroundContents faalt (bijv. bij partial selections), gebruik alternatieve methode
      console.warn('Could not highlight selection directly:', e);
    }
  },

  /**
   * Toon een toast notificatie
   * @param {string} message - Bericht om te tonen
   * @param {string} type - 'success', 'error', of 'info'
   */
  showToast(message, type = 'success') {
    const colors = {
      success: '#22c55e',
      error: '#ef4444',
      info: '#3b82f6'
    };

    const toast = document.createElement('div');
    toast.style.cssText = `
      position: fixed;
      bottom: 20px;
      right: 20px;
      background: ${colors[type] || colors.info};
      color: white;
      padding: 12px 20px;
      border-radius: 6px;
      z-index: 10001;
      font-size: 0.875rem;
      box-shadow: 0 4px 12px rgba(0,0,0,0.15);
      animation: slideIn 0.3s ease;
    `;
    toast.textContent = message;

    // Voeg animatie style toe als die nog niet bestaat
    if (!document.getElementById('toast-styles')) {
      const style = document.createElement('style');
      style.id = 'toast-styles';
      style.textContent = `
        @keyframes slideIn {
          from { transform: translateX(100%); opacity: 0; }
          to { transform: translateX(0); opacity: 1; }
        }
        @keyframes slideOut {
          from { transform: translateX(0); opacity: 1; }
          to { transform: translateX(100%); opacity: 0; }
        }
      `;
      document.head.appendChild(style);
    }

    document.body.appendChild(toast);

    setTimeout(() => {
      toast.style.animation = 'slideOut 0.3s ease forwards';
      setTimeout(() => toast.remove(), 300);
    }, 2000);
  },

  /**
   * Render legende
   */
  renderLegend() {
    return `
      <div style="display:flex; gap:16px; flex-wrap:wrap; padding:8px 0; font-size:0.8125rem; color:#475569;">
        <span><span style="display:inline-block; width:12px; height:12px; background:${this.colors.definition.bg}; border-bottom:2px solid ${this.colors.definition.border}; margin-right:4px;"></span>Definitie</span>
        <span><span style="display:inline-block; width:12px; height:12px; background:${this.colors.input.bg}; border-bottom:2px solid ${this.colors.input.border}; margin-right:4px;"></span>Input</span>
        <span><span style="display:inline-block; width:12px; height:12px; background:${this.colors.output.bg}; border-bottom:2px solid ${this.colors.output.border}; margin-right:4px;"></span>Output</span>
        <span><span style="display:inline-block; width:12px; height:12px; background:${this.colors.openNorm.bg}; border-bottom:2px solid ${this.colors.openNorm.border}; margin-right:4px;"></span>Open norm</span>
        <span><span style="display:inline-block; width:12px; height:12px; background:${this.colors.logic.bg}; border-bottom:2px solid ${this.colors.logic.border}; margin-right:4px;"></span>Logica</span>
        <span><span style="display:inline-block; width:12px; height:12px; background:${this.colors.lawReference.bg}; border-bottom:2px solid ${this.colors.lawReference.border}; margin-right:4px;"></span>Wet referentie</span>
      </div>
    `;
  },

  /**
   * Render inputs diagram (simpele versie)
   */
  renderInputs(article) {
    const inputs = article.input || [];
    if (inputs.length === 0) return '<p style="color:#64748b; font-style:italic;">Geen inputs</p>';

    // Groepeer per bron
    const bySource = {};
    const human = [];

    inputs.forEach(inp => {
      if (inp.source?.human_input) {
        human.push(inp);
      } else if (inp.source?.regulation) {
        const reg = inp.source.regulation;
        if (!bySource[reg]) bySource[reg] = [];
        bySource[reg].push(inp);
      }
    });

    let html = '';

    Object.entries(bySource).forEach(([reg, inps]) => {
      html += `
        <div style="margin-bottom:12px; padding:12px; background:#f8fafc; border-radius:6px; border-left:3px solid #3b82f6;">
          <div style="font-weight:500; margin-bottom:6px;">${reg.replace(/_/g, ' ')}</div>
          ${inps.map(i => `<div style="font-size:0.8125rem; color:#475569; font-family:monospace;">- ${i.name}</div>`).join('')}
        </div>
      `;
    });

    if (human.length > 0) {
      html += `
        <div style="margin-bottom:12px; padding:12px; background:rgba(220,38,38,0.05); border-radius:6px; border-left:3px solid #dc2626;">
          <div style="font-weight:500; color:#dc2626; margin-bottom:6px;">Menselijke beoordeling</div>
          ${human.map(i => `<div style="font-size:0.8125rem; color:#475569;">${i.name}</div>`).join('')}
        </div>
      `;
    }

    return html || '<p style="color:#64748b;">Geen externe inputs</p>';
  },

  /**
   * Render open normen lijst
   */
  renderOpenNorms(article) {
    const norms = article.machine_readable?.open_norms || [];
    if (norms.length === 0) return '<p style="color:#64748b; font-style:italic;">Geen open normen gemarkeerd</p>';

    return norms.map(norm => `
      <div style="margin-bottom:8px; padding:10px; background:#fff7ed; border-radius:6px; border-left:3px solid #f97316;">
        <div style="font-family:monospace; font-weight:500;">${norm.term}</div>
        <div style="font-size:0.8125rem; color:#475569; margin-top:4px;">${this.escapeHtml(norm.description)}</div>
      </div>
    `).join('');
  },

  /**
   * Analyseer alle variabelen en hun link-status naar de tekst
   * @param {string} text - Artikel tekst
   * @param {Object} article - Artikel data
   * @returns {Object} Variabelen met hun link-status en matches
   */
  analyzeVariableLinks(text, article) {
    const textLower = text.toLowerCase();
    const variables = [];

    // Collect all definitions
    if (article.definitions) {
      Object.entries(article.definitions).forEach(([name, def]) => {
        variables.push({
          name,
          type: 'definition',
          value: def.value !== undefined ? def.value : def,
          description: def.description,
          color: this.colors.definition
        });
      });
    }

    // Collect all inputs
    if (article.input) {
      article.input.forEach(inp => {
        variables.push({
          name: inp.name,
          type: 'input',
          dataType: inp.type,
          source: inp.source,
          description: inp.description,
          color: this.colors.input
        });
      });
    }

    // Collect all outputs
    if (article.output) {
      article.output.forEach(out => {
        variables.push({
          name: out.name,
          type: 'output',
          dataType: out.type,
          description: out.description,
          color: this.colors.output
        });
      });
    }

    // Collect all parameters
    if (article.parameters) {
      article.parameters.forEach(param => {
        variables.push({
          name: param.name,
          type: 'parameter',
          dataType: param.type,
          required: param.required,
          color: { bg: 'rgba(99, 102, 241, 0.2)', border: '#6366f1' }
        });
      });
    }

    // Find text matches for each variable
    variables.forEach(v => {
      v.matches = [];
      v.manualLinks = [];

      // Auto-detected matches
      const searchTerms = this.termToSearchWords(v.name);
      searchTerms.forEach(term => {
        const termLower = term.toLowerCase();
        if (termLower.length < 3) return;

        let pos = 0;
        while (pos < textLower.length) {
          const idx = textLower.indexOf(termLower, pos);
          if (idx === -1) break;

          const before = idx > 0 ? textLower[idx - 1] : ' ';
          const after = idx + termLower.length < textLower.length ? textLower[idx + termLower.length] : ' ';

          if (this.isWordBoundary(before) && this.isWordBoundary(after)) {
            v.matches.push({
              start: idx,
              end: idx + termLower.length,
              text: text.substring(idx, idx + termLower.length),
              auto: true
            });
          }
          pos = idx + 1;
        }
      });

      // Check for manual links from annotations
      if (article.annotations) {
        article.annotations.forEach(ann => {
          if (ann.body?.name === v.name || ann.body?.classification === v.type) {
            const resolved = this.resolveSelector(text, ann.target?.selector);
            if (resolved) {
              v.manualLinks.push({
                start: resolved.start,
                end: resolved.end,
                text: text.substring(resolved.start, resolved.end),
                annotation: ann
              });
            }
          }
        });
      }

      // Determine link status
      const totalLinks = v.matches.length + v.manualLinks.length;
      if (totalLinks === 0) {
        v.status = 'unlinked';
      } else if (totalLinks === 1) {
        v.status = 'linked';
      } else {
        v.status = 'multiple';
      }
    });

    return variables;
  },

  /**
   * Render interactieve Logic Linker panel
   * @param {Object} article - Artikel data
   * @param {string} containerId - ID van de tekst container voor highlighting
   * @returns {string} HTML
   */
  renderLogicLinker(article, containerId = 'text-container') {
    const text = article.text || '';
    const variables = this.analyzeVariableLinks(text, article);

    if (variables.length === 0) {
      return '<p style="color:#64748b; font-style:italic;">Geen variabelen gedefinieerd voor dit artikel</p>';
    }

    const statusIcons = {
      linked: '✓',
      unlinked: '○',
      multiple: '◎'
    };
    const statusColors = {
      linked: '#22c55e',
      unlinked: '#ef4444',
      multiple: '#f59e0b'
    };

    // Group by type
    const byType = {};
    variables.forEach(v => {
      if (!byType[v.type]) byType[v.type] = [];
      byType[v.type].push(v);
    });

    const typeLabels = {
      definition: 'Definities',
      input: 'Inputs',
      output: 'Outputs',
      parameter: 'Parameters'
    };

    let html = `
      <div class="logic-linker" data-container="${containerId}">
        <div style="margin-bottom:12px; padding:8px; background:#f0f9ff; border-radius:6px; font-size:0.75rem; color:#0369a1;">
          <strong>Tip:</strong> Klik op een variabele om de gekoppelde tekst te markeren.
          Klik op <span style="background:#154273; color:white; padding:1px 4px; border-radius:3px;">Link</span> om handmatig tekst te koppelen.
        </div>
    `;

    Object.entries(byType).forEach(([type, vars]) => {
      const color = this.colors[type] || { bg: '#f1f5f9', border: '#64748b' };
      html += `
        <div style="margin-bottom:16px;">
          <div style="font-size:0.75rem; font-weight:600; color:#64748b; text-transform:uppercase; margin-bottom:8px; display:flex; align-items:center; gap:6px;">
            <span style="display:inline-block; width:10px; height:10px; background:${color.bg}; border:2px solid ${color.border}; border-radius:2px;"></span>
            ${typeLabels[type] || type}
          </div>
      `;

      vars.forEach((v, idx) => {
        const allLinks = [...v.matches, ...v.manualLinks];
        html += `
          <div class="logic-var" data-var-name="${this.escapeHtml(v.name)}" data-var-type="${v.type}"
               style="margin-bottom:6px; padding:8px; background:white; border:1px solid #e2e8f0; border-radius:6px; cursor:pointer; transition:all 0.15s;">
            <div style="display:flex; align-items:center; justify-content:space-between;">
              <div style="display:flex; align-items:center; gap:8px;">
                <span style="color:${statusColors[v.status]}; font-size:1rem;" title="${v.status === 'linked' ? 'Gekoppeld' : v.status === 'unlinked' ? 'Niet gekoppeld' : 'Meerdere matches'}">
                  ${statusIcons[v.status]}
                </span>
                <span style="font-family:monospace; font-weight:500; font-size:0.8125rem;">${this.escapeHtml(v.name)}</span>
                ${v.dataType ? `<span style="font-size:0.6875rem; padding:1px 4px; background:#f1f5f9; border-radius:3px; color:#64748b;">${v.dataType}</span>` : ''}
              </div>
              <div style="display:flex; gap:4px;">
                <button class="link-btn" data-var-name="${this.escapeHtml(v.name)}" data-var-type="${v.type}"
                        style="padding:2px 6px; font-size:0.6875rem; background:#154273; color:white; border:none; border-radius:3px; cursor:pointer;">
                  Link
                </button>
              </div>
            </div>
            ${allLinks.length > 0 ? `
              <div style="margin-top:6px; padding-top:6px; border-top:1px solid #f1f5f9;">
                <div style="font-size:0.6875rem; color:#94a3b8; margin-bottom:4px;">${allLinks.length} match${allLinks.length > 1 ? 'es' : ''} in tekst:</div>
                <div style="display:flex; flex-wrap:wrap; gap:4px;">
                  ${allLinks.slice(0, 3).map(m => `
                    <span class="match-chip" data-start="${m.start}" data-end="${m.end}"
                          style="font-size:0.6875rem; padding:2px 6px; background:${color.bg}; border:1px solid ${color.border}; border-radius:3px; cursor:pointer;">
                      "${this.escapeHtml(m.text.substring(0, 20))}${m.text.length > 20 ? '...' : ''}"
                    </span>
                  `).join('')}
                  ${allLinks.length > 3 ? `<span style="font-size:0.6875rem; color:#94a3b8;">+${allLinks.length - 3} meer</span>` : ''}
                </div>
              </div>
            ` : `
              <div style="margin-top:6px; font-size:0.6875rem; color:#ef4444; font-style:italic;">
                Geen tekst gevonden - klik "Link" om handmatig te koppelen
              </div>
            `}
            ${v.source?.regulation ? `
              <div style="margin-top:4px; font-size:0.6875rem; color:#64748b;">
                Bron: ${v.source.regulation}${v.source.output ? ' → ' + v.source.output : ''}
              </div>
            ` : ''}
          </div>
        `;
      });

      html += '</div>';
    });

    html += '</div>';
    return html;
  },

  /**
   * Setup Logic Linker interactivity
   * @param {HTMLElement} linkerContainer - Logic linker panel element
   * @param {HTMLElement} textContainer - Text container element
   * @param {Object} article - Article data
   * @param {Function} onSaveAnnotation - Callback to save annotation
   */
  setupLogicLinker(linkerContainer, textContainer, article, onSaveAnnotation) {
    if (!linkerContainer || !textContainer) return;

    let linkMode = null; // { name, type } when in link mode
    const text = article.text || '';

    // Click on variable row to highlight matches
    linkerContainer.querySelectorAll('.logic-var').forEach(row => {
      row.addEventListener('click', (e) => {
        if (e.target.closest('.link-btn') || e.target.closest('.match-chip')) return;

        const varName = row.dataset.varName;
        const varType = row.dataset.varType;

        // Remove previous highlights
        textContainer.querySelectorAll('.temp-highlight').forEach(el => {
          el.classList.remove('temp-highlight');
          el.style.outline = '';
        });

        // Find and highlight matching annotations
        const annotations = textContainer.querySelectorAll('.annotation');
        annotations.forEach(ann => {
          if (ann.dataset.name === varName || ann.dataset.type === varType) {
            ann.classList.add('temp-highlight');
            ann.style.outline = '2px solid #3b82f6';
            ann.scrollIntoView({ behavior: 'smooth', block: 'center' });
          }
        });

        // Visual feedback on row
        linkerContainer.querySelectorAll('.logic-var').forEach(r => r.style.background = 'white');
        row.style.background = '#eff6ff';
      });
    });

    // Click on match chip to scroll to that location
    linkerContainer.querySelectorAll('.match-chip').forEach(chip => {
      chip.addEventListener('click', (e) => {
        e.stopPropagation();
        const start = parseInt(chip.dataset.start);
        // Find annotation at this position
        const annotations = textContainer.querySelectorAll('.annotation');
        annotations.forEach(ann => {
          // Highlight briefly
          ann.style.outline = '';
        });
        // Try to find the right annotation by text content position
        const matchText = chip.textContent.replace(/^"|"$/g, '').replace(/\.\.\./, '');
        annotations.forEach(ann => {
          if (ann.textContent.includes(matchText.substring(0, 10))) {
            ann.style.outline = '3px solid #3b82f6';
            ann.scrollIntoView({ behavior: 'smooth', block: 'center' });
            setTimeout(() => { ann.style.outline = ''; }, 2000);
          }
        });
      });
    });

    // Link button - enter link mode
    linkerContainer.querySelectorAll('.link-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();

        const varName = btn.dataset.varName;
        const varType = btn.dataset.varType;

        // Toggle link mode
        if (linkMode && linkMode.name === varName) {
          // Exit link mode
          linkMode = null;
          btn.textContent = 'Link';
          btn.style.background = '#154273';
          textContainer.style.cursor = '';
          linkerContainer.querySelector('.link-mode-banner')?.remove();
        } else {
          // Enter link mode
          linkMode = { name: varName, type: varType };

          // Update all buttons
          linkerContainer.querySelectorAll('.link-btn').forEach(b => {
            b.textContent = 'Link';
            b.style.background = '#154273';
          });
          btn.textContent = 'Annuleren';
          btn.style.background = '#dc2626';
          textContainer.style.cursor = 'crosshair';

          // Show banner
          linkerContainer.querySelector('.link-mode-banner')?.remove();
          const banner = document.createElement('div');
          banner.className = 'link-mode-banner';
          banner.style.cssText = 'position:sticky; top:0; padding:8px; background:#fef3c7; border:1px solid #f59e0b; border-radius:4px; margin-bottom:8px; font-size:0.75rem; color:#92400e;';
          banner.innerHTML = `<strong>Link modus:</strong> Selecteer tekst om te koppelen aan <code>${this.escapeHtml(varName)}</code>`;
          linkerContainer.insertBefore(banner, linkerContainer.firstChild);
        }
      });
    });

    // Handle text selection in link mode
    textContainer.addEventListener('mouseup', async (e) => {
      if (!linkMode) return;

      const selection = window.getSelection();
      const selectedText = selection?.toString().trim();

      if (selectedText && selectedText.length > 1) {
        // Calculate offset
        const range = selection.getRangeAt(0);
        let startOffset = 0;
        const walker = document.createTreeWalker(textContainer, NodeFilter.SHOW_TEXT);
        let currentOffset = 0;
        let node;
        while ((node = walker.nextNode())) {
          if (node === range.startContainer) {
            startOffset = currentOffset + range.startOffset;
            break;
          }
          currentOffset += node.textContent.length;
        }

        // Create annotation
        const selector = TextAnnotator.createSelector(text, selectedText, startOffset);
        const annotation = {
          type: 'Annotation',
          motivation: 'linking',
          resolution: 'found',
          target: {
            source: `regelrecht://${article.regulationId || 'unknown'}`,
            article: article.number,
            selector: selector
          },
          body: {
            type: 'TextualBody',
            purpose: 'linking',
            name: linkMode.name,
            classification: linkMode.type,
            value: `Linked to ${linkMode.name}`
          }
        };

        // Save annotation
        if (onSaveAnnotation) {
          await onSaveAnnotation(annotation);
        }

        // Exit link mode
        const activeBtn = linkerContainer.querySelector(`.link-btn[data-var-name="${linkMode.name}"]`);
        if (activeBtn) {
          activeBtn.textContent = 'Link';
          activeBtn.style.background = '#154273';
        }
        linkMode = null;
        textContainer.style.cursor = '';
        linkerContainer.querySelector('.link-mode-banner')?.remove();

        // Show success
        const toast = document.createElement('div');
        toast.style.cssText = 'position:fixed; bottom:20px; right:20px; background:#22c55e; color:white; padding:12px 20px; border-radius:6px; z-index:10001; font-size:0.875rem;';
        toast.textContent = `"${selectedText.substring(0, 30)}${selectedText.length > 30 ? '...' : ''}" gekoppeld aan ${linkMode?.name || 'variabele'}`;
        document.body.appendChild(toast);
        setTimeout(() => toast.remove(), 2000);
      }
    });
  }
};

// CSS voor tooltip
const style = document.createElement('style');
style.textContent = `
  .ann-tooltip {
    position: fixed;
    z-index: 10001;
    display: none;
    background: white;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    padding: 10px 12px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.1);
    max-width: 300px;
    font-size: 0.875rem;
  }
`;
document.head.appendChild(style);

// Export
window.TextAnnotator = TextAnnotator;
