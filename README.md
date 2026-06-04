# ternary-compiler-optimizer: Optimization passes for ternary bytecode

An optimization framework for bytecode that operates on {-1, 0, +1} values. Provides dead code elimination, constant folding, trit merging, peephole optimization, loop detection, and a configurable pass pipeline.

## Why This Exists

Ternary virtual machines and compilers produce bytecode that can benefit from the same class of optimizations as traditional compilers — dead code removal, constant propagation, peephole patterns — but the ternary value space creates unique opportunities. For example, multiplication by Zero always yields Zero, and double negation is always identity. This library exploits those algebraic properties specifically.

## Core Concepts

**Ternary bytecode** — Instructions that operate on a stack of trits (-1, 0, +1). Includes arithmetic (Add, Mul, Neg), control flow (Jump, conditional jumps), memory (Load, Store), and I/O (Input, Output).

**Dead trit elimination** — Removing instructions whose results are never consumed. Analogous to dead code elimination in traditional compilers, but aware of ternary stack semantics.

**Constant folding** — Evaluating compile-time-known expressions at compile time. `LoadConst(Pos), LoadConst(Neg), Mul` becomes `LoadConst(Neg)`.

**Trit merging** — Exploiting ternary algebra to simplify patterns: double negation cancels, multiplying by Pos is identity, adding Zero is identity.

**Peephole optimizer** — Scans a small sliding window of instructions and replaces known-inefficient patterns. Window size is configurable.

**Loop detection** — Identifies back-edges (jumps to earlier instruction indices) to find loops in the control flow graph.

**Optimization pipeline** — An ordered sequence of passes that can be run once or iterated to a fixed point (no further reductions).

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-compiler-optimizer = "0.1"
```

```rust
use ternary_compiler_optimizer::*;

// Create a program with optimizable patterns
let program = Program::new(vec![
    Op::LoadConst(Trit::Pos),
    Op::LoadConst(Trit::Neg),
    Op::Mul,           // can be folded to LoadConst(Neg)
    Op::Neg,            // Neg of Neg = Pos
    Op::Neg,            // double neg eliminated
    Op::Nop,            // dead trit eliminated
    Op::Halt,
]);

// Build and run an optimization pipeline
let pipeline = OptimizationPipeline::new()
    .add_pass("constant_folding", |p| constant_folding(p))
    .add_pass("trit_merging", |p| trit_merging(p))
    .add_pass("dead_trit_elimination", |p| dead_trit_elimination(p));

let result = pipeline.run_to_fixed_point(&program);
println!("Reduced from {} to {} instructions", program.len(), result.program.len());
println!("Eliminated {} trits", result.trits_eliminated);
```

## API Overview

| Type / Function | Description |
|----------------|-------------|
| `Trit` | Ternary value: `Neg`, `Zero`, `Pos` |
| `Op` | Bytecode instruction (LoadConst, Add, Mul, Jump, etc.) |
| `Program` | A sequence of instructions |
| `dead_trit_elimination(prog)` | Remove unused instructions and Nops |
| `constant_folding(prog)` | Evaluate constant expressions at compile time |
| `trit_merging(prog)` | Simplify algebraic trit patterns |
| `PeepholeOptimizer` | Window-based pattern replacement |
| `detect_loops(prog)` | Find loops via back-edge analysis |
| `OptimizationPipeline` | Configurable sequence of passes |
| `OptimizationResult` | Output: optimized program + stats |

## How It Works

Each pass takes a `Program` (vector of instructions) and returns a new `Program`. Passes are pure functions — no mutation of the input.

**Constant folding** scans for `LoadConst, LoadConst, (Add|Mul)` triples and evaluates them. The result is clamped to ternary range. Double negation `LoadConst(a), Neg` is also folded.

**Trit merging** recognizes algebraic identities: `Neg, Neg` cancels; `LoadConst(Zero), Add` is identity; `LoadConst(Pos), Mul` is identity; `LoadConst(Zero), Mul` collapses to `LoadConst(Zero)`.

**Peephole** uses a sliding window (default size 2) to spot patterns like `Store(r), Load(r)` where the load is redundant.

**Loop detection** finds backward jumps. A jump to instruction index ≤ current index indicates a back-edge, defining a loop from target to the jump instruction.

The pipeline runs passes in order and can iterate until the program size stabilizes (fixed point), with a configurable maximum iteration count to prevent infinite loops in degenerate cases.

## Known Limitations

- **No control flow graph**: Loop detection is purely based on back-edges. It does not build a full CFG, so it cannot detect irreducible control flow or perform loop-invariant code motion.
- **Jump target remapping**: After optimization removes instructions, jump targets are not remapped. This means the optimizer is currently safe only for programs without jumps, or where jump correctness is verified after optimization.
- **No register allocation**: Load/Store operate on fixed register indices. The optimizer does not perform register allocation or coalescing.
- **Peephole patterns are limited**: Only a few patterns are recognized. A production optimizer would need many more.

## Use Cases

- **Ternary VM JIT compiler** — Optimize bytecode before execution in a ternary virtual machine.
- **Ternary DSL compilation** — Optimize output from a domain-specific language that compiles to ternary bytecode.
- **Code size reduction** — Shrink ternary programs for embedded deployment (e.g., on ternary hardware simulators).
- **Educational compiler** — Teach compiler optimization concepts with a simple ternary ISA.

## Ecosystem Context

Part of the SuperInstance ternary computing ecosystem. Related crates:

- `ternary-compiler` — Frontend: parsing and code generation that produces the bytecode this crate optimizes.
- `ternary-compiler-v2` — More advanced compiler with SSA form.
- `ternary-vm` — Virtual machine that executes the optimized bytecode.

This crate sits between the compiler frontend and the VM, transforming bytecode for efficiency.

## License

MIT
