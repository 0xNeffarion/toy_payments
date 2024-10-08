# toy_payments

Simple payments engine processor

# Usage

You will need cargo to build and run, and a transaction csv file as input.   
You can find a few csv files under `/tests/resources/inputs/` as an example.

Run with: 

`cargo run -- <csv transactions file>`

If you want to run tests, just run the following:

`cargo test`

## Code usage

If you want to use this in a code base instead of a cli, you can use it in the following way:

```rust
use toy_payments::{Accounts, Engine, Transactions};

fn main() -> Result<(), Box<dyn Error>> {
    // Create a new accounts instance
    let accounts = Accounts::new();

    // Create a new engine instance
    let mut engine = Engine::new(accounts);

    // Read the transactions from the csv file in the arguments
    let transactions = Transactions::from_args()?;

    // Feed the transactions to the engine and process them
    //
    // if we want to process multiple transactions files (or in smaller chunks)
    // we can call `engine.process(trxs)`` multiple times with more transactions
    engine.process(transactions);

    // Write the state of the accounts to stdout as csv
    engine.accounts().print_state()?;
}
```
