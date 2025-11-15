/**
 * Sync Panel - Right panel showing sync proposals and article info
 */

import { useState } from 'react';
import { useEditorStore } from '../store/editorStore';
import { BookOpen, Code, AlertCircle, CheckCircle, X } from 'lucide-react';

type TabType = 'info' | 'proposals' | 'references';

export default function SyncPanel() {
  const { currentArticle, currentLaw } = useEditorStore();
  const [activeTab, setActiveTab] = useState<TabType>('info');

  if (!currentArticle) {
    return (
      <div className="flex flex-col items-center justify-center h-full p-8 text-center">
        <AlertCircle className="w-16 h-16 text-gray-300 mb-4" />
        <p className="text-sm text-gray-500">Select an article to see details</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Tabs */}
      <div className="flex border-b border-gray-200 bg-white">
        <button
          onClick={() => setActiveTab('info')}
          className={`
            flex-1 px-4 py-3 text-sm font-medium transition-colors
            ${activeTab === 'info'
              ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
              : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
            }
          `}
        >
          Info
        </button>
        <button
          onClick={() => setActiveTab('proposals')}
          className={`
            flex-1 px-4 py-3 text-sm font-medium transition-colors
            ${activeTab === 'proposals'
              ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
              : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
            }
          `}
        >
          Sync
        </button>
        <button
          onClick={() => setActiveTab('references')}
          className={`
            flex-1 px-4 py-3 text-sm font-medium transition-colors
            ${activeTab === 'references'
              ? 'text-blue-600 border-b-2 border-blue-600 bg-blue-50'
              : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'
            }
          `}
        >
          Links
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4">
        {activeTab === 'info' && (
          <div className="space-y-4">
            {/* Article Text */}
            <div>
              <h3 className="text-xs font-semibold text-gray-500 uppercase mb-2">
                Legal Text
              </h3>
              <div className="p-3 bg-gray-50 rounded-lg border border-gray-200">
                <p className="text-sm text-gray-700 leading-relaxed font-serif">
                  {currentArticle.text}
                </p>
              </div>
            </div>

            {/* Article Metadata */}
            <div>
              <h3 className="text-xs font-semibold text-gray-500 uppercase mb-2">
                Metadata
              </h3>
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600">Article</span>
                  <span className="font-medium text-gray-900">{currentArticle.number}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600">Law</span>
                  <span className="font-medium text-gray-900">{currentLaw?.short_name}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600">Machine-Readable</span>
                  <span className={`font-medium ${currentArticle.machine_readable ? 'text-green-600' : 'text-gray-400'}`}>
                    {currentArticle.machine_readable ? 'Yes' : 'No'}
                  </span>
                </div>
                {currentArticle.url && (
                  <div className="pt-2">
                    <a
                      href={currentArticle.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-sm text-blue-600 hover:underline flex items-center gap-1"
                    >
                      <BookOpen className="w-4 h-4" />
                      View on wetten.nl
                    </a>
                  </div>
                )}
              </div>
            </div>

            {/* Machine-Readable Info */}
            {currentArticle.machine_readable && (
              <div>
                <h3 className="text-xs font-semibold text-gray-500 uppercase mb-2">
                  Execution Details
                </h3>
                <div className="space-y-2 text-sm">
                  {currentArticle.machine_readable.public !== undefined && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">Public API</span>
                      <span className="font-medium text-gray-900">
                        {currentArticle.machine_readable.public ? 'Yes' : 'No'}
                      </span>
                    </div>
                  )}
                  {currentArticle.machine_readable.endpoint && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">Endpoint</span>
                      <code className="text-xs bg-gray-100 px-2 py-1 rounded">
                        {currentArticle.machine_readable.endpoint}
                      </code>
                    </div>
                  )}
                  {currentArticle.machine_readable.competent_authority && (
                    <div className="flex justify-between">
                      <span className="text-gray-600">Authority</span>
                      <span className="font-medium text-gray-900">
                        {currentArticle.machine_readable.competent_authority}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        )}

        {activeTab === 'proposals' && (
          <div className="space-y-3">
            <div className="flex items-center gap-2 p-3 bg-green-50 border border-green-200 rounded-lg">
              <CheckCircle className="w-5 h-5 text-green-600 flex-shrink-0" />
              <div className="text-sm">
                <p className="font-medium text-green-900">YAML and Blockly in sync</p>
                <p className="text-green-700 text-xs mt-1">No sync proposals at this time</p>
              </div>
            </div>

            <div className="text-xs text-gray-500 text-center py-8">
              Sync proposals will appear here when you make changes to either YAML or Blockly
            </div>
          </div>
        )}

        {activeTab === 'references' && (
          <div className="space-y-3">
            {currentArticle.machine_readable?.requires && (
              <div>
                <h3 className="text-xs font-semibold text-gray-500 uppercase mb-2">
                  Required Articles
                </h3>
                <div className="space-y-2">
                  {currentArticle.machine_readable.requires.map((req: any, i: number) => (
                    <div key={i} className="p-2 bg-gray-50 rounded border border-gray-200">
                      <code className="text-xs text-gray-700">
                        {JSON.stringify(req, null, 2)}
                      </code>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {(!currentArticle.machine_readable?.requires || currentArticle.machine_readable.requires.length === 0) && (
              <div className="text-xs text-gray-500 text-center py-8">
                No references found in this article
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
