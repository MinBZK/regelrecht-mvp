/**
 * RegelRecht Browser - JavaScript voor dynamisch laden van verordeningen
 */

const API_BASE = '';

// State
let regulations = [];
let relations = {};
let currentRegulation = null;
let currentArticle = null;

// DOM Elements
const regulationsList = document.getElementById('regulations-list');
const lawPane = document.getElementById('law-pane');
const articlePane = document.getElementById('article-pane');
const searchInput = document.getElementById('search-input');

/**
 * Laad alle verordeningen van de API
 */
async function loadRegulations() {
  try {
    const response = await fetch(`${API_BASE}/api/regulations`);
    regulations = await response.json();
    renderRegulationsList(regulations);
  } catch (error) {
    console.error('Fout bij laden verordeningen:', error);
    regulationsList.innerHTML = '<div class="error">Kon verordeningen niet laden</div>';
  }
}

/**
 * Laad relaties tussen verordeningen
 */
async function loadRelations() {
  try {
    const response = await fetch(`${API_BASE}/api/relations`);
    relations = await response.json();
  } catch (error) {
    console.error('Fout bij laden relaties:', error);
  }
}

/**
 * Laad een specifieke verordening
 */
async function loadRegulation(regId) {
  try {
    const response = await fetch(`${API_BASE}/api/regulation/${regId}`);
    currentRegulation = await response.json();
    renderLawPane(currentRegulation);
    // Selecteer eerste artikel als er artikelen zijn
    if (currentRegulation.articles.length > 0) {
      selectArticle(0);
    }
  } catch (error) {
    console.error('Fout bij laden verordening:', error);
  }
}

/**
 * Render de lijst van verordeningen (linker pane)
 */
function renderRegulationsList(regs) {
  // Groepeer op regulatory_layer
  const grouped = {
    WET: [],
    MINISTERIELE_REGELING: [],
    GEMEENTELIJKE_VERORDENING: [],
    ANDERE: []
  };

  regs.forEach(reg => {
    const layer = reg.regulatory_layer;
    if (grouped[layer]) {
      grouped[layer].push(reg);
    } else {
      grouped.ANDERE.push(reg);
    }
  });

  let html = '';

  // Wetten
  if (grouped.WET.length > 0) {
    html += '<div class="list__section">Wetten</div>';
    grouped.WET.forEach(reg => {
      html += renderRegulationItem(reg);
    });
  }

  // Ministeriele regelingen
  if (grouped.MINISTERIELE_REGELING.length > 0) {
    html += '<div class="list__section">Ministeriële regelingen</div>';
    grouped.MINISTERIELE_REGELING.forEach(reg => {
      html += renderRegulationItem(reg);
    });
  }

  // Gemeentelijke verordeningen
  if (grouped.GEMEENTELIJKE_VERORDENING.length > 0) {
    html += '<div class="list__section">Gemeentelijke verordeningen</div>';
    grouped.GEMEENTELIJKE_VERORDENING.forEach(reg => {
      html += renderRegulationItem(reg);
    });
  }

  // Andere
  if (grouped.ANDERE.length > 0) {
    html += '<div class="list__section">Overig</div>';
    grouped.ANDERE.forEach(reg => {
      html += renderRegulationItem(reg);
    });
  }

  regulationsList.innerHTML = html;

  // Event listeners
  document.querySelectorAll('.regulation-item').forEach(item => {
    item.addEventListener('click', (e) => {
      e.preventDefault();
      const regId = item.dataset.regId;
      document.querySelectorAll('.regulation-item').forEach(i => i.classList.remove('list__item--selected'));
      item.classList.add('list__item--selected');
      loadRegulation(regId);
    });
  });
}

function renderRegulationItem(reg) {
  return `
    <a href="#" class="list__item regulation-item" data-reg-id="${reg.id}">
      <div class="list__content">
        <div class="list__title">${escapeHtml(reg.name)}</div>
        <div class="list__subtitle">${reg.article_count} artikelen</div>
      </div>
      <svg class="list__chevron" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M6 4l4 4-4 4"/>
      </svg>
    </a>
  `;
}

/**
 * Render de wet pane (midden)
 */
function renderLawPane(reg) {
  const titleEl = lawPane.querySelector('.law-pane__title');
  const articlesList = lawPane.querySelector('.articles-list');

  titleEl.textContent = reg.name;

  let html = '';

  // Relaties sectie
  if (reg.relations) {
    const deps = reg.relations.depends_on || [];
    const usedBy = reg.relations.used_by || [];

    if (deps.length > 0 || usedBy.length > 0) {
      html += '<div class="list__section">Relaties</div>';

      if (deps.length > 0) {
        html += `
          <div class="relations-box relations-box--depends">
            <div class="relations-box__label">Gebruikt data van:</div>
            <div class="relations-box__list">
              ${deps.map(d => `<a href="#" class="relations-box__link" data-reg-id="${d}">${formatRegName(d)}</a>`).join('')}
            </div>
          </div>
        `;
      }

      if (usedBy.length > 0) {
        html += `
          <div class="relations-box relations-box--used-by">
            <div class="relations-box__label">Wordt gebruikt door:</div>
            <div class="relations-box__list">
              ${usedBy.map(d => `<a href="#" class="relations-box__link" data-reg-id="${d}">${formatRegName(d)}</a>`).join('')}
            </div>
          </div>
        `;
      }
    }
  }

  // Artikelen
  html += '<div class="list__section">Artikelen</div>';
  reg.articles.forEach((article, index) => {
    const hasLogic = article.actions && article.actions.length > 0;
    const outputs = article.output || [];

    html += `
      <a href="#" class="list__item article-item ${index === 0 ? 'list__item--selected' : ''}" data-article-index="${index}">
        <div class="list__content">
          <div class="list__title">
            Artikel ${escapeHtml(article.number)}
            ${hasLogic ? '<span class="badge badge--logic">Logic</span>' : ''}
          </div>
          ${outputs.length > 0 ? `<div class="list__subtitle">${outputs.map(o => o.name).join(', ')}</div>` : ''}
        </div>
        <svg class="list__chevron" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M6 4l4 4-4 4"/>
        </svg>
      </a>
    `;
  });

  articlesList.innerHTML = html;

  // Event listeners voor artikelen
  document.querySelectorAll('.article-item').forEach(item => {
    item.addEventListener('click', (e) => {
      e.preventDefault();
      const index = parseInt(item.dataset.articleIndex);
      document.querySelectorAll('.article-item').forEach(i => i.classList.remove('list__item--selected'));
      item.classList.add('list__item--selected');
      selectArticle(index);
    });
  });

  // Event listeners voor relatie links
  document.querySelectorAll('.relations-box__link').forEach(link => {
    link.addEventListener('click', (e) => {
      e.preventDefault();
      const regId = link.dataset.regId;
      // Update selection in left pane
      document.querySelectorAll('.regulation-item').forEach(i => {
        i.classList.toggle('list__item--selected', i.dataset.regId === regId);
      });
      loadRegulation(regId);
    });
  });

  lawPane.classList.remove('hidden');
}

/**
 * Selecteer en toon een artikel
 */
function selectArticle(index) {
  if (!currentRegulation || !currentRegulation.articles[index]) return;

  currentArticle = currentRegulation.articles[index];
  renderArticlePane(currentArticle);
  articlePane.classList.remove('hidden');
}

/**
 * Render het artikel pane (rechts)
 */
function renderArticlePane(article) {
  // Update tabs content
  renderTextTab(article);
  renderMachineTab(article);
  renderYamlTab(article);
}

// Track highlighting state
let textHighlightingEnabled = true;

function renderTextTab(article) {
  const container = document.getElementById('tab-text-content');

  // Get annotations if TextAnnotator is available
  let textContent;
  if (typeof TextAnnotator !== 'undefined' && textHighlightingEnabled) {
    const annotations = TextAnnotator.findAnnotations(article.text || '', article);
    textContent = TextAnnotator.renderFormatted(article.text || '', annotations);
  } else {
    textContent = formatLegalText(article.text);
  }

  container.innerHTML = `
    <div class="article-text-toolbar" style="display: flex; gap: 8px; margin-bottom: 16px; padding-bottom: 12px; border-bottom: 1px solid var(--color-slate-200, #e2e8f0);">
      <button class="toggle-highlight-btn" style="display: inline-flex; align-items: center; gap: 6px; padding: 6px 12px; background: ${textHighlightingEnabled ? 'var(--color-primary, #154273)' : 'var(--color-slate-100, #f1f5f9)'}; color: ${textHighlightingEnabled ? 'white' : 'var(--color-slate-700, #334155)'}; border: none; border-radius: 6px; font-size: 0.8125rem; cursor: pointer;">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
          <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
        </svg>
        Highlighting ${textHighlightingEnabled ? 'aan' : 'uit'}
      </button>
      <a href="/participatiewet.html" style="margin-left: auto; font-size: 0.75rem; color: var(--color-blue-600, #2563eb); align-self: center;">
        Open interactieve annotatie tool
      </a>
    </div>
    <div class="article-text-legend" id="text-legend" style="margin-bottom: 16px;"></div>
    <div class="article-text annotated-text" id="annotated-article-text">
      <h2>Artikel ${escapeHtml(article.number)}</h2>
      ${textContent}
    </div>
  `;

  // Render legend and setup interactions
  if (typeof TextAnnotator !== 'undefined') {
    document.getElementById('text-legend').innerHTML = TextAnnotator.renderLegend();
    TextAnnotator.setup(
      document.getElementById('annotated-article-text'),
      article,
      // Legacy open norm callback
      async (normData) => {
        const regId = currentRegulation?.id;
        if (!regId) return;
        try {
          const res = await fetch(`${API_BASE}/api/regulation/${regId}/article/${article.number}/open_norm`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(normData)
          });
          const result = await res.json();
          if (result.status === 'ok') {
            // Reload regulation data
            await loadRegulation(regId);
          }
        } catch (e) {
          console.error('Failed to save open norm:', e);
        }
      },
      // W3C annotation callback
      async (annotation) => {
        const regId = currentRegulation?.id;
        if (!regId) return;
        try {
          const res = await fetch(`${API_BASE}/api/regulation/${regId}/annotation`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(annotation)
          });
          const result = await res.json();
          if (result.status === 'ok') {
            // Show success toast
            const toast = document.createElement('div');
            toast.style.cssText = 'position:fixed; bottom:20px; right:20px; background:#22c55e; color:white; padding:12px 20px; border-radius:6px; z-index:10001; font-size:0.875rem;';
            toast.textContent = 'Annotatie opgeslagen';
            document.body.appendChild(toast);
            setTimeout(() => toast.remove(), 2000);
            // Reload regulation data to show new annotation
            await loadRegulation(regId);
          } else if (result.status === 'exists') {
            alert('Deze annotatie bestaat al');
          }
        } catch (e) {
          console.error('Failed to save annotation:', e);
          alert('Fout bij opslaan annotatie');
        }
      }
    );
  }

  // Toggle highlighting button
  container.querySelector('.toggle-highlight-btn')?.addEventListener('click', () => {
    textHighlightingEnabled = !textHighlightingEnabled;
    renderTextTab(article);
  });
}

function renderMachineTab(article) {
  const container = document.getElementById('tab-machine-content');

  let html = '';

  // Inputs diagram
  if (typeof TextAnnotator !== 'undefined') {
    html += `
      <div class="machine-view__section">
        <h4 class="machine-view__heading">Inputs van andere bronnen</h4>
        ${TextAnnotator.renderInputs(article)}
      </div>
    `;
  }

  // Open normen waarschuwing
  const machineReadable = article.machine_readable || {};
  if (machineReadable.requires_human_assessment) {
    html += `
      <div class="machine-view__human-warning" style="display: flex; gap: 12px; padding: 16px; background: rgba(220, 38, 38, 0.05); border: 1px solid rgba(220, 38, 38, 0.2); border-radius: 8px; margin-bottom: 20px;">
        <span style="color: #dc2626; font-size: 1.25rem;">&#9888;</span>
        <div>
          <strong style="display: block; margin-bottom: 4px; color: #dc2626;">Menselijke beoordeling vereist</strong>
          ${machineReadable.human_assessment_reason ? `<p style="margin: 0; font-size: 0.875rem; color: var(--color-slate-700, #334155);">${escapeHtml(machineReadable.human_assessment_reason)}</p>` : ''}
        </div>
      </div>
    `;
  }

  // Open normen uit YAML
  const openNorms = machineReadable.open_norms || [];
  if (openNorms.length > 0) {
    html += `
      <div class="machine-view__section">
        <h4 class="machine-view__heading">Gemarkeerde Open Normen</h4>
        <div class="machine-view__list machine-view__list--box">
          ${openNorms.map(norm => `
            <div class="machine-view__list-item" style="flex-direction: column; align-items: flex-start;">
              <span style="font-family: monospace; font-weight: 600; color: var(--color-slate-800, #1e293b);">${escapeHtml(norm.term)}</span>
              <span style="font-size: 0.8125rem; color: var(--color-slate-600, #475569); margin-top: 4px;">${escapeHtml(norm.description)}</span>
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }

  // Tekst preview
  if (article.text) {
    const preview = article.text.substring(0, 200) + (article.text.length > 200 ? '...' : '');
    html += `
      <div class="machine-view__text-preview">
        <p class="machine-view__preview-text">${escapeHtml(preview)}</p>
      </div>
    `;
  }

  // Produces metadata
  if (article.produces && Object.keys(article.produces).length > 0) {
    html += `
      <div class="machine-view__list machine-view__list--box">
        ${article.produces.legal_character ? `
          <div class="machine-view__list-item machine-view__list-item--metadata">
            <span class="machine-view__item-label">Juridische basis</span>
            <span class="machine-view__item-value">${escapeHtml(article.produces.legal_character)}</span>
          </div>
        ` : ''}
        ${article.produces.decision_type ? `
          <div class="machine-view__list-item machine-view__list-item--metadata">
            <span class="machine-view__item-label">Besluittype</span>
            <span class="machine-view__item-value">${escapeHtml(article.produces.decision_type)}</span>
          </div>
        ` : ''}
      </div>
    `;
  }

  // Definities
  if (article.definitions && Object.keys(article.definitions).length > 0) {
    html += `
      <div class="machine-view__section">
        <h4 class="machine-view__heading">Definities</h4>
        <div class="machine-view__list machine-view__list--box">
          ${Object.entries(article.definitions).map(([name, def]) => `
            <div class="machine-view__list-item">
              <span class="machine-view__item-title">${escapeHtml(name)}</span>
              <span class="machine-view__item-value">${formatValue(def.value || def)}</span>
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }

  // Parameters
  if (article.parameters && article.parameters.length > 0) {
    html += `
      <div class="machine-view__section">
        <h4 class="machine-view__heading">Parameters</h4>
        <p class="machine-view__subtitle">Nodig om dit artikel uit te kunnen voeren.</p>
        <div class="machine-view__list machine-view__list--box">
          ${article.parameters.map(param => `
            <div class="machine-view__list-item">
              <span class="machine-view__item-title">${escapeHtml(param.name)}</span>
              <span class="machine-view__item-type">(${escapeHtml(param.type)})</span>
              ${param.required ? '<span class="badge badge--required">verplicht</span>' : ''}
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }

  // Input (van andere regels)
  if (article.input && article.input.length > 0) {
    html += `
      <div class="machine-view__section">
        <h4 class="machine-view__heading">Invoer</h4>
        <p class="machine-view__subtitle">Van andere regels/wetten.</p>
        <div class="machine-view__list machine-view__list--box">
          ${article.input.map(inp => `
            <div class="machine-view__list-item machine-view__list-item--input">
              <div class="machine-view__item-main">
                <span class="machine-view__item-title">${escapeHtml(inp.name)}</span>
                <span class="machine-view__item-type">(${escapeHtml(inp.type)})</span>
              </div>
              ${inp.source && inp.source.regulation ? `
                <div class="machine-view__item-source">
                  <span class="source-label">van:</span>
                  <a href="#" class="source-link" data-reg-id="${inp.source.regulation}">
                    ${formatRegName(inp.source.regulation)}
                  </a>
                  ${inp.source.output ? `<span class="source-output">→ ${inp.source.output}</span>` : ''}
                </div>
              ` : ''}
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }

  // Output
  if (article.output && article.output.length > 0) {
    html += `
      <div class="machine-view__section">
        <h4 class="machine-view__heading">Uitvoer</h4>
        <p class="machine-view__subtitle">Beschikbaar voor andere regels.</p>
        <div class="machine-view__list machine-view__list--box">
          ${article.output.map(out => `
            <div class="machine-view__list-item">
              <span class="machine-view__item-title">${escapeHtml(out.name)}</span>
              <span class="machine-view__item-type">(${escapeHtml(out.type)})</span>
              ${out.type_spec ? `<span class="machine-view__item-spec">${formatTypeSpec(out.type_spec)}</span>` : ''}
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }

  // Acties (logica)
  if (article.actions && article.actions.length > 0) {
    html += `
      <div class="machine-view__section">
        <h4 class="machine-view__heading">Acties (Logica)</h4>
        <div class="machine-view__list machine-view__list--box">
          ${article.actions.map(action => `
            <div class="machine-view__list-item machine-view__list-item--action">
              <div class="action-output">${escapeHtml(action.output)}</div>
              <div class="action-logic">${formatAction(action)}</div>
            </div>
          `).join('')}
        </div>
      </div>
    `;
  }

  container.innerHTML = html || '<p class="machine-view__empty">Dit artikel heeft geen machine-leesbare specificatie.</p>';

  // Event listeners voor source links
  container.querySelectorAll('.source-link').forEach(link => {
    link.addEventListener('click', (e) => {
      e.preventDefault();
      const regId = link.dataset.regId;
      document.querySelectorAll('.regulation-item').forEach(i => {
        i.classList.toggle('list__item--selected', i.dataset.regId === regId);
      });
      loadRegulation(regId);
    });
  });
}

function renderYamlTab(article) {
  const container = document.getElementById('tab-yaml-content');

  // Bouw een subset van het artikel voor YAML weergave
  const yamlData = {
    number: article.number,
    text: article.text,
    url: article.url
  };

  if (Object.keys(article.definitions || {}).length > 0 ||
      (article.parameters && article.parameters.length > 0) ||
      (article.input && article.input.length > 0) ||
      (article.output && article.output.length > 0) ||
      (article.actions && article.actions.length > 0)) {
    yamlData.machine_readable = {};

    if (Object.keys(article.definitions || {}).length > 0) {
      yamlData.machine_readable.definitions = article.definitions;
    }

    if (article.parameters?.length || article.input?.length || article.output?.length || article.actions?.length) {
      yamlData.machine_readable.execution = {};
      if (article.produces) yamlData.machine_readable.execution.produces = article.produces;
      if (article.parameters?.length) yamlData.machine_readable.execution.parameters = article.parameters;
      if (article.input?.length) yamlData.machine_readable.execution.input = article.input;
      if (article.output?.length) yamlData.machine_readable.execution.output = article.output;
      if (article.actions?.length) yamlData.machine_readable.execution.actions = article.actions;
    }
  }

  // Simple YAML formatting
  const yamlStr = formatAsYaml(yamlData);

  container.innerHTML = `
    <div class="yaml-view">
      <pre><code>${escapeHtml(yamlStr)}</code></pre>
    </div>
  `;
}

/**
 * Helper functions
 */
function escapeHtml(str) {
  if (str === null || str === undefined) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function formatRegName(id) {
  return id.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
}

function formatValue(val) {
  if (typeof val === 'number') {
    // Check if it looks like eurocents
    if (val > 10000) {
      return '€ ' + (val / 100).toLocaleString('nl-NL', { minimumFractionDigits: 2 });
    }
    // Check if it's a percentage
    if (val < 1 && val > 0) {
      return (val * 100).toFixed(3) + '%';
    }
    return val.toLocaleString('nl-NL');
  }
  return String(val);
}

function formatTypeSpec(spec) {
  const parts = [];
  if (spec.unit) parts.push(spec.unit);
  if (spec.precision !== undefined) parts.push(`precision: ${spec.precision}`);
  return parts.join(', ');
}

function formatLegalText(text) {
  if (!text) return '';
  // Split into paragraphs and format
  return text.split('\n\n').map(para => {
    // Check if paragraph starts with a number (like "1." or "a.")
    const match = para.match(/^(\d+\.|[a-z]\.)\s*/);
    if (match) {
      return `<p><strong>${match[1]}</strong> ${escapeHtml(para.substring(match[0].length))}</p>`;
    }
    return `<p>${escapeHtml(para)}</p>`;
  }).join('');
}

function formatAction(action) {
  if (action.value !== undefined) {
    if (typeof action.value === 'object' && action.value.operation) {
      return formatOperation(action.value);
    }
    return escapeHtml(String(action.value));
  }
  if (action.operation) {
    return formatOperation(action);
  }
  if (action.resolve) {
    return `RESOLVE(${action.resolve.type}/${action.resolve.output})`;
  }
  return JSON.stringify(action);
}

function formatOperation(op) {
  if (!op || typeof op !== 'object') return String(op);

  const opName = op.operation;
  if (!opName) {
    if (op.value !== undefined) return formatOperationValue(op.value);
    return JSON.stringify(op);
  }

  // Format based on operation type
  switch (opName) {
    case 'AND':
    case 'OR':
      const conditions = (op.conditions || []).map(c => formatOperation(c));
      return `${opName}(${conditions.join(', ')})`;

    case 'IF':
      return `IF(${formatOperation(op.when)}) THEN ${formatOperationValue(op.then)} ELSE ${formatOperationValue(op.else)}`;

    case 'EQUALS':
    case 'NOT_EQUALS':
    case 'GREATER_THAN':
    case 'LESS_THAN':
    case 'GREATER_THAN_OR_EQUAL':
    case 'LESS_THAN_OR_EQUAL':
      return `${formatOperationValue(op.subject)} ${opName} ${formatOperationValue(op.value)}`;

    case 'ADD':
    case 'SUBTRACT':
    case 'MULTIPLY':
    case 'DIVIDE':
    case 'MAX':
    case 'MIN':
      const values = (op.values || []).map(v => formatOperationValue(v));
      return `${opName}(${values.join(', ')})`;

    default:
      return `${opName}(...)`;
  }
}

function formatOperationValue(val) {
  if (val === null || val === undefined) return 'null';
  if (typeof val === 'string') {
    if (val.startsWith('$')) return val;
    return `"${val}"`;
  }
  if (typeof val === 'number') return formatValue(val);
  if (typeof val === 'boolean') return val ? 'true' : 'false';
  if (typeof val === 'object') {
    if (val.operation) return formatOperation(val);
    return JSON.stringify(val);
  }
  return String(val);
}

function formatAsYaml(obj, indent = 0) {
  const spaces = '  '.repeat(indent);
  let result = '';

  if (Array.isArray(obj)) {
    obj.forEach(item => {
      if (typeof item === 'object' && item !== null) {
        result += `${spaces}- `;
        const itemYaml = formatAsYaml(item, indent + 1).trimStart();
        result += itemYaml;
      } else {
        result += `${spaces}- ${formatYamlValue(item)}\n`;
      }
    });
  } else if (typeof obj === 'object' && obj !== null) {
    Object.entries(obj).forEach(([key, value]) => {
      if (value === null || value === undefined) return;
      if (typeof value === 'object') {
        result += `${spaces}${key}:\n`;
        result += formatAsYaml(value, indent + 1);
      } else {
        result += `${spaces}${key}: ${formatYamlValue(value)}\n`;
      }
    });
  } else {
    result += `${formatYamlValue(obj)}\n`;
  }

  return result;
}

function formatYamlValue(val) {
  if (typeof val === 'string') {
    if (val.includes('\n')) {
      return '|-\n' + val.split('\n').map(line => '  ' + line).join('\n');
    }
    if (val.match(/[:#{}[\],&*?|<>=!%@`]/)) {
      return `"${val.replace(/"/g, '\\"')}"`;
    }
    return val;
  }
  if (typeof val === 'boolean') return val ? 'true' : 'false';
  return String(val);
}

/**
 * Search functionality
 */
function filterRegulations(query) {
  if (!query) {
    renderRegulationsList(regulations);
    return;
  }

  const q = query.toLowerCase();
  const filtered = regulations.filter(reg =>
    reg.name.toLowerCase().includes(q) ||
    reg.id.toLowerCase().includes(q)
  );
  renderRegulationsList(filtered);
}

/**
 * Initialize
 */
document.addEventListener('DOMContentLoaded', async () => {
  // Load data
  await Promise.all([loadRegulations(), loadRelations()]);

  // Setup search
  if (searchInput) {
    searchInput.addEventListener('input', (e) => {
      filterRegulations(e.target.value);
    });
  }

  // Listen for open norm changes to refresh text view
  document.addEventListener('openNormAdded', () => {
    if (currentArticle) {
      renderTextTab(currentArticle);
    }
  });

  // Listen for regulation navigation from interrelation diagram
  document.addEventListener('navigateToRegulation', (e) => {
    const regId = e.detail.regId;
    document.querySelectorAll('.regulation-item').forEach(i => {
      i.classList.toggle('list__item--selected', i.dataset.regId === regId);
    });
    loadRegulation(regId);
  });

  // Load first regulation if available
  if (regulations.length > 0) {
    loadRegulation(regulations[0].id);
  }
});
