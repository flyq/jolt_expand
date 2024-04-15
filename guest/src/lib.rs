#![cfg_attr(feature = "guest", no_std)]
#![no_main]

extern crate alloc;

use alloc::rc::Rc;
use alloc::vec::Vec;

use jolt_core::{
    host::Program,
    jolt::vm::{rv32i_vm::RV32IJoltVM, Jolt, JoltPreprocessing},
};
use jolt_sdk::{postcard, Proof, F, G, RV32IM};

pub fn build_fib() -> (impl Fn(u32) -> (u128, Proof), impl Fn(Proof) -> bool) {
    let (program, preprocessing) = preprocess_fib();
    let program = Rc::new(program);
    let preprocessing = Rc::new(preprocessing);
    let program_cp = program.clone();
    let preprocessing_cp = preprocessing.clone();
    let prove_closure = move |n: u32| {
        let program = (*program).clone();
        let preprocessing = (*preprocessing).clone();
        prove_fib(program, preprocessing, n)
    };
    let verify_closure = move |proof: Proof| {
        let _program = (*program_cp).clone();
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

pub fn analyze_fib(n: u32) -> (usize, Vec<(RV32IM, usize)>) {
    let mut program = Program::new("guest");
    program.set_input(&n);
    program.trace_analyze()
}

pub fn preprocess_fib() -> (Program, JoltPreprocessing<F, G>) {
    let mut program = Program::new("guest");
    program.set_func("fib");
    let (bytecode, memory_init) = program.decode();
    let preprocessing: JoltPreprocessing<F, G> =
        RV32IJoltVM::preprocess(bytecode, memory_init, 1 << 10, 1 << 10, 1 << 14);
    (program, preprocessing)
}

pub fn prove_fib(
    mut program: Program,
    preprocessing: JoltPreprocessing<F, G>,
    n: u32,
) -> (u128, Proof) {
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
    let ret_val = postcard::from_bytes::<u128>(&output_bytes).unwrap();
    let proof = Proof {
        proof: jolt_proof,
        commitments: jolt_commitments,
    };
    (ret_val, proof)
}
