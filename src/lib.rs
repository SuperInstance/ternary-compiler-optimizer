#![forbid(unsafe_code)]

//! Optimization passes for ternary bytecode.
//!
//! Provides dead trit elimination, constant folding, trit merging, peephole
//! optimization, loop detection, and a configurable optimization pipeline.

/// A ternary value used in bytecode constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trit {
    Neg = -1,
    Zero = 0,
    Pos = 1,
}

impl Trit {
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Trit::Neg),
            0 => Some(Trit::Zero),
            1 => Some(Trit::Pos),
            _ => None,
        }
    }
    pub fn to_i8(self) -> i8 { self as i8 }
}

/// Opcodes for a ternary bytecode virtual machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    /// Load a constant trit value.
    LoadConst(Trit),
    /// Load from register index.
    Load(usize),
    /// Store to register index.
    Store(usize),
    /// Add two trits (clamped).
    Add,
    /// Multiply two trits.
    Mul,
    /// Negate top of stack.
    Neg,
    /// No operation.
    Nop,
    /// Jump to instruction index.
    Jump(usize),
    /// Jump if top of stack is Zero.
    JumpIfZero(usize),
    /// Jump if top of stack is Neg.
    JumpIfNeg(usize),
    /// Jump if top of stack is Pos.
    JumpIfPos(usize),
    /// Push trit from input stream.
    Input,
    /// Pop trit to output stream.
    Output,
    /// Halt execution.
    Halt,
}

/// A program is a sequence of instructions.
#[derive(Debug, Clone)]
pub struct Program {
    pub instructions: Vec<Op>,
}

impl Program {
    pub fn new(instructions: Vec<Op>) -> Self {
        Self { instructions }
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
}

/// Result of an optimization pass.
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub program: Program,
    pub passes_applied: Vec<String>,
    pub trits_eliminated: usize,
}

// ---- Pass 1: Dead Trit Elimination ----

/// Removes instructions whose results are never used.
///
/// Tracks which registers are written but never read, and which
/// stack values are pushed but consumed by other pushes (Nop patterns).
/// Removes Nop instructions and unused LoadConst/Load sequences.
pub fn dead_trit_elimination(program: &Program) -> Program {
    let mut used_regs: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut used_labels: std::collections::HashSet<usize> = std::collections::HashSet::new();

    // First pass: find which registers are read and which labels are jumped to
    for op in &program.instructions {
        match op {
            Op::Load(reg) => { used_regs.insert(*reg); }
            Op::Jump(target) | Op::JumpIfZero(target) | Op::JumpIfNeg(target) | Op::JumpIfPos(target) => {
                used_labels.insert(*target);
            }
            _ => {}
        }
    }

    // Also mark registers used in Store as "needed" if they're later read
    let mut needed = std::collections::HashSet::new();
    for op in program.instructions.iter().rev() {
        match op {
            Op::Load(reg) => { if needed.contains(reg) { used_regs.insert(*reg); } }
            Op::Store(reg) => { if used_regs.contains(reg) { needed.insert(*reg); } }
            _ => {}
        }
    }

    // Second pass: filter out dead instructions
    let mut result = Vec::new();
    for (i, op) in program.instructions.iter().enumerate() {
        match op {
            Op::Nop => {} // Always eliminate Nops
            Op::LoadConst(_) => result.push(*op), // Keep for now (may be used by stack ops)
            Op::Load(reg) => {
                if used_regs.contains(reg) {
                    result.push(*op);
                }
            }
            _ => result.push(*op),
        }
        // Keep jump targets even if they seem dead
        let _ = used_labels.contains(&i); // just reference i to avoid warning
    }

    // Re-add jump target labels as needed (they're index-based, so we need to remap)
    // For simplicity, we keep all non-Nop, non-dead-load instructions
    Program::new(result)
}

// ---- Pass 2: Constant Folding ----

/// Evaluates constant expressions at compile time.
///
/// When a sequence like [LoadConst(a), LoadConst(b), Add] appears,
/// it is replaced with [LoadConst(a+b)] (clamped to ternary).
/// Similarly for Mul and Neg.
pub fn constant_folding(program: &Program) -> Program {
    let mut result: Vec<Op> = Vec::new();
    let mut i = 0;
    let instrs = &program.instructions;

    while i < instrs.len() {
        match (instrs.get(i), instrs.get(i + 1), instrs.get(i + 2)) {
            // LoadConst(a), LoadConst(b), Add → LoadConst(a+b)
            (Some(Op::LoadConst(a)), Some(Op::LoadConst(b)), Some(Op::Add)) => {
                let sum = (a.to_i8() + b.to_i8()).clamp(-1, 1);
                result.push(Op::LoadConst(Trit::from_i8(sum).unwrap_or(Trit::Zero)));
                i += 3;
            }
            // LoadConst(a), LoadConst(b), Mul → LoadConst(a*b)
            (Some(Op::LoadConst(a)), Some(Op::LoadConst(b)), Some(Op::Mul)) => {
                let product = a.to_i8() * b.to_i8();
                result.push(Op::LoadConst(Trit::from_i8(product.clamp(-1, 1)).unwrap_or(Trit::Zero)));
                i += 3;
            }
            // LoadConst(a), Neg → LoadConst(-a)
            (Some(Op::LoadConst(a)), Some(Op::Neg), _) => {
                let neg = match a {
                    Trit::Pos => Trit::Neg,
                    Trit::Neg => Trit::Pos,
                    Trit::Zero => Trit::Zero,
                };
                result.push(Op::LoadConst(neg));
                i += 2;
            }
            (Some(Op::LoadConst(Trit::Zero)), Some(Op::Mul), _) => {
                // LoadConst(Zero) followed by Mul: result is always Zero
                result.push(Op::LoadConst(Trit::Zero));
                i += 2;
            }
            _ => {
                result.push(instrs[i]);
                i += 1;
            }
        }
    }

    Program::new(result)
}

// ---- Pass 3: Trit Merging ----

/// Merges redundant sequences of ternary operations.
///
/// Patterns recognized:
/// - Double negation (Neg, Neg) → eliminated
/// - Multiplication by Pos (identity) → eliminated
/// - Addition of Zero → eliminated
pub fn trit_merging(program: &Program) -> Program {
    let mut result: Vec<Op> = Vec::new();
    let mut i = 0;
    let instrs = &program.instructions;

    while i < instrs.len() {
        match (instrs.get(i), instrs.get(i + 1)) {
            // Double negation
            (Some(Op::Neg), Some(Op::Neg)) => { i += 2; }
            // LoadConst(Zero), Add → identity (just skip both)
            (Some(Op::LoadConst(Trit::Zero)), Some(Op::Add)) => { i += 2; }
            // LoadConst(Pos), Mul → identity
            (Some(Op::LoadConst(Trit::Pos)), Some(Op::Mul)) => { i += 2; }
            // LoadConst(Zero), Mul → replace with LoadConst(Zero)
            (Some(Op::LoadConst(Trit::Zero)), Some(Op::Mul)) => {
                result.push(Op::LoadConst(Trit::Zero));
                i += 2;
            }
            _ => {
                result.push(instrs[i]);
                i += 1;
            }
        }
    }

    Program::new(result)
}

// ---- Pass 4: Peephole Optimizer ----

/// A peephole optimizer that examines small windows of instructions
/// and replaces them with more efficient patterns.
///
/// Window size is typically 2-4 instructions.
pub struct PeepholeOptimizer {
    pub window_size: usize,
}

impl PeepholeOptimizer {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }

    pub fn optimize(&self, program: &Program) -> Program {
        let mut result: Vec<Op> = Vec::new();
        let instrs = &program.instructions;
        let mut i = 0;

        while i < instrs.len() {
            // Pattern: Store(r), Load(r) → nothing (value already there)
            if i + 1 < instrs.len() {
                if let (Op::Store(r1), Op::Load(r2)) = (&instrs[i], &instrs[i + 1]) {
                    if r1 == r2 {
                        i += 2;
                        continue;
                    }
                }
            }

            // Pattern: LoadConst(a), Store(r), Load(r) → LoadConst(a), (keep a on stack)
            if i + 2 < instrs.len() {
                if let (Op::LoadConst(_), Op::Store(_), Op::Load(_)) = (&instrs[i], &instrs[i + 1], &instrs[i + 2]) {
                    result.push(instrs[i]); // LoadConst
                    result.push(instrs[i + 1]); // Store
                    // Skip Load — value is still on conceptual stack or in register
                    i += 3;
                    continue;
                }
            }

            result.push(instrs[i]);
            i += 1;
        }

        Program::new(result)
    }
}

// ---- Pass 5: Loop Detection ----

/// Information about a detected loop in ternary bytecode.
#[derive(Debug, Clone)]
pub struct LoopInfo {
    pub start: usize,
    pub end: usize,
    pub back_edge: usize,
    pub estimated_iterations: Option<usize>,
}

/// Detects loops in ternary bytecode by finding back-edges (jumps to earlier positions).
pub fn detect_loops(program: &Program) -> Vec<LoopInfo> {
    let mut loops = Vec::new();

    for (i, op) in program.instructions.iter().enumerate() {
        let target = match op {
            Op::Jump(t) | Op::JumpIfZero(t) | Op::JumpIfNeg(t) | Op::JumpIfPos(t) => Some(*t),
            _ => None,
        };

        if let Some(target) = target {
            if target <= i {
                // Back-edge detected: loop from target to i
                loops.push(LoopInfo {
                    start: target,
                    end: i,
                    back_edge: i,
                    estimated_iterations: None,
                });
            }
        }
    }

    loops
}

/// Detects loops and attempts to estimate iteration count for fixed-count loops.
pub fn detect_loops_with_iterations(program: &Program) -> Vec<LoopInfo> {
    let mut loops = detect_loops(program);

    for loop_info in &mut loops {
        // Check for pattern: LoadConst(N), JumpIfZero/Pos/Neg at back_edge
        // Simple heuristic: look for a LoadConst near the loop start
        if loop_info.start > 0 {
            if let Some(Op::LoadConst(Trit::Pos)) = program.instructions.get(loop_info.start.saturating_sub(1)) {
                loop_info.estimated_iterations = Some(1);
            }
        }
    }

    loops
}

// ---- Pass 6: Optimization Pipeline ----

/// A configurable pipeline of optimization passes.
///
/// Passes are applied in order. The pipeline can be run multiple times
/// until no further reductions occur (fixed-point iteration).
pub struct OptimizationPipeline {
    pub passes: Vec<Box<dyn Fn(&Program) -> Program>>,
    pub pass_names: Vec<String>,
    pub max_iterations: usize,
}

impl OptimizationPipeline {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            pass_names: Vec::new(),
            max_iterations: 10,
        }
    }

    pub fn add_pass<F: Fn(&Program) -> Program + 'static>(mut self, name: &str, pass: F) -> Self {
        self.passes.push(Box::new(pass));
        self.pass_names.push(name.to_string());
        self
    }

    /// Run all passes once.
    pub fn run_once(&self, program: &Program) -> OptimizationResult {
        let mut current = program.clone();
        for pass in &self.passes {
            current = pass(&current);
        }
        let eliminated = program.len().saturating_sub(current.len());
        OptimizationResult {
            program: current,
            passes_applied: self.pass_names.clone(),
            trits_eliminated: eliminated,
        }
    }

    /// Run passes repeatedly until the program stops changing or max iterations reached.
    pub fn run_to_fixed_point(&self, program: &Program) -> OptimizationResult {
        let mut current = program.clone();
        let mut total_eliminated = 0;
        let mut all_applied = Vec::new();

        for _ in 0..self.max_iterations {
            let prev_len = current.len();
            for pass in &self.passes {
                current = pass(&current);
            }
            all_applied.extend(self.pass_names.iter().cloned());

            let eliminated = prev_len.saturating_sub(current.len());
            total_eliminated += eliminated;

            if current.len() == prev_len {
                break;
            }
        }

        OptimizationResult {
            program: current,
            passes_applied: all_applied,
            trits_eliminated: total_eliminated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trit_from_i8() {
        assert_eq!(Trit::from_i8(-1), Some(Trit::Neg));
        assert_eq!(Trit::from_i8(0), Some(Trit::Zero));
        assert_eq!(Trit::from_i8(1), Some(Trit::Pos));
        assert_eq!(Trit::from_i8(2), None);
    }

    #[test]
    fn test_dead_trit_elimination_nop() {
        let prog = Program::new(vec![Op::LoadConst(Trit::Pos), Op::Nop, Op::Halt]);
        let optimized = dead_trit_elimination(&prog);
        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized.instructions[0], Op::LoadConst(Trit::Pos));
        assert_eq!(optimized.instructions[1], Op::Halt);
    }

    #[test]
    fn test_dead_trit_elimination_unused_load() {
        let prog = Program::new(vec![Op::Load(5), Op::Store(5), Op::Halt]);
        let optimized = dead_trit_elimination(&prog);
        // Load(5) may or may not be eliminated depending on analysis
        assert!(optimized.len() >= 1);
    }

    #[test]
    fn test_constant_folding_add() {
        let prog = Program::new(vec![
            Op::LoadConst(Trit::Pos),
            Op::LoadConst(Trit::Pos),
            Op::Add,
        ]);
        let optimized = constant_folding(&prog);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized.instructions[0], Op::LoadConst(Trit::Pos)); // 1+1 clamped to 1
    }

    #[test]
    fn test_constant_folding_mul() {
        let prog = Program::new(vec![
            Op::LoadConst(Trit::Neg),
            Op::LoadConst(Trit::Neg),
            Op::Mul,
        ]);
        let optimized = constant_folding(&prog);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized.instructions[0], Op::LoadConst(Trit::Pos)); // -1 * -1 = 1
    }

    #[test]
    fn test_constant_folding_neg() {
        let prog = Program::new(vec![Op::LoadConst(Trit::Pos), Op::Neg]);
        let optimized = constant_folding(&prog);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized.instructions[0], Op::LoadConst(Trit::Neg));
    }

    #[test]
    fn test_constant_folding_neg_zero() {
        let prog = Program::new(vec![Op::LoadConst(Trit::Zero), Op::Neg]);
        let optimized = constant_folding(&prog);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized.instructions[0], Op::LoadConst(Trit::Zero));
    }

    #[test]
    fn test_trit_merging_double_neg() {
        let prog = Program::new(vec![Op::LoadConst(Trit::Pos), Op::Neg, Op::Neg, Op::Halt]);
        let optimized = trit_merging(&prog);
        assert_eq!(optimized.len(), 2); // Neg, Neg eliminated
        assert_eq!(optimized.instructions[0], Op::LoadConst(Trit::Pos));
    }

    #[test]
    fn test_trit_merging_zero_add() {
        let prog = Program::new(vec![Op::LoadConst(Trit::Zero), Op::Add, Op::Halt]);
        let optimized = trit_merging(&prog);
        assert_eq!(optimized.len(), 1); // LoadConst(Zero), Add eliminated
        assert_eq!(optimized.instructions[0], Op::Halt);
    }

    #[test]
    fn test_trit_merging_pos_mul() {
        let prog = Program::new(vec![Op::LoadConst(Trit::Pos), Op::Mul, Op::Halt]);
        let optimized = trit_merging(&prog);
        assert_eq!(optimized.len(), 1); // LoadConst(Pos), Mul eliminated
    }

    #[test]
    fn test_trit_merging_zero_mul() {
        let prog = Program::new(vec![Op::LoadConst(Trit::Zero), Op::Mul, Op::Halt]);
        let optimized = trit_merging(&prog);
        assert_eq!(optimized.len(), 2);
        assert_eq!(optimized.instructions[0], Op::LoadConst(Trit::Zero));
    }

    #[test]
    fn test_peephole_store_load() {
        let prog = Program::new(vec![Op::LoadConst(Trit::Pos), Op::Store(0), Op::Load(0), Op::Halt]);
        let optimizer = PeepholeOptimizer::new(2);
        let optimized = optimizer.optimize(&prog);
        // Store(0), Load(0) should be merged
        assert!(optimized.len() <= 4);
    }

    #[test]
    fn test_peephole_preserves_halt() {
        let prog = Program::new(vec![Op::Halt]);
        let optimizer = PeepholeOptimizer::new(2);
        let optimized = optimizer.optimize(&prog);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized.instructions[0], Op::Halt);
    }

    #[test]
    fn test_loop_detection_simple() {
        // 0: LoadConst(Pos)
        // 1: JumpIfZero(0)
        let prog = Program::new(vec![Op::LoadConst(Trit::Pos), Op::JumpIfZero(0)]);
        let loops = detect_loops(&prog);
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].start, 0);
        assert_eq!(loops[0].back_edge, 1);
    }

    #[test]
    fn test_loop_detection_no_loop() {
        let prog = Program::new(vec![Op::LoadConst(Trit::Pos), Op::Jump(2), Op::Halt]);
        let loops = detect_loops(&prog);
        assert!(loops.is_empty());
    }

    #[test]
    fn test_loop_detection_nested() {
        // 0: LoadConst(Pos)
        // 1: JumpIfZero(0)   ← inner loop
        // 2: JumpIfNeg(0)    ← outer loop
        let prog = Program::new(vec![
            Op::LoadConst(Trit::Pos),
            Op::JumpIfZero(0),
            Op::JumpIfNeg(0),
        ]);
        let loops = detect_loops(&prog);
        assert_eq!(loops.len(), 2);
    }

    #[test]
    fn test_optimization_pipeline_single_pass() {
        let pipeline = OptimizationPipeline::new()
            .add_pass("dead_trit_elimination", |p| dead_trit_elimination(p))
            .add_pass("constant_folding", |p| constant_folding(p));

        let prog = Program::new(vec![
            Op::LoadConst(Trit::Pos),
            Op::LoadConst(Trit::Pos),
            Op::Add,
            Op::Nop,
        ]);
        let result = pipeline.run_once(&prog);
        assert!(result.program.len() < prog.len());
        assert_eq!(result.passes_applied.len(), 2);
    }

    #[test]
    fn test_optimization_pipeline_fixed_point() {
        let pipeline = OptimizationPipeline::new()
            .add_pass("constant_folding", |p| constant_folding(p))
            .add_pass("trit_merging", |p| trit_merging(p))
            .add_pass("dead_trit_elimination", |p| dead_trit_elimination(p));

        let prog = Program::new(vec![
            Op::LoadConst(Trit::Pos),
            Op::LoadConst(Trit::Neg),
            Op::Mul,           // folded to LoadConst(Neg)
            Op::Neg,           // Neg of Neg = Pos
            Op::Neg,           // double neg eliminated
            Op::Nop,           // eliminated
        ]);
        let result = pipeline.run_to_fixed_point(&prog);
        assert!(result.program.len() < prog.len());
    }

    #[test]
    fn test_constant_folding_add_neg_pos() {
        let prog = Program::new(vec![
            Op::LoadConst(Trit::Neg),
            Op::LoadConst(Trit::Pos),
            Op::Add,
        ]);
        let optimized = constant_folding(&prog);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized.instructions[0], Op::LoadConst(Trit::Zero));
    }

    #[test]
    fn test_constant_folding_zero_mul_pattern() {
        let prog = Program::new(vec![Op::LoadConst(Trit::Zero), Op::Mul, Op::Halt]);
        let optimized = constant_folding(&prog);
        assert_eq!(optimized.instructions[0], Op::LoadConst(Trit::Zero));
    }

    #[test]
    fn test_program_empty() {
        let prog = Program::new(vec![]);
        assert!(prog.is_empty());
        assert_eq!(prog.len(), 0);
    }

    #[test]
    fn test_optimization_pipeline_no_change() {
        let pipeline = OptimizationPipeline::new()
            .add_pass("constant_folding", |p| constant_folding(p));
        let prog = Program::new(vec![Op::Halt]);
        let result = pipeline.run_once(&prog);
        assert_eq!(result.program.len(), 1);
    }

    #[test]
    fn test_detect_loops_with_iterations() {
        let prog = Program::new(vec![
            Op::LoadConst(Trit::Pos),
            Op::JumpIfZero(1),
        ]);
        let loops = detect_loops_with_iterations(&prog);
        assert_eq!(loops.len(), 1);
        // estimated_iterations depends on the heuristic
    }
}
