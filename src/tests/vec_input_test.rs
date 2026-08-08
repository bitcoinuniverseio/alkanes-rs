use crate::{message::AlkaneMessageContext, tests::std::alkanes_std_test_build};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use metashrew_support::utils::consensus_encode;

use crate::index_block;
use crate::tests::helpers::{self as alkane_helpers, assert_binary_deployed_to_id};
use alkane_helpers::clear;
use alkanes::view;
#[allow(unused_imports)]
use metashrew_core::{
    println,
    stdio::{stdout, Write},
};
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn test_vec_inputs() -> Result<()> {
    clear();
    let block_height: u32 = 0;
    // Get the LoggerAlkane ID
    let logger_alkane_id = AlkaneId { block: 2, tx: 1 };

    // The bootstrap contract creates the logger from the attached WASM.
    // Keep deployment separate from the logger calls, as the factory fixture does.
    let deployment_cellpack = Cellpack {
        target: AlkaneId { block: 1, tx: 0 },
        inputs: vec![30, 2, 1, 1_000_000],
    };

    let process_numbers_cellpack = Cellpack {
        target: logger_alkane_id.clone(),
        inputs: vec![11, 4, 10, 20, 30, 40],
    };
    // Create a cellpack to call the process_strings method (opcode 12)
    // For "hello" and "world" strings with null terminators
    let hello_bytes = u128::from_le_bytes(*b"hello\0\0\0\0\0\0\0\0\0\0\0");
    let world_bytes = u128::from_le_bytes(*b"world\0\0\0\0\0\0\0\0\0\0\0");

    let process_strings_cellpack = Cellpack {
        target: logger_alkane_id.clone(),
        inputs: vec![
            12,          // opcode for process_strings
            2,           // length of the vector
            hello_bytes, // "hello" string
            world_bytes, // "world" string
        ],
    };

    // Create a cellpack to call the process_nested_vec method (opcode 15)
    let process_nested_vec_cellpack = Cellpack {
        target: logger_alkane_id.clone(),
        inputs: vec![
            13, // opcode for process_nested_vec
            2,  // length of the outer vector
            3,  // length of first inner vector
            1, 2, 3, // elements of first inner vector
            2, // length of second inner vector
            4, 5, // elements of second inner vector
        ],
    };

    // Deploy the logger through the bootstrap contract before exercising its
    // public vector methods.
    let test_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [alkanes_std_test_build::get_bytes()].into(),
        [deployment_cellpack].into(),
    );

    index_block(&test_block, block_height)?;

    // Verify the binary was deployed correctly
    let _ = assert_binary_deployed_to_id(
        logger_alkane_id.clone(),
        alkanes_std_test_build::get_bytes(),
    );

    let process_numbers_data = view::call_view(
        &logger_alkane_id,
        &process_numbers_cellpack.inputs,
        1_000_000,
    )?;

    // Verify the process_numbers result contains the expected values
    assert_eq!(
        process_numbers_data[process_numbers_data.len() - 16],
        100,
    );

    let process_strings_data = view::call_view(
        &logger_alkane_id,
        &process_strings_cellpack.inputs,
        1_000_000,
    )?;
    let strings = String::from_utf8_lossy(&process_strings_data);
    let expected_name = "hello,world";

    // Verify the get_strings result contains the expected values
    // The result should be a vector with ["hello", "world"]
    assert!(
        strings.contains(expected_name),
        "Response data should contain the name '{}', but it doesn't",
        expected_name
    );

    let process_nested_vec_data = view::call_view(
        &logger_alkane_id,
        &process_nested_vec_cellpack.inputs,
        1_000_000,
    )?;

    // The result should be the total number of elements: 3 + 2 = 5
    assert_eq!(
        process_nested_vec_data[process_nested_vec_data.len() - 16],
        5,
    );

    Ok(())
}
