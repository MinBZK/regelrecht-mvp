/**
 * Middle Panel - YAML/Blockly toggle with beautiful transitions
 */

import { useState, useRef, useEffect } from 'react';
import { Code, Blocks, ArrowLeftRight, Eye } from 'lucide-react';
import * as Blockly from 'blockly';
import Editor from '@monaco-editor/react';
import { useEditorStore } from '../store/editorStore';
import { BLOCKLY_CONFIG, deserializeOperationToWorkspace } from '../utils/blockly';
import { validateYaml, serializeToYaml } from '../utils/yaml';

interface MiddlePanelProps {
  color?: string;
}

export default function MiddlePanel({ color = '#3B82F6' }: MiddlePanelProps) {
  const [mode, setMode] = useState<'yaml' | 'blockly'>('yaml');
  const {
    currentArticle,
    yamlCode,
    updateYamlCode,
    setBlocklyWorkspace,
    syncBlocklyToYaml,
    syncYamlToBlockly,
  } = useEditorStore();

  const blocklyDiv = useRef<HTMLDivElement>(null);
  const workspaceRef = useRef<Blockly.WorkspaceSvg | null>(null);
  const [validation, setValidation] = useState<{ valid: boolean; error?: string }>({ valid: true });

  // Initialize Blockly workspace
  useEffect(() => {
    if (mode !== 'blockly' || !blocklyDiv.current || !currentArticle) {
      return;
    }

    if (workspaceRef.current) {
      return; // Already initialized
    }

    try {
      const workspace = Blockly.inject(blocklyDiv.current, BLOCKLY_CONFIG);
      workspaceRef.current = workspace;
      setBlocklyWorkspace(workspace);

      // Load article data
      if (currentArticle.machine_readable?.execution) {
        const execution = currentArticle.machine_readable.execution;
        if (execution.actions && Array.isArray(execution.actions)) {
          const firstAction = execution.actions[0];
          if (firstAction?.operation) {
            deserializeOperationToWorkspace(firstAction, workspace);
          }
        } else if (execution.operation) {
          deserializeOperationToWorkspace(execution, workspace);
        }
      }
    } catch (error) {
      console.error('Failed to initialize Blockly:', error);
    }
  }, [mode, currentArticle, setBlocklyWorkspace]);

  // Validate YAML
  useEffect(() => {
    if (yamlCode) {
      const result = validateYaml(yamlCode);
      setValidation(result);
    } else {
      setValidation({ valid: true });
    }
  }, [yamlCode]);

  // Handle mode switch
  const handleModeSwitch = (newMode: 'yaml' | 'blockly') => {
    if (newMode === 'blockly' && mode === 'yaml') {
      // Switching to Blockly - sync YAML to blocks
      syncYamlToBlockly();
    } else if (newMode === 'yaml' && mode === 'blockly') {
      // Switching to YAML - sync blocks to YAML
      syncBlocklyToYaml();
    }
    setMode(newMode);
  };

  if (!currentArticle) {
    return (
      <div className="flex flex-col items-center justify-center h-full bg-white">
        <Eye className="w-16 h-16 text-gray-300 mb-4" />
        <p className="text-sm text-gray-500">Select an article to view</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-white">
      {/* Header with Mode Toggle */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200">
        <div className="flex items-center gap-3">
          <h2 className="text-sm font-semibold text-gray-700">
            Article {currentArticle.number}
          </h2>

          {/* Validation Badge */}
          {mode === 'yaml' && (
            <span className={`
              text-xs px-2 py-1 rounded-full
              ${validation.valid
                ? 'bg-green-100 text-green-700'
                : 'bg-red-100 text-red-700'
              }
            `}>
              {validation.valid ? '✓ Valid' : '✗ Invalid'}
            </span>
          )}
        </div>

        {/* Mode Toggle */}
        <div className="flex items-center gap-2">
          <button
            onClick={() => handleModeSwitch('yaml')}
            className={`
              flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium
              transition-all duration-200
              ${mode === 'yaml'
                ? 'text-white shadow-md'
                : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
              }
            `}
            style={{
              backgroundColor: mode === 'yaml' ? color : undefined,
            }}
          >
            <Code className="w-4 h-4" />
            YAML
          </button>

          <button
            onClick={() => handleModeSwitch('blockly')}
            className={`
              flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium
              transition-all duration-200
              ${mode === 'blockly'
                ? 'text-white shadow-md'
                : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
              }
            `}
            style={{
              backgroundColor: mode === 'blockly' ? color : undefined,
            }}
          >
            <Blocks className="w-4 h-4" />
            Blocks
          </button>

          {/* Sync Button */}
          <button
            onClick={() => mode === 'yaml' ? syncYamlToBlockly() : syncBlocklyToYaml()}
            className="
              p-2 rounded-lg bg-gray-100 text-gray-600 hover:bg-gray-200
              transition-colors
            "
            title="Sync between YAML and Blockly"
          >
            <ArrowLeftRight className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Content Area */}
      <div className="flex-1 overflow-hidden relative">
        {/* YAML Editor */}
        {mode === 'yaml' && (
          <div className="h-full">
            <Editor
              height="100%"
              defaultLanguage="yaml"
              value={yamlCode}
              onChange={(value) => updateYamlCode(value || '')}
              theme="vs-light"
              options={{
                fontSize: 14,
                fontFamily: '"Fira Code", monospace',
                fontLigatures: true,
                minimap: { enabled: false },
                lineNumbers: 'on',
                scrollBeyondLastLine: false,
                wordWrap: 'on',
                automaticLayout: true,
                tabSize: 2,
                insertSpaces: true,
                folding: true,
              }}
            />
          </div>
        )}

        {/* Blockly Workspace */}
        {mode === 'blockly' && (
          <div ref={blocklyDiv} className="h-full bg-gray-50" />
        )}
      </div>

      {/* Footer */}
      <div className="px-4 py-2 border-t border-gray-200 bg-gray-50">
        {mode === 'yaml' && !validation.valid && validation.error && (
          <p className="text-xs text-red-600">{validation.error}</p>
        )}
        {mode === 'yaml' && validation.valid && (
          <p className="text-xs text-gray-500">
            {yamlCode.split('\n').length} lines • {yamlCode.length} characters
          </p>
        )}
        {mode === 'blockly' && (
          <p className="text-xs text-gray-500">
            Drag blocks from the toolbox to build logic
          </p>
        )}
      </div>
    </div>
  );
}
