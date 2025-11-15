/**
 * Right Panel: YAML code preview with Monaco editor
 */

import { useEffect, useState } from 'react';
import Editor from '@monaco-editor/react';
import { useEditorStore } from '../store/editorStore';
import { validateYaml } from '../utils/yaml';
import { Code, CheckCircle, XCircle } from 'lucide-react';

export default function RightPanel() {
  const {
    currentArticle,
    yamlCode,
    updateYamlCode,
    isDirty,
  } = useEditorStore();

  const [localYaml, setLocalYaml] = useState('');
  const [validation, setValidation] = useState<{ valid: boolean; error?: string }>({ valid: true });

  // Update local YAML when store changes
  useEffect(() => {
    setLocalYaml(yamlCode);
  }, [yamlCode]);

  // Validate YAML as user types
  useEffect(() => {
    if (localYaml) {
      const result = validateYaml(localYaml);
      setValidation(result);
    } else {
      setValidation({ valid: true });
    }
  }, [localYaml]);

  const handleEditorChange = (value: string | undefined) => {
    const newValue = value || '';
    setLocalYaml(newValue);
    updateYamlCode(newValue);
  };

  if (!currentArticle) {
    return (
      <div className="flex flex-col items-center justify-center h-full p-8 text-center bg-gray-900">
        <Code className="w-16 h-16 text-gray-600 mb-4" />
        <h2 className="text-xl font-semibold text-gray-300 mb-2">
          No Article Selected
        </h2>
        <p className="text-sm text-gray-500">
          YAML code will appear here when you select an article.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Panel Header */}
      <div className="bg-gray-800 border-b border-gray-700 px-4 py-3">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-white flex items-center gap-2">
              <Code className="w-5 h-5 text-legal-purple" />
              YAML Preview
            </h2>
            <p className="text-xs text-gray-400 mt-1 font-mono">
              Article {currentArticle.number} • machine_readable
            </p>
          </div>

          {/* Validation Status */}
          <div className="flex items-center gap-2">
            {validation.valid ? (
              <div className="flex items-center gap-1 text-legal-green">
                <CheckCircle className="w-4 h-4" />
                <span className="text-xs">Valid YAML</span>
              </div>
            ) : (
              <div className="flex items-center gap-1 text-legal-red">
                <XCircle className="w-4 h-4" />
                <span className="text-xs">Invalid YAML</span>
              </div>
            )}
            {isDirty && (
              <span className="px-2 py-0.5 bg-legal-amber text-white text-xs rounded">
                Modified
              </span>
            )}
          </div>
        </div>

        {/* Validation Error */}
        {!validation.valid && validation.error && (
          <div className="mt-2 p-2 bg-red-900/30 border border-red-700 rounded">
            <p className="text-xs text-red-300 font-mono">{validation.error}</p>
          </div>
        )}
      </div>

      {/* Monaco Editor */}
      <div className="flex-1 overflow-hidden">
        <Editor
          height="100%"
          defaultLanguage="yaml"
          value={localYaml}
          onChange={handleEditorChange}
          theme="vs-dark"
          options={{
            fontSize: 13,
            fontFamily: '"Fira Code", Menlo, Monaco, monospace',
            fontLigatures: true,
            minimap: { enabled: true },
            lineNumbers: 'on',
            scrollBeyondLastLine: false,
            wordWrap: 'on',
            wrappingIndent: 'indent',
            automaticLayout: true,
            tabSize: 2,
            insertSpaces: true,
            folding: true,
            bracketPairColorization: {
              enabled: true,
            },
          }}
        />
      </div>

      {/* Panel Footer */}
      <div className="bg-gray-800 border-t border-gray-700 px-4 py-2">
        <div className="flex items-center justify-between text-xs text-gray-400">
          <span className="font-mono">
            {localYaml.split('\n').length} lines • {localYaml.length} chars
          </span>
          <span>
            Edit YAML directly or use the visual editor
          </span>
        </div>
      </div>
    </div>
  );
}
