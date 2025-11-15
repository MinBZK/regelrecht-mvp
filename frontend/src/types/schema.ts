/**
 * TypeScript interfaces matching the Dutch Law Schema v0.2.0
 * and the backend Pydantic models
 */

export type RegulatoryLayer = 'wet' | 'amvb' | 'ministeriele_regeling';

export interface LawSummary {
  uuid: string;
  short_name: string;
  regulatory_layer: RegulatoryLayer;
  article_count: number;
  bwb_id?: string;
}

export interface MachineReadable {
  public?: boolean;
  endpoint?: string;
  competent_authority?: string;
  requires?: Array<Record<string, any>>;
  definitions?: Record<string, any>;
  execution?: Record<string, any>;
  [key: string]: any; // Allow extra fields
}

export interface Article {
  number: string;
  text: string;
  url: string;
  machine_readable?: MachineReadable;
}

export interface Law {
  $schema?: string;
  $id?: string;
  uuid: string;
  publication_date: string;
  regulatory_layer: RegulatoryLayer;
  url: string;
  short_name?: string;
  full_name?: string;
  abbreviation?: string;
  bwb_id?: string;
  articles: Article[];
  [key: string]: any; // Allow extra fields
}

export interface ArticleWithId {
  id: string;
  article: Article;
}

/**
 * Blockly-specific types for visual programming
 */

export type OperationType =
  | 'ADD'
  | 'SUBTRACT'
  | 'MULTIPLY'
  | 'DIVIDE'
  | 'MODULO'
  | 'MAX'
  | 'MIN'
  | 'ROUND'
  | 'FLOOR'
  | 'CEILING'
  | 'IF'
  | 'LOOKUP'
  | 'TABLE_LOOKUP'
  | 'GET'
  | 'EQUALS'
  | 'NOT_EQUALS'
  | 'GREATER_THAN'
  | 'LESS_THAN'
  | 'GREATER_THAN_OR_EQUALS'
  | 'LESS_THAN_OR_EQUALS'
  | 'AND'
  | 'OR'
  | 'NOT';

export interface Operation {
  operation: OperationType;
  values?: any[];
  value?: any;
  condition?: Operation;
  then?: Operation;
  else?: Operation;
  table?: Record<string, any>;
  key?: string;
  default?: any;
  [key: string]: any; // Flexible for nested structures
}

/**
 * Editor state types
 */

export interface EditorState {
  // Current law being edited
  currentLaw: Law | null;
  currentArticle: Article | null;

  // UI state
  selectedArticleId: string | null;
  isLoading: boolean;
  error: string | null;

  // Blockly workspace
  blocklyWorkspace: any; // Blockly.WorkspaceSvg type

  // Sync state
  yamlCode: string;
  isSyncing: boolean;
}

/**
 * API response types
 */

export interface ApiError {
  detail: string;
}

export interface HealthResponse {
  status: string;
  laws_loaded: number;
}
