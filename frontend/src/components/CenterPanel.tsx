/**
 * Center Panel: Blockly visual programming workspace
 */

import { useEffect, useRef } from 'react';
import * as Blockly from 'blockly';
import { useEditorStore } from '../store/editorStore';
import { BLOCKLY_CONFIG, deserializeOperationToWorkspace } from '../utils/blockly';
import { extractExecution } from '../utils/yaml';
import { Blocks, ArrowLeftRight } from 'lucide-react';

export default function CenterPanel() {
  const blocklyDiv = useRef<HTMLDivElement>(null);
  const workspaceRef = useRef<Blockly.WorkspaceSvg | null>(null);

  const {
    currentArticle,
    setBlocklyWorkspace,
    syncBlocklyToYaml,
    syncYamlToBlockly,
  } = useEditorStore();

  // Initialize Blockly workspace
  useEffect(() => {
    if (!blocklyDiv.current) return;

    // Create workspace
    const workspace = Blockly.inject(blocklyDiv.current, BLOCKLY_CONFIG);
    workspaceRef.current = workspace;
    setBlocklyWorkspace(workspace);

    // Listen to workspace changes
    workspace.addChangeListener((event) => {
      // Auto-sync to YAML on block changes (with debouncing in production)
      if (event.type === Blockly.Events.BLOCK_CHANGE ||
          event.type === Blockly.Events.BLOCK_CREATE ||
          event.type === Blockly.Events.BLOCK_DELETE ||
          event.type === Blockly.Events.BLOCK_MOVE) {
        // Trigger sync after changes
        // In production, this should be debounced
      }
    });

    // Cleanup
    return () => {
      workspace.dispose();
    };
  }, [setBlocklyWorkspace]);

  // Load article's machine-readable logic when article changes
  useEffect(() => {
    if (!workspaceRef.current || !currentArticle) {
      // Clear workspace if no article selected
      if (workspaceRef.current) {
        workspaceRef.current.clear();
      }
      return;
    }

    // Load machine-readable execution into Blockly
    const machineReadable = currentArticle.machine_readable;
    if (machineReadable) {
      const execution = extractExecution(machineReadable);
      if (execution) {
        try {
          deserializeOperationToWorkspace(execution, workspaceRef.current);
        } catch (error) {
          console.error('Failed to load execution into Blockly:', error);
        }
      } else {
        workspaceRef.current.clear();
      }
    } else {
      workspaceRef.current.clear();
    }
  }, [currentArticle]);

  if (!currentArticle) {
    return (
      <div className="flex flex-col items-center justify-center h-full p-8 text-center bg-gray-50">
        <Blocks className="w-16 h-16 text-gray-400 mb-4" />
        <h2 className="text-xl font-semibold text-gray-700 mb-2">
          No Article Selected
        </h2>
        <p className="text-sm text-gray-600">
          Select an article from the left panel to begin editing its logic.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Panel Header */}
      <div className="bg-white border-b border-gray-300 px-4 py-3">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-gray-800 flex items-center gap-2">
              <Blocks className="w-5 h-5 text-legal-blue" />
              Visual Logic Editor
            </h2>
            <p className="text-xs text-gray-600 mt-1">
              Article {currentArticle.number}
            </p>
          </div>

          {/* Sync Controls */}
          <div className="flex items-center gap-2">
            <button
              onClick={syncYamlToBlockly}
              className="px-3 py-1.5 bg-legal-amber text-white text-sm rounded hover:bg-amber-600 transition-colors flex items-center gap-1"
              title="Load YAML into Blockly (overwrites current blocks)"
            >
              <ArrowLeftRight className="w-4 h-4" />
              YAML → Blocks
            </button>
            <button
              onClick={syncBlocklyToYaml}
              className="px-3 py-1.5 bg-legal-green text-white text-sm rounded hover:bg-green-700 transition-colors flex items-center gap-1"
              title="Generate YAML from Blockly blocks"
            >
              <ArrowLeftRight className="w-4 h-4" />
              Blocks → YAML
            </button>
          </div>
        </div>
      </div>

      {/* Blockly Workspace */}
      <div ref={blocklyDiv} className="flex-1" />

      {/* Panel Footer with hints */}
      <div className="bg-gray-50 border-t border-gray-300 px-4 py-2">
        <p className="text-xs text-gray-600">
          <strong>Tip:</strong> Drag blocks from the toolbox on the left. Connect them to build logic expressions.
          Use the sync buttons above to convert between blocks and YAML code.
        </p>
      </div>
    </div>
  );
}
