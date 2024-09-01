use anyhow::{Context, Result};
use toy_payments::{Accounts, Engine, Transactions};

fn main() -> Result<()> {
    // Create a new accounts instance
    let accounts = Accounts::new();

    // Create a new engine instance
    let mut engine = Engine::new(accounts);

    // Read the transactions from the csv file in the arguments
    let transactions = Transactions::from_args()
        .with_context(|| "Failed to retrieve transactions file in arguments")?;

    // Feed the transactions to the engine and process them
    //
    // if we want to process multiple transactions files (or in smaller chunks)
    // we can call `engine.process(trxs)`` multiple times with more transactions
    engine.process(transactions);

    // Write the state of the accounts to stdout as csv
    engine
        .accounts()
        .print_state()
        .with_context(|| "Failed to print accounts state to stdout")?;

    Ok(())
}
