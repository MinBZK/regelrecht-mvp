// Test suite for the WASM package
// Run after: wasm-pack build --target web --features wasm
import { initSync, WasmEngine } from '../pkg/regelrecht_engine.js';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));

let passed = 0;
let failed = 0;

function test(name, fn) {
    try {
        fn();
        console.log(`  [PASS] ${name}`);
        passed++;
    } catch (err) {
        console.log(`  [FAIL] ${name}: ${err.message}`);
        failed++;
    }
}

function assertEqual(actual, expected, message) {
    if (actual !== expected) {
        throw new Error(`${message}: expected ${expected}, got ${actual}`);
    }
}

function assertTrue(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

async function main() {
    console.log('Initializing WASM...');

    // Load WASM binary manually for Node.js
    const wasmPath = join(__dirname, '../pkg/regelrecht_engine_bg.wasm');
    const wasmBuffer = readFileSync(wasmPath);
    initSync({ module: wasmBuffer });

    const testYaml = `
$id: test_law
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Test article
    machine_readable:
      execution:
        parameters:
          - name: value
            type: number
            required: true
        output:
          - name: result
            type: number
        actions:
          - output: result
            operation: MULTIPLY
            values:
              - $value
              - 2
`;

    // ==========================================
    // Basic functionality tests
    // ==========================================
    console.log('\n--- Basic functionality tests ---');

    const engine = new WasmEngine();

    test('new engine has zero laws', () => {
        assertEqual(engine.lawCount(), 0, 'Law count');
    });

    test('version returns string', () => {
        const version = engine.version();
        assertTrue(typeof version === 'string', 'Version should be string');
        assertTrue(version.length > 0, 'Version should not be empty');
    });

    test('loadLaw returns law ID', () => {
        const lawId = engine.loadLaw(testYaml);
        assertEqual(lawId, 'test_law', 'Law ID');
    });

    test('lawCount increases after load', () => {
        assertEqual(engine.lawCount(), 1, 'Law count');
    });

    test('hasLaw returns true for loaded law', () => {
        assertTrue(engine.hasLaw('test_law'), 'hasLaw should return true');
    });

    test('hasLaw returns false for unknown law', () => {
        assertTrue(!engine.hasLaw('nonexistent'), 'hasLaw should return false');
    });

    test('listLaws returns array with law IDs', () => {
        const laws = engine.listLaws();
        assertTrue(Array.isArray(laws), 'listLaws should return array');
        assertTrue(laws.includes('test_law'), 'listLaws should include test_law');
    });

    // ==========================================
    // getLawInfo tests
    // ==========================================
    console.log('\n--- getLawInfo tests ---');

    test('getLawInfo returns law metadata', () => {
        const info = engine.getLawInfo('test_law');
        assertEqual(info.id, 'test_law', 'Law ID');
        assertEqual(info.regulatory_layer, 'WET', 'Regulatory layer');
        assertTrue(Array.isArray(info.outputs), 'Outputs should be array');
        assertTrue(info.outputs.includes('result'), 'Outputs should include result');
    });

    // ==========================================
    // Execute tests
    // ==========================================
    console.log('\n--- Execute tests ---');

    test('execute computes correct result', () => {
        const result = engine.execute('test_law', 'result', { value: 21 }, '2025-01-01');
        assertEqual(result.outputs.result, 42, 'Output value (21 * 2 = 42)');
    });

    test('execute result has correct structure', () => {
        const result = engine.execute('test_law', 'result', { value: 10 }, '2025-01-01');
        assertEqual(result.article_number, '1', 'Article number');
        assertEqual(result.law_id, 'test_law', 'Law ID');
        assertTrue('outputs' in result, 'Result should have outputs');
        assertTrue('resolved_inputs' in result, 'Result should have resolved_inputs');
    });

    // ==========================================
    // HashMap serialization verification
    // ==========================================
    console.log('\n--- HashMap serialization tests ---');

    test('outputs is plain object not Map', () => {
        const result = engine.execute('test_law', 'result', { value: 5 }, '2025-01-01');
        assertTrue(!(result.outputs instanceof Map), 'outputs should not be a Map');
        assertTrue(typeof result.outputs === 'object', 'outputs should be an object');
        assertTrue(!Array.isArray(result.outputs), 'outputs should not be an array');
    });

    test('outputs accessible via dot notation', () => {
        const result = engine.execute('test_law', 'result', { value: 5 }, '2025-01-01');
        assertEqual(result.outputs.result, 10, 'Dot notation access');
    });

    test('outputs accessible via bracket notation', () => {
        const result = engine.execute('test_law', 'result', { value: 5 }, '2025-01-01');
        assertEqual(result.outputs['result'], 10, 'Bracket notation access');
    });

    test('Object.keys works on outputs', () => {
        const result = engine.execute('test_law', 'result', { value: 5 }, '2025-01-01');
        const keys = Object.keys(result.outputs);
        assertTrue(Array.isArray(keys), 'Object.keys should return array');
        assertTrue(keys.includes('result'), 'Keys should include result');
    });

    // ==========================================
    // Error handling tests
    // ==========================================
    console.log('\n--- Error handling tests ---');

    test('loadLaw rejects invalid YAML', () => {
        try {
            engine.loadLaw('invalid yaml {{{');
            throw new Error('Should have thrown');
        } catch (err) {
            assertTrue(err.toString().includes('YAML') || err.toString().includes('parse'),
                'Error should mention YAML or parse');
        }
    });

    test('loadLaw rejects duplicate law ID', () => {
        try {
            engine.loadLaw(testYaml); // Already loaded
            throw new Error('Should have thrown');
        } catch (err) {
            assertTrue(err.toString().includes('already loaded'), 'Error should mention already loaded');
        }
    });

    test('execute throws for unknown law', () => {
        try {
            engine.execute('nonexistent', 'output', {}, '2025-01-01');
            throw new Error('Should have thrown');
        } catch (err) {
            assertTrue(err.toString().includes('not found') || err.toString().includes('nonexistent'),
                'Error should mention law not found');
        }
    });

    test('execute throws for unknown output', () => {
        try {
            engine.execute('test_law', 'nonexistent_output', {}, '2025-01-01');
            throw new Error('Should have thrown');
        } catch (err) {
            assertTrue(err.toString().includes('not found') || err.toString().includes('nonexistent'),
                'Error should mention output not found');
        }
    });

    test('getLawInfo throws for unknown law', () => {
        try {
            engine.getLawInfo('nonexistent');
            throw new Error('Should have thrown');
        } catch (err) {
            assertTrue(err.toString().includes('not found') || err.toString().includes('nonexistent'),
                'Error should mention law not found');
        }
    });

    // ==========================================
    // Cleanup tests
    // ==========================================
    console.log('\n--- Cleanup tests ---');

    test('unloadLaw removes law', () => {
        assertTrue(engine.unloadLaw('test_law'), 'unloadLaw should return true');
        assertEqual(engine.lawCount(), 0, 'Law count after unload');
    });

    test('unloadLaw returns false for unknown law', () => {
        assertTrue(!engine.unloadLaw('nonexistent'), 'unloadLaw should return false');
    });

    // ==========================================
    // Summary
    // ==========================================
    console.log('\n========================================');
    console.log(`Tests: ${passed} passed, ${failed} failed`);
    console.log('========================================');

    if (failed > 0) {
        process.exit(1);
    }
}

main().catch(err => {
    console.error('Error:', err);
    process.exit(1);
});
