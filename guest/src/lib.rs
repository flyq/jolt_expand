use jolt_core::{
    host::Program, instruction::add::ADDInstruction, tracer, BytecodeRow, Jolt, JoltCommitments,
    JoltField, JoltPreprocessing, MemoryOp, RV32IJoltProof, RV32IJoltVM,
    MEMORY_OPS_PER_INSTRUCTION, RV32I,
};

pub fn build_fib() -> (
    impl Fn(u32) -> (u128, jolt::Proof),
    impl Fn(jolt::Proof) -> bool,
) {
    let (program, preprocessing) = preprocess_fib();
    let program = std::rc::Rc::new(program);
    let preprocessing = std::rc::Rc::new(preprocessing);
    let program_cp = program.clone();
    let preprocessing_cp = preprocessing.clone();
    let prove_closure = move |n: u32| {
        let program = (*program).clone();
        let preprocessing = (*preprocessing).clone();
        prove_fib(program, preprocessing, n)
    };
    let verify_closure = move |proof: jolt::Proof| {
        let program = (*program_cp).clone();
        let preprocessing = (*preprocessing_cp).clone();
        RV32IJoltVM::verify(preprocessing, proof.proof, proof.commitments).is_ok()
    };
    (prove_closure, verify_closure)
}
pub fn fib(n: u32) -> u128 {
    {
        let mut a: u128 = 0;
        let mut b: u128 = 1;
        let mut sum: u128;
        for _ in 1..n {
            sum = a + b;
            a = b;
            b = sum;
        }
        b
    }
}

pub fn analyze_fib(n: u32) -> jolt::host::analyze::ProgramSummary {
    let mut program = Program::new("fibonacci-guest");
    program.set_func("fib");
    program.set_std(false);
    program.set_memory_size(10485760u64);
    program.set_stack_size(4096u64);
    program.set_max_input_size(4096u64);
    program.set_max_output_size(4096u64);
    program.set_input(&n);
    program.trace_analyze::<jolt::F>()
}

pub fn preprocess_fib() -> (
    jolt::host::Program,
    jolt::JoltPreprocessing<jolt::F, jolt::CommitmentScheme>,
) {
    let mut program = Program::new("fibonacci-guest");
    program.set_func("fib");
    program.set_std(false);
    program.set_memory_size(10485760u64);
    program.set_stack_size(4096u64);
    program.set_max_input_size(4096u64);
    program.set_max_output_size(4096u64);
    let (bytecode, memory_init) = program.decode();
    let preprocessing: JoltPreprocessing<jolt::F, jolt::CommitmentScheme> =
        RV32IJoltVM::preprocess(bytecode, memory_init, 1 << 20, 1 << 20, 1 << 24);
    (program, preprocessing)
}

pub fn prove_fib(
    mut program: jolt::host::Program,
    preprocessing: jolt::JoltPreprocessing<jolt::F, jolt::CommitmentScheme>,
    n: u32,
) -> (u128, jolt::Proof) {
    program.set_input(&n);
    let (io_device, bytecode_trace, instruction_trace, memory_trace, circuit_flags) =
        program.trace();
    let output_bytes = io_device.outputs.clone();
    let (jolt_proof, jolt_commitments) = RV32IJoltVM::prove(
        io_device,
        bytecode_trace,
        memory_trace,
        instruction_trace,
        circuit_flags,
        preprocessing,
    );
    let ret_val = jolt::postcard::from_bytes::<u128>(&output_bytes).unwrap();
    let proof = jolt::Proof {
        proof: jolt_proof,
        commitments: jolt_commitments,
    };
    (ret_val, proof)
}
