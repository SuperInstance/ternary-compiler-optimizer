# Future Integration: ternary-compiler-optimizer

## Current State
Optimization passes for ternary bytecode with a VM instruction set (`Op`: LoadConst, Load, Store, Add, Mul, Neg, Nop, Jump, JumpIfZero/Neg/Pos, Input, Output, Halt), `Program` representation, and optimization passes: dead trit elimination, constant folding, trit merging, peephole optimization, and loop detection.

## Integration Opportunities

### With ESP32 Bare Metal
The bytecode VM is perfect for ESP32: a simple stack machine with 15 opcodes. `Program` compiles to a compact byte array (each Op is 1-3 bytes). Dead trit elimination and constant folding reduce program size for flash-constrained devices. The `JumpIfNeg`/`JumpIfZero`/`JumpIfPos` opcodes map directly to ternary decision points in the agent's strategy.

### With ternary-compiler Pipeline
The optimizer becomes the middle stage: `ternary-compiler`'s `StrategyIR` lowers to `Program` (bytecode), then optimizer passes run (dead trit elimination, constant folding, merging), producing an optimized `Program` that `ternary-compiler`'s `Compiler` lowers to `CompiledPolicy`. This separation enables target-independent optimization before target-specific compilation.

### With WASM Target
The bytecode VM can be compiled to WASM itself, or the `Program` can be transpiled to WASM instructions. The optimizer's output is small enough to load instantly in a browser. Loop detection identifies hot loops for JIT compilation.

## Potential in Mature Systems
Every room runs optimized bytecode. The optimizer runs at room provisioning time — when a room is created, its strategy is compiled, optimized, and deployed. The optimization pipeline is pluggable: new passes can be added for specific targets (NEON-specific folding, CUDA-specific parallelization hints). The `Profiler` from ternary-compiler feeds back into the optimizer for profile-guided optimization.

## Cross-Pollination Ideas
- Peephole optimization patterns could be discovered by `ternary-inference` — infer optimization opportunities from program structure
- Loop detection connects to `ternary-scheduling-v2` — loops are scheduling cycles that need resource allocation
- Optimized programs could be diffed by `ternary-protocol::SyncProtocol` for efficient program updates across the fleet
- `Op::JumpIfNeg/Zero/Pos` maps to `ternary-grammar`'s conditional expressions — the VM directly executes grammar ASTs

## Dependencies for Next Steps
- Integration with ternary-compiler's pipeline (StrategyIR → Program → CompiledPolicy)
- no_std VM implementation for ESP32
- WASM transpilation backend
- Profile-guided optimization feedback loop from ternary-compiler's Profiler
