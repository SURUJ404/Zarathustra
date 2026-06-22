wasm_bindgen_test_configure!(run_in_browser);

extern crate wasm_bindgen_test;
extern crate zarathustra_core;
extern crate zarathustra_field;
use wasm_bindgen_test::*;
use zarathustra_ast::flat::{Parameter, Variable};
use zarathustra_ast::ir::{Prog, Statement};
use zarathustra_field::Bn128Field;
use zarathustra_interpreter::Interpreter;
use zarathustra_proof_systems::{Backend, NonUniversalBackend};

use rand_0_8::{rngs::StdRng, SeedableRng};
use zarathustra_ark::Ark;
use zarathustra_proof_systems::groth16::G16;

#[wasm_bindgen_test]
fn generate_proof() {
    let program: Prog<Bn128Field> = Prog {
        module_map: Default::default(),
        arguments: vec![Parameter::public(Variable::new(0))],
        return_count: 1,
        statements: vec![Statement::constraint(
            Variable::new(0),
            Variable::new(0),
            None,
        )],
        solvers: vec![],
    };

    let interpreter = Interpreter::default();
    let witness = interpreter
        .execute(
            &[Bn128Field::from(42)],
            program.statements.iter(),
            &program.arguments,
            &program.solvers,
        )
        .unwrap();

    let rng = &mut StdRng::from_entropy();
    let keypair = <Ark as NonUniversalBackend<Bn128Field, G16>>::setup(program.clone(), rng);
    let _proof = <Ark as Backend<Bn128Field, G16>>::generate_proof(
        program,
        witness,
        keypair.pk.as_slice(),
        rng,
    );
}
