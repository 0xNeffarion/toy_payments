use crate::account::Accounts;
use crate::transaction::{Transactions, Type};

///
/// This struct is responsible for managing accounts and processing incoming transactions
/// It keeps track of the last processed transaction index so we don't have to reprocess all the transactions
/// if we process multiple transactions files
///
pub struct Engine {
    accounts: Accounts,
    transactions: Transactions,
    last_processed_transaction_index: usize,
}

impl Engine {
    ///
    /// Creates a new Engine instance with a collection of accounts
    /// and an empty collection of transactions
    ///
    pub fn new(accounts: Accounts) -> Self {
        Self {
            accounts,
            transactions: Transactions::default(),
            last_processed_transaction_index: 0,
        }
    }

    pub const fn accounts(&self) -> &Accounts {
        &self.accounts
    }

    ///
    /// Processes a new collection of transactions.
    ///
    pub fn process(&mut self, trxs: Transactions) {
        self.transactions.extend(trxs);

        for index in self.last_processed_transaction_index..self.transactions.len() {
            if let Some(transaction) = self.transactions.get(index) {
                let client = transaction.client;

                // Process current transaction
                self.process_transaction(index, client);
            }
        }

        // Update the last processed transaction index so we don't have to reprocess all transactions from the start the next time
        self.last_processed_transaction_index = self.transactions.len();
    }

    ///
    /// Processes a single transaction
    ///
    fn process_transaction(&mut self, current_transaction_index: usize, client: u16) {
        // Retrieve the account for the client
        let account = self.accounts.get_mut(client);

        // Check if the account is locked, if so, skip the transaction
        if account.locked {
            return;
        }

        let transaction = self.transactions.get(current_transaction_index);
        if let Some(transaction) = transaction {
            match transaction.r#type {
                Type::Deposit => {
                    // Check if the transaction is disputed, if so, skip the transaction
                    if !transaction.disputed {
                        if let Some(amount) = &transaction.amount {
                            account.available += amount;
                            account.total += amount;
                        }
                    }
                }
                Type::Withdrawal => {
                    // Check if the transaction is disputed, if so, skip the transaction
                    if !transaction.disputed {
                        if let Some(amount) = &transaction.amount {
                            // Check if the account has enough funds to withdraw
                            if account.available < *amount {
                                return;
                            }

                            account.available -= amount;
                            account.total -= amount;
                        }
                    }
                }
                Type::Dispute => {
                    // Retrieve the referenced transaction
                    if let Some(tx) = self.transactions.get_tx_mut(transaction.tx) {
                        // Check if the transaction is already disputed, if so, skip the transaction
                        if tx.disputed {
                            return;
                        }

                        if let Some(amount) = &tx.amount {
                            account.available -= amount;
                            account.held += amount;
                            tx.disputed = true;
                        }
                    }
                }
                Type::Resolve => {
                    // Retrieve the referenced transaction
                    if let Some(tx) = self.transactions.get_tx_mut(transaction.tx) {
                        // Check if the transaction is disputed, if not, skip the transaction
                        if tx.disputed {
                            if let Some(amount) = &tx.amount {
                                account.available += amount;
                                account.held -= amount;
                                tx.disputed = false;
                            }
                        }
                    }
                }
                Type::Chargeback => {
                    // Retrieve the referenced transaction
                    if let Some(tx) = self.transactions.get_tx_mut(transaction.tx) {
                        if tx.disputed {
                            if let Some(amount) = &tx.amount {
                                account.held -= amount;
                                account.total -= amount;

                                // Lock the account
                                account.locked = true;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::account::Accounts;
    use crate::engine::Engine;
    use crate::transaction::{Transaction, Transactions, Type};
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal::Decimal;

    #[test]
    fn single_transaction_deposit_succeeds() {
        let mut engine = Engine::new(Accounts::new());
        let transaction = Transaction {
            r#type: Type::Deposit,
            client: 1,
            tx: 1,
            amount: Decimal::from_f64(0.5),
            disputed: false,
        };

        let transactions = Transactions::from(vec![transaction]);
        engine.process(transactions);

        let account = engine.accounts().get(1).expect("Failed to get account");
        assert_eq!(account.available, Decimal::from_f64(0.5).unwrap());
    }

    #[test]
    fn single_transaction_withdrawal_succeeds() {
        let mut engine = Engine::new(Accounts::new());
        let transaction = Transaction {
            r#type: Type::Withdrawal,
            client: 1,
            tx: 1,
            amount: Decimal::from_f64(0.5),
            disputed: false,
        };

        let transactions = Transactions::from(vec![transaction]);
        engine.process(transactions);
        let account = engine.accounts().get(1).expect("Failed to get account");
        assert_eq!(account.available, 0.into());
    }

    #[test]
    fn double_transaction_succeeds() {
        let mut engine = Engine::new(Accounts::new());
        let transaction1 = Transaction {
            r#type: Type::Deposit,
            client: 1,
            tx: 1,
            amount: Decimal::from_f64(0.5),
            disputed: false,
        };
        let transaction2 = Transaction {
            r#type: Type::Withdrawal,
            client: 1,
            tx: 2,
            amount: Decimal::from_f64(0.3),
            disputed: false,
        };

        engine.process(Transactions::from(vec![transaction1, transaction2]));
        let account = engine.accounts().get(1).expect("Failed to get account");
        assert_eq!(account.available, Decimal::from_f64(0.2).unwrap());
    }

    #[test]
    fn dispute_transaction_succeeds() {
        let mut engine = Engine::new(Accounts::new());

        let transaction1 = Transaction {
            r#type: Type::Deposit,
            client: 1,
            tx: 1,
            amount: Decimal::from_f64(0.5),
            disputed: false,
        };

        let transaction2 = Transaction {
            r#type: Type::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };

        engine.process(Transactions::from(vec![transaction1, transaction2]));
        let account = engine.accounts().get(1).expect("Failed to get account");
        assert_eq!(account.available, 0.into());
        assert_eq!(account.held, Decimal::from_f64(0.5).unwrap());
    }

    #[test]
    fn resolve_transaction_succeeds() {
        let mut engine = Engine::new(Accounts::new());

        let transaction1 = Transaction {
            r#type: Type::Deposit,
            client: 1,
            tx: 1,
            amount: Decimal::from_f64(0.5),
            disputed: false,
        };

        let transaction2 = Transaction {
            r#type: Type::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };

        let transaction3 = Transaction {
            r#type: Type::Resolve,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };

        engine.process(Transactions::from(vec![
            transaction1,
            transaction2,
            transaction3,
        ]));
        let account = engine.accounts().get(1).expect("Failed to get account");
        assert_eq!(account.available, Decimal::from_f64(0.5).unwrap());
        assert_eq!(account.held, 0.into());
    }

    #[test]
    fn chargeback_transaction_succeeds() {
        let mut engine = Engine::new(Accounts::new());

        let transaction1 = Transaction {
            r#type: Type::Deposit,
            client: 1,
            tx: 1,
            amount: Decimal::from_f64(0.5),
            disputed: false,
        };

        let transaction2 = Transaction {
            r#type: Type::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };

        let transaction3 = Transaction {
            r#type: Type::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };

        engine.process(Transactions::from(vec![
            transaction1,
            transaction2,
            transaction3,
        ]));
        let account = engine.accounts().get(1).expect("Failed to get account");
        assert_eq!(account.available, 0.into());
        assert_eq!(account.held, 0.into());
        assert!(account.locked);
    }

    #[test]
    fn locked_account_withdraw_fails() {
        let mut engine = Engine::new(Accounts::new());

        let transaction1 = Transaction {
            r#type: Type::Deposit,
            client: 1,
            tx: 1,
            amount: Decimal::from_f64(0.5),
            disputed: false,
        };

        let transaction2 = Transaction {
            r#type: Type::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };

        engine.process(Transactions::from(vec![transaction1, transaction2]));

        let account = engine.accounts().get(1).expect("Failed to get account");

        assert_eq!(account.available, 0.into());
        assert_eq!(account.held, Decimal::from_f32(0.5).unwrap());
        assert!(!account.locked);

        let transaction3 = Transaction {
            r#type: Type::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };

        let transaction4 = Transaction {
            r#type: Type::Withdrawal,
            client: 1,
            tx: 2,
            amount: Decimal::from_f64(0.5),
            disputed: false,
        };

        engine.process(Transactions::from(vec![transaction3, transaction4]));

        let account = engine.accounts().get(1).expect("Failed to get account");
        assert_eq!(account.available, 0.into());
        assert_eq!(account.held, 0.into());
        assert!(account.locked);
    }
}
