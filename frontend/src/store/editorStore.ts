/**
 * Zustand store for editor state management
 * Handles law selection, article selection, and sync between Blockly and YAML
 */

import { create } from 'zustand';
import type { Law, Article, LawSummary } from '../types/schema';
import { getAllLaws, getLawByUuid } from '../services/api';
import { serializeToYaml, deserializeFromYaml } from '../utils/yaml';
import { serializeWorkspaceToOperation, deserializeOperationToWorkspace } from '../utils/blockly';

interface EditorStore {
  // Available laws
  availableLaws: LawSummary[];

  // Current law and article
  currentLaw: Law | null;
  currentArticle: Article | null;
  currentArticleId: string | null;

  // UI state
  isLoading: boolean;
  error: string | null;

  // Blockly workspace (set by Blockly component)
  blocklyWorkspace: any | null;

  // YAML code for preview
  yamlCode: string;

  // Sync state
  isSyncing: boolean;
  isDirty: boolean; // Has unsaved changes

  // Actions
  loadAvailableLaws: () => Promise<void>;
  selectLaw: (uuid: string) => Promise<void>;
  selectArticle: (articleId: string) => void;
  setBlocklyWorkspace: (workspace: any) => void;
  updateYamlCode: (code: string) => void;
  syncBlocklyToYaml: () => void;
  syncYamlToBlockly: () => void;
  setError: (error: string | null) => void;
  clearError: () => void;
  reset: () => void;
}

export const useEditorStore = create<EditorStore>((set, get) => ({
  // Initial state
  availableLaws: [],
  currentLaw: null,
  currentArticle: null,
  currentArticleId: null,
  isLoading: false,
  error: null,
  blocklyWorkspace: null,
  yamlCode: '',
  isSyncing: false,
  isDirty: false,

  /**
   * Load all available laws from the backend
   */
  loadAvailableLaws: async () => {
    set({ isLoading: true, error: null });
    try {
      const laws = await getAllLaws();
      set({ availableLaws: laws, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to load laws';
      set({ error: message, isLoading: false });
    }
  },

  /**
   * Select and load a law by UUID
   */
  selectLaw: async (uuid: string) => {
    set({ isLoading: true, error: null });
    try {
      const law = await getLawByUuid(uuid);
      set({
        currentLaw: law,
        currentArticle: null,
        currentArticleId: null,
        yamlCode: '',
        isDirty: false,
        isLoading: false,
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to load law';
      set({ error: message, isLoading: false });
    }
  },

  /**
   * Select an article from the current law
   */
  selectArticle: (articleId: string) => {
    const { currentLaw } = get();
    if (!currentLaw) return;

    const article = currentLaw.articles.find((a) => a.number === articleId);
    if (!article) {
      set({ error: `Article ${articleId} not found` });
      return;
    }

    // Generate YAML code for the article's machine-readable content
    let yamlCode = '';
    if (article.machine_readable) {
      yamlCode = generateYamlFromMachineReadable(article.machine_readable);
    }

    set({
      currentArticle: article,
      currentArticleId: articleId,
      yamlCode,
      isDirty: false,
      error: null,
    });
  },

  /**
   * Set the Blockly workspace instance
   */
  setBlocklyWorkspace: (workspace: any) => {
    set({ blocklyWorkspace: workspace });
  },

  /**
   * Update the YAML code (from Monaco editor or sync)
   */
  updateYamlCode: (code: string) => {
    set({ yamlCode: code, isDirty: true });
  },

  /**
   * Sync from Blockly workspace to YAML
   */
  syncBlocklyToYaml: () => {
    const { blocklyWorkspace } = get();
    if (!blocklyWorkspace) {
      set({ error: 'Blockly workspace not initialized' });
      return;
    }

    set({ isSyncing: true });
    try {
      // TODO: Implement Blockly to YAML serialization
      // This will traverse the Blockly blocks and generate YAML
      const yamlCode = serializeBlocklyToYaml(blocklyWorkspace);
      set({ yamlCode, isSyncing: false, isDirty: true });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Sync failed';
      set({ error: message, isSyncing: false });
    }
  },

  /**
   * Sync from YAML to Blockly workspace
   */
  syncYamlToBlockly: () => {
    const { blocklyWorkspace, yamlCode } = get();
    if (!blocklyWorkspace) {
      set({ error: 'Blockly workspace not initialized' });
      return;
    }

    set({ isSyncing: true });
    try {
      // TODO: Implement YAML to Blockly deserialization
      // This will parse YAML and create Blockly blocks
      deserializeYamlToBlockly(yamlCode, blocklyWorkspace);
      set({ isSyncing: false, isDirty: true });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Sync failed';
      set({ error: message, isSyncing: false });
    }
  },

  /**
   * Set error message
   */
  setError: (error: string | null) => {
    set({ error });
  },

  /**
   * Clear error message
   */
  clearError: () => {
    set({ error: null });
  },

  /**
   * Reset the store to initial state
   */
  reset: () => {
    set({
      currentLaw: null,
      currentArticle: null,
      currentArticleId: null,
      yamlCode: '',
      isDirty: false,
      error: null,
    });
  },
}));

/**
 * Helper function to generate YAML from machine-readable object
 */
function generateYamlFromMachineReadable(machineReadable: any): string {
  try {
    return serializeToYaml(machineReadable);
  } catch (error) {
    console.error('Failed to serialize machine-readable to YAML:', error);
    return JSON.stringify(machineReadable, null, 2);
  }
}

/**
 * Helper function to serialize Blockly workspace to YAML
 */
function serializeBlocklyToYaml(workspace: any): string {
  try {
    const operation = serializeWorkspaceToOperation(workspace);
    if (!operation) {
      return '# No blocks in workspace';
    }
    return serializeToYaml({ execution: operation });
  } catch (error) {
    console.error('Failed to serialize Blockly to YAML:', error);
    throw new Error('Failed to convert blocks to YAML');
  }
}

/**
 * Helper function to deserialize YAML to Blockly blocks
 */
function deserializeYamlToBlockly(yamlCode: string, workspace: any): void {
  try {
    const parsed = deserializeFromYaml(yamlCode);

    // Extract execution block
    const execution = parsed.execution || parsed;

    if (!execution || !execution.operation) {
      throw new Error('No execution block found in YAML');
    }

    deserializeOperationToWorkspace(execution, workspace);
  } catch (error) {
    console.error('Failed to deserialize YAML to Blockly:', error);
    throw new Error('Failed to convert YAML to blocks');
  }
}
