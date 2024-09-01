use std::process::{Command, Output};

#[test]
fn basic_transaction_1_succeeds() {
    let output = start_program("tests/resources/inputs/trx1.csv");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, include_bytes!("resources/outputs/trx1.csv"));
}

#[test]
fn basic_transaction_2_succeeds() {
    let output = start_program("tests/resources/inputs/trx2.csv");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, include_bytes!("resources/outputs/trx2.csv"));
}

#[test]
fn basic_transaction_2_changed_tx_order_is_equal_succeeds() {
    let output = start_program("tests/resources/inputs/trx4.csv");
    let output2 = start_program("tests/resources/inputs/trx4.csv");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, include_bytes!("resources/outputs/trx2.csv"));
    assert_eq!(output2.status.code(), Some(0));
    assert_eq!(output2.stdout, include_bytes!("resources/outputs/trx2.csv"));
}

#[test]
fn basic_transaction_3_succeeds() {
    let output = start_program("tests/resources/inputs/trx3.csv");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, include_bytes!("resources/outputs/trx3.csv"));
}

///
/// # Panics
///
/// Panics if the command fails to run with cargo
///
pub fn start_program(input: &str) -> Output {
    Command::new("cargo")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("run")
        .arg("--")
        .arg(input)
        .output()
        .expect("Failed to run command with cargo")
}
