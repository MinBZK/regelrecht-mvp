/**
 * YAML serialization and deserialization utilities
 * Handles conversion between JavaScript objects and YAML strings
 */

import yaml from 'js-yaml';
import type { MachineReadable, Operation } from '../types/schema';

/**
 * Serialize a machine-readable object to YAML string
 */
export function serializeToYaml(obj: MachineReadable | Operation): string {
  try {
    return yaml.dump(obj, {
      indent: 2,
      lineWidth: 80,
      noRefs: true,
      sortKeys: false,
    });
  } catch (error) {
    console.error('Failed to serialize to YAML:', error);
    throw new Error('YAML serialization failed');
  }
}

/**
 * Deserialize YAML string to JavaScript object
 */
export function deserializeFromYaml(yamlString: string): any {
  try {
    return yaml.load(yamlString);
  } catch (error) {
    console.error('Failed to parse YAML:', error);
    throw new Error('YAML parsing failed');
  }
}

/**
 * Pretty-print YAML with syntax highlighting hints
 * Returns object with line-by-line info for Monaco editor
 */
export function formatYaml(yamlString: string): string {
  try {
    const parsed = yaml.load(yamlString);
    return yaml.dump(parsed, {
      indent: 2,
      lineWidth: 80,
      noRefs: true,
      sortKeys: false,
    });
  } catch (error) {
    // If parsing fails, return original
    return yamlString;
  }
}

/**
 * Validate YAML syntax
 */
export function validateYaml(yamlString: string): { valid: boolean; error?: string } {
  try {
    yaml.load(yamlString);
    return { valid: true };
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Invalid YAML';
    return { valid: false, error: message };
  }
}

/**
 * Extract execution block from machine-readable YAML
 */
export function extractExecution(machineReadable: MachineReadable): Operation | null {
  if (!machineReadable.execution) {
    return null;
  }
  return machineReadable.execution as Operation;
}

/**
 * Create a machine-readable object from an execution operation
 */
export function createMachineReadable(execution: Operation): MachineReadable {
  return {
    execution,
  };
}
