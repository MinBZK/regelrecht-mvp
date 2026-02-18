# RFC-007: Mutation Testing for Test Quality Assurance

**Status:** Proposed
**Date:** 2026-02-18
**Authors:** regelrecht team

## Context

This project relies heavily on AI-assisted development (Claude Code) for writing both production code and tests. While AI generates tests that pass, there is a known failure mode: **AI tends to write tests that are always green but have no functional value**. Examples include:

- Tests that assert on hardcoded expected values derived from the code itself (tautological tests)
- Tests that only check happy paths without verifying edge cases
- Tests with overly permissive assertions (`assert result.is_ok()` instead of checking the actual value)
- Tests that pass regardless of whether the underlying logic is correct

Traditional code coverage metrics (line coverage, branch coverage) cannot detect this problem. A test suite can achieve 100% coverage while catching zero real bugs. We need a way to verify that our tests actually detect faults in the code.

Mutation testing solves this by systematically introducing small faults ("mutants") into the production code — for example replacing `>` with `>=`, removing a `+ 1`, or swapping `true` for `false` — and then checking whether the test suite catches these mutations. A mutant that survives (tests still pass) reveals a weakness in the test suite.

This is especially important for a law execution engine where correctness is critical: a subtle off-by-one error or wrong comparison operator in a tax calculation must be caught by tests.

## Decision

Adopt **cargo-mutants** as the mutation testing tool for the Rust engine codebase. Run mutation testing in CI on a periodic basis (weekly) and on-demand during development.

## Why

### Benefits

| Benefit | Description |
|---------|-------------|
| Catches weak AI-generated tests | Directly addresses the problem of tests that pass but verify nothing meaningful |
| Validates test effectiveness | Quantifies how well tests detect real faults, beyond coverage metrics |
| Improves confidence in law execution | For a legal engine, we need assurance that tests would catch calculation errors |
| Guides test improvement | Surviving mutants point to exactly where tests need strengthening |
| Rust-native tooling | cargo-mutants integrates naturally with the existing Cargo workflow |

### Tradeoffs

| Tradeoff | Mitigation |
|----------|------------|
| Slow execution (runs full test suite per mutant) | Run weekly in CI, not on every push; use `--in-place` for speed; parallelize with `-j` |
| False positives (equivalent mutants) | Accept some noise; review surviving mutants manually; use `mutants.toml` to skip known-equivalent mutations |
| Additional CI cost | Weekly schedule limits cost; can use GitHub Actions concurrency to bound resource usage |
| Learning curve for interpreting results | Mutation score and surviving mutant list are intuitive; no complex setup needed |

### Alternatives Considered

**Alternative 1: mutmut (Python)**
- Mutation testing tool for Python
- Not applicable: our engine is Rust, not Python
- Would only be relevant if we still had the Python prototype

**Alternative 2: Stryker**
- Mature mutation testing framework supporting JavaScript/TypeScript, C#, Scala
- No Rust support
- Could be relevant if we add a TypeScript frontend with logic, but not for the engine

**Alternative 3: Manual test review**
- Rely on code review to catch weak tests
- Humans consistently miss tautological tests, especially AI-generated ones
- Does not scale; not systematic
- Why not: mutation testing automates exactly this judgment

**Alternative 4: Property-based testing only (proptest)**
- Already partially used; generates random inputs
- Good complement but does not verify that existing unit tests are meaningful
- Why not: solves a different problem (input coverage vs. assertion quality)

### Implementation Notes

**Tool installation:**
```bash
cargo install cargo-mutants
```

**Basic usage:**
```bash
# Run mutation testing on the engine
cd packages/engine
cargo mutants

# Run with parallelism for speed
cargo mutants -j 4

# Run only on specific files
cargo mutants -f src/engine.rs
```

**CI integration (weekly):**
```yaml
# .github/workflows/mutation-testing.yml
name: Mutation Testing
on:
  schedule:
    - cron: '0 6 * * 1'  # Monday 06:00 UTC
  workflow_dispatch: {}

jobs:
  mutants:
    name: Mutation Testing
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: packages/engine
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-mutants
      - run: cargo mutants --timeout-multiplier 3 -j 2
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: mutation-report
          path: packages/engine/mutants.out/
```

**Configuration (`packages/engine/mutants.toml`):**
```toml
# Exclude generated code or code where mutations are always equivalent
exclude_globs = ["src/generated/**"]
```

**Interpreting results:**
- **Killed mutant**: test suite caught the fault (good)
- **Survived mutant**: tests passed despite the fault (test gap — investigate)
- **Timeout**: mutant caused an infinite loop (usually fine, counts as caught)
- **Unviable**: mutant didn't compile (skip, not meaningful)

Target: aim for a mutation score above 70% initially, increasing over time as we address surviving mutants.

## References

- [cargo-mutants](https://github.com/sourcefrog/cargo-mutants) — Rust mutation testing tool
- [Stryker Mutator](https://stryker-mutator.io/) — mutation testing for JS/TS/C#/Scala
- [mutmut](https://github.com/boxed/mutmut) — Python mutation testing
- [Mutation Testing on Wikipedia](https://en.wikipedia.org/wiki/Mutation_testing)
- ["An Analysis and Survey of the Development of Mutation Testing"](https://doi.org/10.1109/TSE.2010.62) — Jia & Harman, IEEE TSE 2011
