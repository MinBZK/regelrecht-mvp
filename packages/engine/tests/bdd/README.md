# Rust BDD Tests

This directory contains Cucumber/Gherkin BDD tests for the Rust engine.

## Running the Tests

```bash
# Via just
just rust-bdd

# Or directly via cargo
cd packages/engine && cargo test --test bdd -- --nocapture
```

## Current Status

**6 of 17 scenarios pass** (as of implementation date)

### Passing Scenarios

1. **Bijstand: Burger uit gemeente zonder verordening** - Error handling for missing municipal regulations
2. **Erfgrensbeplanting: Boom in Amsterdam** - Municipal regulation is correctly found and applied
3. **Erfgrensbeplanting: Heg in Amsterdam** - Municipal regulation is correctly found and applied
4. **Zorgtoeslag: Standard premium for 2025** - Ministry regulation correctly loaded
5. **Zorgtoeslag: No regeling for 2024** - Known limitation: passes because engine returns 2025 data (year mismatch detection)
6. **Zorgtoeslag: Healthcare allowance calculation** - Full calculation with mocked external data

### Known Limitations (Failing Scenarios)

#### Bijstand Tests (10 scenarios)
- **Issue**: Rust engine doesn't have "uitvoerder context" mechanism for `gedragscategorie`
- **Root cause**: The participatiewet YAML only passes `bsn` to the delegation, but the verordening needs `gedragscategorie` which requires an uitvoerder context mechanism
- **Fix needed**: Implement parameter forwarding or uitvoerder context in the engine

#### Erfgrensbeplanting Without Verordening (2 scenarios)
- **Issue**: Rust engine doesn't support delegation defaults yet
- **Root cause**: When no municipal regulation is found, the engine should fall back to defaults defined in `legal_basis_for.defaults`, but this isn't implemented
- **Fix needed**: Implement delegation defaults support in resolver/service

## Architecture

```
tests/bdd/
├── main.rs                    # Test runner entry point
├── world.rs                   # World struct with test state
├── steps/
│   ├── mod.rs                 # Module exports
│   ├── given.rs               # Given step definitions
│   ├── when.rs                # When step definitions
│   └── then.rs                # Then step definitions
└── helpers/
    ├── mod.rs                 # Helper module
    ├── regulation_loader.rs   # Loads all YAML regulations
    └── value_conversion.rs    # Gherkin value type conversion
```

## Feature Files

The tests use the shared feature files from `features/`:
- `bijstand.feature` - 10 scenarios for Participatiewet
- `zorgtoeslag.feature` - 3 scenarios for healthcare allowance
- `erfgrensbeplanting.feature` - 4 scenarios for boundary planting regulations
