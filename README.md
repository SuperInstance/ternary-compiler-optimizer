# ternary-compiler-optimizer

**Optimization passes for ternary bytecode**

[![ternary](https://img.shields.io/badge/ecosystem-ternary-blue)](https://github.com/orgs/SuperInstance/repositories?q=ternary)
[![tests](https://img.shields.io/badge/tests-23-green)]()

## Overview

Optimization passes for ternary bytecode.

Provides dead trit elimination, constant folding, trit merging, peephole
optimization, loop detection, and a configurable optimization pipeline.

## Architecture

- **`Program`** — core data structure
- **`OptimizationResult`** — core data structure
- **`PeepholeOptimizer`** — core data structure
- **`LoopInfo`** — core data structure
- **`OptimizationPipeline`** — core data structure
- **`Trit`** — state enumeration
- **`Op`** — state enumeration

### Key Functions

- `from_i8()`
- `to_i8()`
- `new()`
- `len()`
- `is_empty()`
- `dead_trit_elimination()`
- `constant_folding()`
- `trit_merging()`
- `new()`
- `optimize()`
- ... and 6 more

## Why Ternary?

The balanced ternary system {-1, 0, +1} (also known as Z₃) is the mathematically optimal discrete encoding:
- **More expressive than binary**: three states capture positive, neutral, and negative
- **Natural for decisions**: accept/reject/abstain, buy/hold/sell, agree/disagree/neutral
- **Self-balancing**: the 0 state acts as a universal screen, preventing pathological lock-in
- **Z₃ cyclic dynamics**: rock-paper-scissors is the only natural coordination mechanism

## Stats

| Metric | Value |
|--------|-------|
| Lines of Rust | 635 |
| Test count | 23 |
| Public types | 7 |
| Public functions | 16 |

## Ecosystem

This crate is part of the **[SuperInstance Ternary Fleet](https://github.com/orgs/SuperInstance/repositories?q=ternary)**:

- **[ternary-core](https://github.com/SuperInstance/ternary-core)** — shared traits and Z₃ arithmetic
- **[ternary-grid](https://github.com/SuperInstance/ternary-grid)** — spatial grid with {-1, 0, +1} cells
- **[ternary-graph](https://github.com/SuperInstance/ternary-graph)** — ternary-weighted graph algorithms
- **[ternary-automata](https://github.com/SuperInstance/ternary-automata)** — three-state cellular automata
- **[ternary-compiler](https://github.com/SuperInstance/ternary-compiler)** — expression compiler and optimizer

200+ crates. 4,300+ tests. One pattern.

## Research Context

The ternary approach connects to several active research areas:
- **Ternary Neural Networks** (TNNs): weights constrained to {-1, 0, +1} for efficient inference
- **Huawei's ternary chip**: 7nm ternary silicon with 60% less power consumption
- **Active inference**: free energy minimization naturally maps to ternary action selection
- **Cyclic dominance**: RPS dynamics maintain biodiversity in spatial ecology
- **Z₃ group theory**: the only algebraic group on three elements is cyclic addition mod 3

## Usage

```toml
[dependencies]
ternary-compiler-optimizer = "0.1.0"
```

```rust
use ternary_compiler_optimizer;
```

## License

MIT
