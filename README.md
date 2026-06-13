# Ternary Compiler Optimizer

**Ternary Compiler Optimizer** provides optimization passes for ternary bytecode — dead trit elimination, constant folding, trit merging, peephole optimization, loop detection, and a configurable optimization pipeline.

## Why It Matters

Ternary virtual machines execute bytecode representing {-1, 0, +1} operations. Unoptimized ternary code from compilation or human authoring contains redundancies: NOP sequences, dead code after unconditional jumps, constant expressions that could be precomputed, and repeated patterns that could be merged. This optimizer applies classical compiler optimization techniques adapted for ternary operations, producing smaller and faster bytecode while preserving semantics. It's the `-O2` flag for ternary VMs.

## How It Works

### Ternary Bytecode

```rust
enum Op {
    LoadConst(Trit),    // Push constant {-1, 0, +1}
    Load(usize),        // Push from register
    Store(usize),       // Pop to register
    Add,                // Pop two, push ternary sum
    Mul,                // Pop two, push ternary product
    Neg,                // Pop, push negation
    Nop,                // No operation (optimization target)
    Jump(usize),        // Unconditional jump
    JumpIfZero(usize),  // Conditional jump
    JumpIfNeg(usize),   // Conditional jump
    JumpIfPos(usize),   // Conditional jump
    Input,              // Read from input stream
    Output,             // Write to output stream
    Halt,               // Stop execution
}
```

### Optimization Passes

**1. Dead Trit Elimination:**

Remove code that is unreachable or whose results are unused:

```
After unconditional Jump → all code until next label is dead
After Halt → all subsequent code is dead
Store to register never read again → eliminate
```

Pass cost: **O(N)** single pass with liveness analysis.

**2. Constant Folding:**

Precompute constant expressions at compile time:

```
LoadConst(1), LoadConst(-1), Add → LoadConst(0)    // 1 + (-1) = 0
LoadConst(0), Mul → LoadConst(0)                    // x × 0 = 0
LoadConst(1), LoadConst(1), Mul → LoadConst(1)      // 1 × 1 = 1
```

Pass cost: **O(N)** with pattern matching.

**3. Peephole Optimization:**

Slide a small window over the instruction stream and replace patterns:

```
Neg, Neg → (nothing)           // Double negation
Jump(next_instruction) → (nothing)  // Unnecessary jump
LoadConst(x), Store(r), Load(r) → LoadConst(x)  // Redundant store/load
```

Window size: typically 2-4 instructions. Pass cost: **O(N)**.

**4. Loop Detection:**

Identify backward jumps (potential loops):

```
for each Jump(target) where target ≤ current_position:
    record loop: [target, current_position]
    estimate iteration count from condition analysis
```

Detection: **O(N)** single pass.

**5. Trit Merging:**

Combine adjacent operations on the same operands:

```
LoadConst(1), Add, LoadConst(1), Add → LoadConst(2... wait, in ternary: LoadConst(1), Add, Add
Actually: collapse multiple Add operations when possible
```

Pass cost: **O(N)** with operand tracking.

### Optimization Pipeline

```rust
Pipeline {
    passes: Vec<OptimizationPass>,  // configurable pass ordering
    iterations: usize,              // max iterations (fixpoint)
}
```

Run passes in sequence, repeat until no changes or max iterations. Total cost: **O(I · N · P)** for I iterations, N instructions, P passes.

## Quick Start

```rust
use ternary_compiler_optimizer::{Program, Optimizer, OptLevel};

let mut program = Program::new(vec![
    Op::LoadConst(Trit::Pos),
    Op::LoadConst(Trit::Neg),
    Op::Add,           // Can be folded: Pos + Neg = Zero
    Op::Nop,           // Can be eliminated
    Op::Output,
    Op::Halt,
    Op::LoadConst(Trit::Pos),  // Dead code after Halt
]);

let mut optimizer = Optimizer::new(OptLevel::Aggressive);
optimizer.optimize(&mut program);

// Program now: LoadConst(Zero), Output, Halt — 3 instructions
```

## API

| Type | Description |
|------|-------------|
| `Program` | Vec<Op> instruction sequence |
| `Op` | LoadConst, Load, Store, Add, Mul, Neg, Nop, Jump, JumpIf*, Input, Output, Halt |
| `Trit` | Neg (-1), Zero (0), Pos (+1) |
| `Optimizer` | Configurable optimization pipeline |
| `OptLevel` | None, Basic, Aggressive |
| `OptimizationPass` | Trait for custom passes |

Built-in passes: dead_trit_elimination, constant_folding, peephole, loop_detection, trit_merging.

## Architecture Notes

Ternary Compiler Optimizer provides the code optimization layer for the ternary VM in SuperInstance. In γ + η = C, optimization preserves C (semantic meaning) while reducing code size and execution time. Dead code elimination removes η (avoidance — unreachable code paths), constant folding precomputes γ (growth — evaluating expressions at compile time). Integrates with `op-codec` for bytecode encoding and `ternary-circuit` for hardware-targeted optimization.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for VM and compiler architecture.

## References

1. Aho, A. V. et al. (2006). *Compilers: Principles, Techniques, and Tools*, 2nd ed. Addison-Wesley. (Dragon Book)
2. Cooper, K. D. & Torczon, L. (2011). *Engineering a Compiler*, 2nd ed. Morgan Kaufmann.
3. Muchnick, S. S. (1997). *Advanced Compiler Design and Implementation*. Morgan Kaufmann.

## License

MIT
