/**
 * Left Panel: Article list with traditional legal styling
 * Uses Crimson Text serif font and parchment background
 */

import { useEditorStore } from '../store/editorStore';
import { BookOpen, FileText } from 'lucide-react';

export default function LeftPanel() {
  const {
    currentLaw,
    currentArticleId,
    selectArticle,
  } = useEditorStore();

  if (!currentLaw) {
    return (
      <div className="flex flex-col items-center justify-center h-full p-8 text-center">
        <BookOpen className="w-16 h-16 text-gray-400 mb-4" />
        <h2 className="text-xl font-serif font-semibold text-gray-700 mb-2">
          No Law Selected
        </h2>
        <p className="text-sm text-gray-600 font-serif">
          Please select a law or regulation from the dropdown menu above to begin.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Panel Header */}
      <div className="bg-parchment-100 border-b border-parchment-300 px-4 py-3">
        <h2 className="text-lg font-serif font-bold text-gray-800 flex items-center gap-2">
          <FileText className="w-5 h-5" />
          Articles
        </h2>
        <p className="text-xs text-gray-600 font-serif mt-1">
          {currentLaw.articles.length} article(s)
        </p>
      </div>

      {/* Article List */}
      <div className="flex-1 overflow-y-auto">
        {currentLaw.articles.map((article) => {
          const isSelected = currentArticleId === article.number;
          const hasMachineReadable = !!article.machine_readable;

          return (
            <button
              key={article.number}
              onClick={() => selectArticle(article.number)}
              className={`w-full text-left px-4 py-3 border-b border-parchment-200 transition-colors ${
                isSelected
                  ? 'bg-legal-blue text-white'
                  : 'hover:bg-parchment-100'
              }`}
            >
              {/* Article Number */}
              <div className="flex items-center justify-between mb-1">
                <span
                  className={`text-sm font-serif font-semibold ${
                    isSelected ? 'text-white' : 'text-gray-800'
                  }`}
                >
                  Article {article.number}
                </span>

                {/* Machine-readable indicator */}
                {hasMachineReadable && (
                  <span
                    className={`px-2 py-0.5 text-xs font-mono rounded ${
                      isSelected
                        ? 'bg-white text-legal-blue'
                        : 'bg-legal-green text-white'
                    }`}
                  >
                    M-R
                  </span>
                )}
              </div>

              {/* Article Text Preview */}
              <p
                className={`text-sm font-serif leading-relaxed line-clamp-3 ${
                  isSelected ? 'text-blue-100' : 'text-gray-700'
                }`}
              >
                {article.text}
              </p>

              {/* Machine-readable execution preview */}
              {hasMachineReadable && article.machine_readable?.execution && (
                <div className="mt-2">
                  <span
                    className={`text-xs font-mono ${
                      isSelected ? 'text-blue-200' : 'text-gray-500'
                    }`}
                  >
                    {getOperationSummary(article.machine_readable.execution)}
                  </span>
                </div>
              )}
            </button>
          );
        })}
      </div>

      {/* Panel Footer */}
      <div className="bg-parchment-100 border-t border-parchment-300 px-4 py-2">
        <p className="text-xs text-gray-600 font-serif italic">
          Select an article to view and edit its machine-readable logic
        </p>
      </div>
    </div>
  );
}

/**
 * Get a short summary of an operation for preview
 */
function getOperationSummary(execution: any): string {
  if (!execution || typeof execution !== 'object') {
    return '';
  }

  const op = execution.operation;
  if (!op) {
    return '';
  }

  // Show operation type and hint at complexity
  const valueCount = execution.values?.length || 0;
  const hasNested = !!execution.then || !!execution.else;

  if (hasNested) {
    return `${op} (conditional logic)`;
  }

  if (valueCount > 0) {
    return `${op} (${valueCount} values)`;
  }

  return op;
}
