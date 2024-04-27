// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use rand::{Rng, thread_rng};
use aptos_crypto::poseidon_bn254::{BYTES_PACKED_PER_SCALAR, MAX_NUM_INPUT_SCALARS, pad_and_hash_bytes_with_len};
use aptos_crypto::test_utils::random_bytes;
use aptos_keyless_common::input_processing::circuit_input_signals::CircuitInputSignals;
use aptos_keyless_common::input_processing::config::CircuitPaddingConfig;
use crate::TestCircuitHandle;

static CIRCUIT_SRC_TEMPLATE: &str = r#"
pragma circom 2.1.3;

include "helpers/hashtofield.circom";

template hash_bytes_to_field_with_len_test(max_len) {
    signal input in[max_len];
    signal input len;
    signal input expected_output;
    component c1 = HashBytesToFieldWithLen(max_len);
    c1.in <== in;
    c1.len <== len;
    expected_output === c1.hash;
}

component main = hash_bytes_to_field_with_len_test(__MAX_LEN__);
"#;

#[test]
fn hash_bytes_to_field_with_len() {
    // Have to save 1 scalar slot for the length.
    let max_supported_byte_len = (MAX_NUM_INPUT_SCALARS - 1) * BYTES_PACKED_PER_SCALAR;

    let mut rng = thread_rng();
    let num_iterations = std::env::var("NUM_ITERATIONS").unwrap_or("10".to_string()).parse::<usize>().unwrap_or(10);

    //TODO: hardcode some interesting circuit dimensions that's widely used in keyless.

    for i in 0..num_iterations {
        println!();
        println!("Iteration {} starts.", i);
        let num_bytes_circuit_capacity: usize = rng.gen_range(1, max_supported_byte_len);
        println!("num_bytes_circuit_capacity={}", num_bytes_circuit_capacity);
        let circuit_src = CIRCUIT_SRC_TEMPLATE.replace("__MAX_LEN__", num_bytes_circuit_capacity.to_string().as_str());
        let circuit = TestCircuitHandle::new_from_str(circuit_src.as_str()).unwrap();
        let input_len = rng.gen_range(0, num_bytes_circuit_capacity + 1);
        println!("input_len={}", input_len);
        let msg = random_bytes(&mut rng, input_len);
        let expected_output = pad_and_hash_bytes_with_len(msg.as_slice(), num_bytes_circuit_capacity).unwrap();
        println!("expected_output={}", expected_output);
        let config = CircuitPaddingConfig::new()
            .max_length("in", num_bytes_circuit_capacity);
        let circuit_input_signals = CircuitInputSignals::new()
            .bytes_input("in", msg.as_slice())
            .usize_input("len", msg.len())
            .fr_input("expected_output", expected_output)
            .pad(&config)
            .unwrap();
        let result = circuit.gen_witness(circuit_input_signals);
        println!("gen_witness_result={:?}", result);
        assert!(result.is_ok());
    }
}
