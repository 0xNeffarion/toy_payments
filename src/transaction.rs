use anyhow::{Context, Result};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

///
/// Represents all possible transaction types
///
#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

///
/// The actual transaction struct that holds the transaction data.
/// The disputed field is not part of the CSV file, but is used internally to keep track of disputed transactions
/// Since only two transaction types have amounts, the amount field is optional.
///
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Transaction {
    pub r#type: Type,
    pub client: u16,
    pub tx: u32,

    #[serde(default)]
    pub amount: Option<Decimal>,

    #[serde(skip, default)]
    pub disputed: bool,
}

///
/// Represents a collection of transactions
/// All the transactions are stored in a vec.
/// A Hashmap is used as a way to quickly find the transaction vec index by a tx id.
///
#[derive(Default)]
pub struct Transactions {
    transactions: Vec<Transaction>,
    tx_index_map: HashMap<u32, usize>,
}

impl From<Vec<Transaction>> for Transactions {
    fn from(transactions: Vec<Transaction>) -> Self {
        let mut transactions = Self {
            transactions,
            tx_index_map: HashMap::new(),
        };

        transactions.populate_map();
        transactions
    }
}

impl Transactions {
    ///
    /// Extends Transactions with another collection of Transactions.
    /// This is useful when reading multiple csv files
    /// The hashmap is repopulated after the transactions are extended
    ///
    pub fn extend(&mut self, trxs: Self) {
        self.transactions.extend(trxs.transactions);
        self.populate_map();
    }

    ///
    /// Populates the hashmap with the transaction id as the key and the index of the transaction in the vec as the value
    /// Only deposit and withdrawal transactions are added to the hashmap
    ///
    fn populate_map(&mut self) {
        for (index, transaction) in self.transactions.iter().enumerate() {
            if transaction.r#type == Type::Deposit || transaction.r#type == Type::Withdrawal {
                self.tx_index_map.insert(transaction.tx, index);
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<&Transaction> {
        self.transactions.get(index)
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    ///
    /// Returns a mutable reference to a transaction by transaction id
    /// Uses a hashmap to quickly find the index of the transaction
    ///
    pub fn get_tx_mut(&mut self, tx: u32) -> Option<&mut Transaction> {
        if let Some(index) = self.tx_index_map.get(&tx) {
            return self.transactions.get_mut(*index);
        }

        None
    }

    ///
    /// Parses the command line arguments to get the input file path from the first argument and returns a Transactions struct
    ///
    /// # Errors
    ///
    /// Returns an error if the input file path is not provided in the command line arguments
    ///
    pub fn from_args() -> Result<Self> {
        let arguments = std::env::args().collect::<Vec<_>>();
        if arguments.len() < 2 {
            eprintln!("Usage: {} <csv transactions input file>", arguments[0]);
            std::process::exit(1);
        }

        let transactions_path = PathBuf::from(&arguments[1].trim());
        Self::from_csv(&transactions_path)
    }

    ///
    /// Handles the csv parsing of a file by deserializing the records and returns a Transactions struct
    ///
    /// # Errors
    ///
    /// Returns an error if the file does not exist or if the csv parsing fails
    ///
    pub fn from_csv(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Transactions csv file does not exist: '{path:?}'"
            ));
        }

        let file = File::open(path)
            .with_context(|| format!("Failed to open transactions file: '{path:?}'"))?;

        let mut csv_reader = csv::ReaderBuilder::default()
            .delimiter(b',')
            .trim(csv::Trim::All)
            .has_headers(true)
            .flexible(true)
            .from_reader(file);

        let mut transactions = vec![];
        for (index, record) in csv_reader.records().enumerate() {
            // Deserialize the csv record
            let trx = record?
                .deserialize::<Transaction>(None)
                .with_context(|| format!("Failed to parse transaction at index: '{index}'"))?;

            // Push the transaction into the vec
            transactions.push(trx);
        }

        Ok(Self::from(transactions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transactions_count_from_csv() {
        let transactions = Transactions::from_csv(Path::new("tests/resources/inputs/trx1.csv"))
            .expect("Failed to read transactions from csv");

        assert_eq!(transactions.len(), 5);
    }

    #[test]
    fn test_transactions_get_tx_mut() {
        let mut transactions = Transactions::from_csv(Path::new("tests/resources/inputs/trx1.csv"))
            .expect("Failed to read transactions from csv");

        let tx = transactions
            .get_tx_mut(5)
            .expect("Failed to get transaction by id");

        assert_eq!(tx.client, 2);
    }
}
