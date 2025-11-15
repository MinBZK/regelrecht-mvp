/**
 * Article List - Left panel with clickable article list
 */

import { useEditorStore } from '../store/editorStore';
import { Search, BookOpen } from 'lucide-react';
import { useState } from 'react';

interface ArticleListProps {
  color?: string;
}

export default function ArticleList({ color = '#3B82F6' }: ArticleListProps) {
  const { currentLaw, currentArticleId, selectArticle } = useEditorStore();
  const [searchQuery, setSearchQuery] = useState('');

  if (!currentLaw) {
    return (
      <div className="flex flex-col items-center justify-center h-full p-8 text-center">
        <BookOpen className="w-16 h-16 text-gray-300 mb-4" />
        <p className="text-sm text-gray-500">Open a law to see articles</p>
      </div>
    );
  }

  // Filter articles based on search
  const filteredArticles = currentLaw.articles.filter(article =>
    searchQuery
      ? article.number.toLowerCase().includes(searchQuery.toLowerCase()) ||
        article.text.toLowerCase().includes(searchQuery.toLowerCase())
      : true
  );

  return (
    <div className="flex flex-col h-full bg-white">
      {/* Header */}
      <div
        className="px-4 py-3 border-b border-gray-200"
        style={{ borderLeftWidth: '4px', borderLeftColor: color }}
      >
        <h2 className="text-sm font-semibold text-gray-700 mb-2">Articles</h2>

        {/* Search */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search articles..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="
              w-full pl-9 pr-3 py-2 text-sm border border-gray-300 rounded-lg
              focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent
            "
          />
        </div>
      </div>

      {/* Article List */}
      <div className="flex-1 overflow-y-auto">
        {filteredArticles.length === 0 ? (
          <div className="p-8 text-center text-sm text-gray-500">
            No articles found
          </div>
        ) : (
          filteredArticles.map(article => {
            const isActive = article.number === currentArticleId;
            const hasMachineReadable = !!article.machine_readable;

            return (
              <button
                key={article.number}
                onClick={() => selectArticle(article.number)}
                className={`
                  w-full text-left px-4 py-3 border-b border-gray-100
                  transition-all duration-150
                  ${isActive
                    ? 'bg-blue-50'
                    : 'hover:bg-gray-50'
                  }
                `}
                style={{
                  borderLeftWidth: isActive ? '4px' : '0px',
                  borderLeftColor: isActive ? color : 'transparent',
                }}
              >
                {/* Article Header */}
                <div className="flex items-center justify-between mb-1">
                  <span
                    className={`text-sm font-semibold ${isActive ? 'text-gray-900' : 'text-gray-700'}`}
                  >
                    Article {article.number}
                  </span>

                  {/* Machine-Readable Badge */}
                  {hasMachineReadable && (
                    <span
                      className="px-2 py-0.5 text-xs font-medium rounded-full"
                      style={{
                        backgroundColor: isActive ? color : '#E5E7EB',
                        color: isActive ? 'white' : '#6B7280',
                      }}
                    >
                      M-R
                    </span>
                  )}
                </div>

                {/* Article Text Preview */}
                <p className={`text-xs leading-relaxed line-clamp-2 ${isActive ? 'text-gray-700' : 'text-gray-600'}`}>
                  {article.text}
                </p>

                {/* Machine-Readable Indicator */}
                {hasMachineReadable && article.machine_readable?.execution && (
                  <div className="mt-2 text-xs text-gray-500">
                    {article.machine_readable.execution.actions
                      ? `${article.machine_readable.execution.actions.length} operations`
                      : article.machine_readable.execution.operation
                        ? article.machine_readable.execution.operation
                        : 'Has execution'}
                  </div>
                )}
              </button>
            );
          })
        )}
      </div>

      {/* Footer */}
      <div className="px-4 py-2 border-t border-gray-200 bg-gray-50">
        <p className="text-xs text-gray-500">
          {filteredArticles.length} of {currentLaw.articles.length} articles
        </p>
      </div>
    </div>
  );
}
