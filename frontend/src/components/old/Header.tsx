/**
 * Header component with law selector and branding
 */

import { useEditorStore } from '../store/editorStore';
import { Scale } from 'lucide-react';

export default function Header() {
  const {
    availableLaws,
    currentLaw,
    selectLaw,
    isLoading,
  } = useEditorStore();

  const handleLawChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const uuid = e.target.value;
    if (uuid) {
      selectLaw(uuid);
    }
  };

  return (
    <header className="bg-gradient-to-r from-legal-blue to-blue-700 text-white shadow-lg">
      <div className="px-6 py-4">
        <div className="flex items-center justify-between">
          {/* Branding */}
          <div className="flex items-center gap-3">
            <Scale className="w-8 h-8" />
            <div>
              <h1 className="text-2xl font-bold font-serif">RegelRecht</h1>
              <p className="text-sm text-blue-100">Visual Legal Logic Editor</p>
            </div>
          </div>

          {/* Law Selector */}
          <div className="flex items-center gap-4">
            <label htmlFor="law-select" className="text-sm font-medium">
              Select Law:
            </label>
            <select
              id="law-select"
              value={currentLaw?.uuid || ''}
              onChange={handleLawChange}
              disabled={isLoading}
              className="px-4 py-2 bg-white text-gray-900 border border-gray-300 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed min-w-[300px]"
            >
              <option value="">Choose a law or regulation...</option>
              {availableLaws.map((law) => (
                <option key={law.uuid} value={law.uuid}>
                  {law.short_name} ({law.article_count} articles)
                </option>
              ))}
            </select>

            {isLoading && (
              <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-white"></div>
            )}
          </div>
        </div>

        {/* Current Law Info */}
        {currentLaw && (
          <div className="mt-3 pt-3 border-t border-blue-600">
            <div className="flex items-center gap-6 text-sm">
              <span className="text-blue-100">
                <strong>Type:</strong> {currentLaw.regulatory_layer}
              </span>
              {currentLaw.bwb_id && (
                <span className="text-blue-100">
                  <strong>BWB ID:</strong> {currentLaw.bwb_id}
                </span>
              )}
              <span className="text-blue-100">
                <strong>Articles:</strong> {currentLaw.articles.length}
              </span>
              {currentLaw.publication_date && (
                <span className="text-blue-100">
                  <strong>Published:</strong> {currentLaw.publication_date}
                </span>
              )}
            </div>
          </div>
        )}
      </div>
    </header>
  );
}
