/**
 * Blockly utility functions
 * Handles workspace initialization, block definitions, and serialization
 */

import * as Blockly from 'blockly';
import type { Operation, OperationType } from '../types/schema';

/**
 * Blockly workspace configuration
 */
export const BLOCKLY_CONFIG = {
  toolbox: {
    kind: 'categoryToolbox',
    contents: [
      {
        kind: 'category',
        name: 'Arithmetic',
        colour: '#5C81A6',
        contents: [
          { kind: 'block', type: 'operation_add' },
          { kind: 'block', type: 'operation_subtract' },
          { kind: 'block', type: 'operation_multiply' },
          { kind: 'block', type: 'operation_divide' },
          { kind: 'block', type: 'operation_modulo' },
        ],
      },
      {
        kind: 'category',
        name: 'Math Functions',
        colour: '#5CA65C',
        contents: [
          { kind: 'block', type: 'operation_max' },
          { kind: 'block', type: 'operation_min' },
          { kind: 'block', type: 'operation_round' },
          { kind: 'block', type: 'operation_floor' },
          { kind: 'block', type: 'operation_ceiling' },
        ],
      },
      {
        kind: 'category',
        name: 'Logic',
        colour: '#5C68A6',
        contents: [
          { kind: 'block', type: 'operation_if' },
          { kind: 'block', type: 'operation_and' },
          { kind: 'block', type: 'operation_or' },
          { kind: 'block', type: 'operation_not' },
        ],
      },
      {
        kind: 'category',
        name: 'Comparison',
        colour: '#A65C81',
        contents: [
          { kind: 'block', type: 'operation_equals' },
          { kind: 'block', type: 'operation_not_equals' },
          { kind: 'block', type: 'operation_greater_than' },
          { kind: 'block', type: 'operation_less_than' },
          { kind: 'block', type: 'operation_gte' },
          { kind: 'block', type: 'operation_lte' },
        ],
      },
      {
        kind: 'category',
        name: 'Data Access',
        colour: '#A68B5C',
        contents: [
          { kind: 'block', type: 'operation_get' },
          { kind: 'block', type: 'operation_lookup' },
          { kind: 'block', type: 'operation_table_lookup' },
        ],
      },
      {
        kind: 'category',
        name: 'Values',
        colour: '#745CA6',
        contents: [
          { kind: 'block', type: 'value_number' },
          { kind: 'block', type: 'value_text' },
          { kind: 'block', type: 'value_boolean' },
        ],
      },
    ],
  },
  grid: {
    spacing: 20,
    length: 3,
    colour: '#ccc',
    snap: true,
  },
  zoom: {
    controls: true,
    wheel: true,
    startScale: 1.0,
    maxScale: 3,
    minScale: 0.3,
    scaleSpeed: 1.2,
  },
  trashcan: true,
};

/**
 * Initialize custom Blockly blocks for all operation types
 * This will be called once when the app starts
 */
export function initializeBlocklyBlocks(): void {
  // Arithmetic operations
  defineArithmeticBlock('operation_add', 'ADD', '#5C81A6');
  defineArithmeticBlock('operation_subtract', 'SUBTRACT', '#5C81A6');
  defineArithmeticBlock('operation_multiply', 'MULTIPLY', '#5C81A6');
  defineArithmeticBlock('operation_divide', 'DIVIDE', '#5C81A6');
  defineArithmeticBlock('operation_modulo', 'MODULO', '#5C81A6');

  // Math functions
  defineArithmeticBlock('operation_max', 'MAX', '#5CA65C');
  defineArithmeticBlock('operation_min', 'MIN', '#5CA65C');
  defineSingleValueBlock('operation_round', 'ROUND', '#5CA65C');
  defineSingleValueBlock('operation_floor', 'FLOOR', '#5CA65C');
  defineSingleValueBlock('operation_ceiling', 'CEILING', '#5CA65C');

  // Logic operations
  defineIfBlock();
  defineLogicBlock('operation_and', 'AND', '#5C68A6');
  defineLogicBlock('operation_or', 'OR', '#5C68A6');
  defineSingleValueBlock('operation_not', 'NOT', '#5C68A6');

  // Comparison operations
  defineComparisonBlock('operation_equals', 'EQUALS', '#A65C81');
  defineComparisonBlock('operation_not_equals', 'NOT_EQUALS', '#A65C81');
  defineComparisonBlock('operation_greater_than', 'GREATER_THAN', '#A65C81');
  defineComparisonBlock('operation_less_than', 'LESS_THAN', '#A65C81');
  defineComparisonBlock('operation_gte', 'GREATER_THAN_OR_EQUALS', '#A65C81');
  defineComparisonBlock('operation_lte', 'LESS_THAN_OR_EQUALS', '#A65C81');

  // Data access
  defineGetBlock();
  defineLookupBlock();
  defineTableLookupBlock();

  // Value blocks
  defineValueBlocks();
}

/**
 * Define arithmetic operation blocks (ADD, SUBTRACT, etc.)
 */
function defineArithmeticBlock(type: string, operation: OperationType, colour: string): void {
  Blockly.Blocks[type] = {
    init: function () {
      this.appendValueInput('VALUE0').setCheck('Number').appendField(operation);
      this.appendValueInput('VALUE1').setCheck('Number').appendField('+');
      this.setOutput(true, 'Number');
      this.setColour(colour);
      this.setTooltip(`Perform ${operation} operation on two or more values`);
      this.setHelpUrl('');
    },
  };
}

/**
 * Define single-value operation blocks (ROUND, FLOOR, etc.)
 */
function defineSingleValueBlock(type: string, operation: OperationType, colour: string): void {
  Blockly.Blocks[type] = {
    init: function () {
      this.appendValueInput('VALUE').setCheck('Number').appendField(operation);
      this.setOutput(true, 'Number');
      this.setColour(colour);
      this.setTooltip(`Perform ${operation} operation`);
      this.setHelpUrl('');
    },
  };
}

/**
 * Define IF/THEN/ELSE block
 */
function defineIfBlock(): void {
  Blockly.Blocks['operation_if'] = {
    init: function () {
      this.appendValueInput('CONDITION').setCheck('Boolean').appendField('IF');
      this.appendValueInput('THEN').appendField('THEN');
      this.appendValueInput('ELSE').appendField('ELSE');
      this.setOutput(true);
      this.setColour('#5C68A6');
      this.setTooltip('Conditional IF/THEN/ELSE operation');
      this.setHelpUrl('');
    },
  };
}

/**
 * Define logic operation blocks (AND, OR)
 */
function defineLogicBlock(type: string, operation: OperationType, colour: string): void {
  Blockly.Blocks[type] = {
    init: function () {
      this.appendValueInput('VALUE0').setCheck('Boolean').appendField(operation);
      this.appendValueInput('VALUE1').setCheck('Boolean').appendField('+');
      this.setOutput(true, 'Boolean');
      this.setColour(colour);
      this.setTooltip(`Perform ${operation} operation`);
      this.setHelpUrl('');
    },
  };
}

/**
 * Define comparison blocks (EQUALS, GREATER_THAN, etc.)
 */
function defineComparisonBlock(type: string, operation: OperationType, colour: string): void {
  Blockly.Blocks[type] = {
    init: function () {
      this.appendValueInput('VALUE0').appendField(operation);
      this.appendValueInput('VALUE1');
      this.setOutput(true, 'Boolean');
      this.setColour(colour);
      this.setTooltip(`Compare two values using ${operation}`);
      this.setHelpUrl('');
    },
  };
}

/**
 * Define GET block for accessing variables
 */
function defineGetBlock(): void {
  Blockly.Blocks['operation_get'] = {
    init: function () {
      this.appendDummyInput().appendField('GET').appendField(new Blockly.FieldTextInput('variable'), 'KEY');
      this.setOutput(true);
      this.setColour('#A68B5C');
      this.setTooltip('Get a variable value');
      this.setHelpUrl('');
    },
  };
}

/**
 * Define LOOKUP block
 */
function defineLookupBlock(): void {
  Blockly.Blocks['operation_lookup'] = {
    init: function () {
      this.appendValueInput('TABLE').appendField('LOOKUP in table');
      this.appendValueInput('KEY').appendField('with key');
      this.appendValueInput('DEFAULT').appendField('default');
      this.setOutput(true);
      this.setColour('#A68B5C');
      this.setTooltip('Lookup value in a table');
      this.setHelpUrl('');
    },
  };
}

/**
 * Define TABLE_LOOKUP block
 */
function defineTableLookupBlock(): void {
  Blockly.Blocks['operation_table_lookup'] = {
    init: function () {
      this.appendDummyInput().appendField('TABLE_LOOKUP').appendField(new Blockly.FieldTextInput('table_name'), 'TABLE');
      this.appendValueInput('KEY').appendField('key');
      this.setOutput(true);
      this.setColour('#A68B5C');
      this.setTooltip('Lookup value in a named table');
      this.setHelpUrl('');
    },
  };
}

/**
 * Define value blocks (numbers, text, booleans)
 */
function defineValueBlocks(): void {
  Blockly.Blocks['value_number'] = {
    init: function () {
      this.appendDummyInput().appendField(new Blockly.FieldNumber(0), 'VALUE');
      this.setOutput(true, 'Number');
      this.setColour('#745CA6');
      this.setTooltip('A number value');
      this.setHelpUrl('');
    },
  };

  Blockly.Blocks['value_text'] = {
    init: function () {
      this.appendDummyInput().appendField(new Blockly.FieldTextInput(''), 'VALUE');
      this.setOutput(true, 'String');
      this.setColour('#745CA6');
      this.setTooltip('A text value');
      this.setHelpUrl('');
    },
  };

  Blockly.Blocks['value_boolean'] = {
    init: function () {
      this.appendDummyInput().appendField(new Blockly.FieldCheckbox('TRUE'), 'VALUE');
      this.setOutput(true, 'Boolean');
      this.setColour('#745CA6');
      this.setTooltip('A boolean value (true/false)');
      this.setHelpUrl('');
    },
  };
}

/**
 * Serialize Blockly workspace to Operation object
 * This will be used for Blockly -> YAML conversion
 */
export function serializeWorkspaceToOperation(workspace: Blockly.WorkspaceSvg): Operation | null {
  const topBlocks = workspace.getTopBlocks(false);
  if (topBlocks.length === 0) {
    return null;
  }

  // For now, just serialize the first top block
  const block = topBlocks[0];
  return serializeBlockToOperation(block);
}

/**
 * Recursively serialize a Blockly block to an Operation object
 */
function serializeBlockToOperation(block: Blockly.Block): Operation {
  const type = block.type;

  // Handle different block types
  if (type === 'operation_if') {
    const condition = getInputValue(block, 'CONDITION');
    const thenValue = getInputValue(block, 'THEN');
    const elseValue = getInputValue(block, 'ELSE');
    return {
      operation: 'IF',
      condition: condition || undefined,
      then: thenValue || undefined,
      else: elseValue || undefined,
    };
  }

  if (type === 'operation_get') {
    const key = block.getFieldValue('KEY');
    return {
      operation: 'GET',
      key,
    };
  }

  if (type.startsWith('value_')) {
    const value = block.getFieldValue('VALUE');
    return { operation: 'GET', value }; // Simplified
  }

  // Default: handle as multi-value operation
  const operation = type.replace('operation_', '').toUpperCase() as OperationType;
  const values: any[] = [];
  let i = 0;
  while (block.getInput(`VALUE${i}`)) {
    const val = getInputValue(block, `VALUE${i}`);
    if (val !== null) {
      values.push(val);
    }
    i++;
  }

  return {
    operation,
    values: values.length > 0 ? values : undefined,
  };
}

/**
 * Get the value connected to an input
 */
function getInputValue(block: Blockly.Block, inputName: string): Operation | null {
  const input = block.getInput(inputName);
  if (!input) return null;

  const connection = input.connection;
  if (!connection) return null;

  const targetBlock = connection.targetBlock();
  if (!targetBlock) return null;

  return serializeBlockToOperation(targetBlock);
}

/**
 * Deserialize an Operation object to Blockly blocks
 * This will be used for YAML -> Blockly conversion
 */
export function deserializeOperationToWorkspace(operation: Operation, workspace: Blockly.WorkspaceSvg): void {
  workspace.clear();
  const block = deserializeOperationToBlock(operation, workspace);
  if (block) {
    // Blocks are automatically rendered in workspace
    // No need to call initSvg() or render() manually
  }
}

/**
 * Recursively deserialize an Operation to a Blockly block
 */
function deserializeOperationToBlock(operation: Operation, workspace: Blockly.WorkspaceSvg): Blockly.Block | null {
  const opType = operation.operation.toLowerCase();
  const blockType = `operation_${opType}`;

  const block = workspace.newBlock(blockType);

  // Handle special cases
  if (operation.operation === 'IF') {
    if (operation.condition) {
      const condBlock = deserializeOperationToBlock(operation.condition, workspace);
      if (condBlock) {
        block.getInput('CONDITION')?.connection?.connect(condBlock.outputConnection!);
      }
    }
    if (operation.then) {
      const thenBlock = deserializeOperationToBlock(operation.then, workspace);
      if (thenBlock) {
        block.getInput('THEN')?.connection?.connect(thenBlock.outputConnection!);
      }
    }
    if (operation.else) {
      const elseBlock = deserializeOperationToBlock(operation.else, workspace);
      if (elseBlock) {
        block.getInput('ELSE')?.connection?.connect(elseBlock.outputConnection!);
      }
    }
  } else if (operation.operation === 'GET' && operation.key) {
    block.setFieldValue(operation.key, 'KEY');
  } else if (operation.values) {
    // Multi-value operations
    operation.values.forEach((val, i) => {
      if (typeof val === 'object' && 'operation' in val) {
        const childBlock = deserializeOperationToBlock(val, workspace);
        if (childBlock) {
          block.getInput(`VALUE${i}`)?.connection?.connect(childBlock.outputConnection!);
        }
      } else {
        // Create a value block
        const valueBlock = workspace.newBlock('value_number');
        valueBlock.setFieldValue(val.toString(), 'VALUE');
        block.getInput(`VALUE${i}`)?.connection?.connect(valueBlock.outputConnection!);
      }
    });
  }

  return block;
}
