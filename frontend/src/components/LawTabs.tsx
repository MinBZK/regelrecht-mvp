/**
 * Law Tabs - Beautiful tabbed interface for multiple open laws
 */

import { X, Plus, ChevronDown } from 'lucide-react';
import { useState } from 'react';
import type { LawSummary } from '../types/schema';

interface LawTabsProps {
  openLaws: string[];
  availableLaws: LawSummary[];
  activeLawId: string | null;
  onTabClick: (uuid: string) => void;
  onCloseLaw: (uuid: string, e: React.MouseEvent) => void;
  onAddLaw: (uuid: string) => void;
  getLawColor: (uuid: string) => string;
}

export default function LawTabs({
  openLaws,
  availableLaws,
  activeLawId,
  onTabClick,
  onCloseLaw,
  onAddLaw,
  getLawColor,
}: LawTabsProps) {
  const [showLawSelector, setShowLawSelector] = useState(false);

  const getShortName = (law: LawSummary | undefined) => {
    if (!law) return '?';
    // Extract acronym from short_name or take first letters
    const words = law.short_name.split(' ');
    if (words.length === 1) {
      return law.short_name.substring(0, 3).toUpperCase();
    }
    return words.map(w => w[0]).join('').toUpperCase();
  };

  return (
    <div className="bg-white border-b border-gray-300 shadow-sm">
      <div className="flex items-center h-12 px-2 gap-1 overflow-x-auto">
        {/* Open Law Tabs */}
        {openLaws.map(uuid => {
          const law = availableLaws.find(l => l.uuid === uuid);
          const isActive = uuid === activeLawId;
          const color = getLawColor(uuid);

          return (
            <button
              key={uuid}
              onClick={() => onTabClick(uuid)}
              className={`
                group relative flex items-center gap-2 px-4 py-2 rounded-t-lg
                transition-all duration-200 min-w-0
                ${isActive
                  ? 'bg-white shadow-md z-10'
                  : 'bg-gray-100 hover:bg-gray-200'
                }
              `}
              style={{
                borderBottom: isActive ? `3px solid ${color}` : 'none',
              }}
            >
              {/* Law Color Indicator */}
              <div
                className="w-2 h-2 rounded-full flex-shrink-0"
                style={{ backgroundColor: color }}
              />

              {/* Law Short Name */}
              <span
                className={`
                  text-sm font-medium truncate max-w-[200px]
                  ${isActive ? 'text-gray-900' : 'text-gray-600'}
                `}
                title={law?.short_name}
              >
                {law?.short_name || 'Loading...'}
              </span>

              {/* Article Count Badge */}
              <span
                className={`
                  text-xs px-1.5 py-0.5 rounded-full flex-shrink-0
                  ${isActive
                    ? 'bg-gray-100 text-gray-600'
                    : 'bg-gray-200 text-gray-500'
                  }
                `}
              >
                {law?.article_count || 0}
              </span>

              {/* Close Button */}
              {openLaws.length > 1 && (
                <button
                  onClick={(e) => onCloseLaw(uuid, e)}
                  className={`
                    ml-1 p-0.5 rounded hover:bg-gray-300
                    transition-colors flex-shrink-0
                    ${isActive ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'}
                  `}
                >
                  <X className="w-3 h-3 text-gray-500" />
                </button>
              )}
            </button>
          );
        })}

        {/* Add Law Button */}
        <div className="relative">
          <button
            onClick={() => setShowLawSelector(!showLawSelector)}
            className="
              flex items-center gap-1 px-3 py-2 text-sm text-gray-600
              hover:bg-gray-100 rounded-lg transition-colors
            "
          >
            <Plus className="w-4 h-4" />
            <span className="hidden sm:inline">Open Law</span>
            <ChevronDown className="w-3 h-3" />
          </button>

          {/* Law Selector Dropdown */}
          {showLawSelector && (
            <>
              {/* Backdrop */}
              <div
                className="fixed inset-0 z-20"
                onClick={() => setShowLawSelector(false)}
              />

              {/* Dropdown Menu */}
              <div className="
                absolute top-full left-0 mt-1 w-64 bg-white border
                border-gray-200 rounded-lg shadow-lg z-30 py-1
              ">
                {availableLaws.map(law => {
                  const isOpen = openLaws.includes(law.uuid);
                  const color = getLawColor(law.uuid);

                  return (
                    <button
                      key={law.uuid}
                      onClick={() => {
                        onAddLaw(law.uuid);
                        setShowLawSelector(false);
                      }}
                      className={`
                        w-full flex items-center gap-2 px-3 py-2 text-left
                        hover:bg-gray-50 transition-colors
                        ${isOpen ? 'bg-gray-50' : ''}
                      `}
                    >
                      <div
                        className="w-2 h-2 rounded-full flex-shrink-0"
                        style={{ backgroundColor: color }}
                      />
                      <div className="flex-1 min-w-0">
                        <div className="text-sm font-medium text-gray-900 truncate">
                          {law.short_name}
                        </div>
                        <div className="text-xs text-gray-500">
                          {law.article_count} articles
                        </div>
                      </div>
                      {isOpen && (
                        <span className="text-xs text-gray-400">Open</span>
                      )}
                    </button>
                  );
                })}
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
