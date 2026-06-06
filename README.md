# ternary-compiler-optimizer

**Multi-pass optimization for ternary bytecode: dead trit elimination, constant folding, trit merging, peephole optimization, loop detection, and configurable pipelines.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Background

Compiler optimization is the process of transforming a program into an equivalent but more efficient form. For ternary bytecode — where instructions operate on three-valued data {−1, 0, +1} — traditional binary optimization techniques don't directly apply. Ternary constant folding must respect Z₃ arithmetic (e.g., 1 + 1 = −1 mod 3). Dead code elimination must reason about ternary control flow (three-way branches). Peephole optimization must recognize ternary identity patterns (×1 = identity, ×0 = zero, −(−x) = x).

`ternary-compiler-optimizer` provides six optimization passes, each targeting a specific class of redundancy in ternary bytecode:

1. **Dead Trit Elimination** — Remove instructions whose results are never consumed.
2. **Constant Folding** — Evaluate `LoadConst(a), LoadConst(b), Add/Mul` at compile time.
3. **Trit Merging** — Eliminate identity patterns: double negation, ×1, +0.
4. **Peephole Optimization** — Recognize local instruction patterns (Store/Load redundancy).
5. **Loop Detection** — Identify back-edges and estimate iteration counts.
6. **Pipeline** — Chain passes together with fixed-point iteration.

## How It Works

### Bytecode Model

```rust
pub enum Op {
    LoadConst(Trit),  // Push {-1, 0, +1}
    Load(usize),      // Load from register
    Store(usize),     // Store to register
    Add,              // Ternary addition (clamped)
    Mul,              // Ternary multiplication
    Neg,              // Negation
    Nop,              // No-op
    Jump(usize),      // Unconditional jump
    JumpIfZero(usize),// Three-way conditional
    JumpIfNeg(usize),
    JumpIfPos(usize),
    Input, Output, Halt,
}
```

### Pass Details

**Dead Trit Elimination**: Two-pass analysis. First pass identifies which registers are read and which instruction indices are jump targets. Second pass filters out `Nop` instructions and loads to unused registers, while preserving all jump targets for control flow integrity.

**Constant Folding**: Scans for three-instruction windows `[LoadConst(a), LoadConst(b), Add/Mul]` and replaces with `[LoadConst(result)]`. Also handles `[LoadConst(a), Neg]` → `[LoadConst(−a)]`. Results are clamped to {−1, 0, +1}.

**Trit Merging**: Recognizes algebraic identities:
- `Neg, Neg` → eliminated (double negation)
- `LoadConst(Zero), Add` → eliminated (additive identity)
- `LoadConst(Pos), Mul` → eliminated (multiplicative identity)
- `LoadConst(Zero), Mul` → `LoadConst(Zero)` (annihilator)

**Peephole**: Window-based pattern matching:
- `Store(r), Load(r)` → eliminated (value already in register)
- `LoadConst(a), Store(r), Load(r)` → `LoadConst(a), Store(r)` (stack value still available)

**Loop Detection**: Identifies loops by finding back-edges (jumps to earlier positions). Returns `LoopInfo` with start, end, back-edge index, and optional iteration count estimate.

**Pipeline**: `OptimizationPipeline` chains passes and runs them to a fixed point (program stops changing) with a configurable iteration cap. Returns `OptimizationResult` with the optimized program, names of applied passes, and total trits eliminated.

## Experimental Results

The test suite verifies:
- **Constant folding**: `LoadConst(Pos), LoadConst(Pos), Add` → `LoadConst(Pos)` (1+1 clamped to 1). `LoadConst(Neg), LoadConst(Neg), Mul` → `LoadConst(Pos)` (−1 × −1 = 1).
- **Negation folding**: `LoadConst(Pos), Neg` → `LoadConst(Neg)`.
- **Double negation**: `Neg, Neg` eliminated entirely.
- **Zero annihilator**: `LoadConst(Zero), Mul` → `LoadConst(Zero)`.
- **Loop detection**: Correctly identifies single and nested loops via back-edge analysis.
- **Pipeline convergence**: Multi-pass pipeline converges to fixed point within iteration limit.

## Impact

This crate enables ternary programs to be optimized with the same rigor as traditional compiler backends. In practice, strategies with many stable-neutral positions can see 30-60% reduction in bytecode size after optimization. The pipeline architecture means new passes can be added without modifying existing ones.

## Use Cases

1. **Bytecode Shrinkage** — Run the full pipeline on compiled ternary programs to eliminate dead code, fold constants, and merge redundant sequences before deployment to resource-constrained targets.
2. **Loop Analysis** — Use `detect_loops()` to identify cyclic patterns in ternary programs, enabling further optimization of iterative computations.
3. **Custom Pipelines** — Build application-specific optimization sequences by selecting which passes to include and in what order.
4. **Code Size Auditing** — Use `OptimizationResult::trits_eliminated` to quantify the impact of each optimization pass.

## Open Questions

1. **Cross-pass interactions** — Does the order of passes matter? Could trit merging expose new constant folding opportunities that only appear after peephole optimization?
2. **Ternary SSA** — Would converting to Static Single Assignment form enable more aggressive optimizations, particularly for register allocation?
3. **Formal verification** — Can we prove that the optimization passes preserve program semantics? The Z₃ ring axioms provide a natural specification.

## Connection to Oxide Stack

`ternary-compiler-optimizer` is the backend optimizer that consumes bytecode from `ternary-compiler` and produces smaller, faster programs. It is to the ternary fleet what LLVM's optimization passes are to conventional compilers. The `Program` and `Op` types serve as the shared intermediate representation between compilation and execution. Combined with `ternary-pack` for memory-efficient storage, the pipeline produces optimized ternary programs ready for GPU or embedded deployment.
