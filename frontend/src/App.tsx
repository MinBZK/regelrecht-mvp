/**
 * Main App component with three-panel layout:
 * - Left: Articles list with traditional legal styling
 * - Center: Blockly visual editor
 * - Right: YAML code preview with Monaco
 */

import { useEffect } from 'react';
import { useEditorStore } from './store/editorStore';
import { initializeBlocklyBlocks } from './utils/blockly';
import Header from './components/Header';
import LeftPanel from './components/LeftPanel';
import CenterPanel from './components/CenterPanel';
import RightPanel from './components/RightPanel';

function App() {
  const { loadAvailableLaws, error, clearError } = useEditorStore();

  // Initialize on mount
  useEffect(() => {
    // Initialize Blockly custom blocks
    initializeBlocklyBlocks();

    // Load available laws from backend
    loadAvailableLaws();
  }, [loadAvailableLaws]);

  return (
    <div className="flex flex-col h-screen bg-gray-50">
      {/* Header with law selector */}
      <Header />

      {/* Error notification */}
      {error && (
        <div className="bg-red-50 border-l-4 border-red-500 p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center">
              <svg
                className="h-5 w-5 text-red-500 mr-2"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fillRule="evenodd"
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                  clipRule="evenodd"
                />
              </svg>
              <p className="text-sm text-red-700">{error}</p>
            </div>
            <button
              onClick={clearError}
              className="text-red-500 hover:text-red-700"
            >
              <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
                <path
                  fillRule="evenodd"
                  d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                  clipRule="evenodd"
                />
              </svg>
            </button>
          </div>
        </div>
      )}

      {/* Three-panel layout */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left Panel: Articles */}
        <div className="w-1/4 min-w-[250px] max-w-[400px] border-r border-gray-300 overflow-y-auto bg-parchment-50">
          <LeftPanel />
        </div>

        {/* Center Panel: Blockly Editor */}
        <div className="flex-1 min-w-[400px] bg-white">
          <CenterPanel />
        </div>

        {/* Right Panel: YAML Preview */}
        <div className="w-1/3 min-w-[300px] max-w-[500px] border-l border-gray-300 bg-gray-900">
          <RightPanel />
        </div>
      </div>
    </div>
  );
}

export default App;
