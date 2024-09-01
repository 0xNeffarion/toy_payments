use anyhow::{Context, Result};
use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::BTreeMap;

///
/// Represents an account of a client
///
#[derive(Serialize, Debug)]
pub struct Account {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    pub const fn new(client: u16) -> Self {
        Self {
            client,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false,
        }
    }
}

///
/// Represents a collection of accounts
/// Client id is used for the key for faster lookups
///
pub struct Accounts(BTreeMap<u16, Account>);

impl Default for Accounts {
    fn default() -> Self {
        Self::new()
    }
}

impl Accounts {
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn get(&self, client: u16) -> Option<&Account> {
        self.0.get(&client)
    }

    ///
    /// Returns a mutable account for a given client id
    /// If the account does not exist, it will be created and returned
    ///
    pub fn get_mut(&mut self, client: u16) -> &mut Account {
        self.0.entry(client).or_insert_with(|| Account::new(client))
    }

    ///
    /// Writes to stdout the state of all accounts in a CSV format
    /// Since the accounts are stored in a `BTreeMap`, the output is sorted by the client id
    ///
    /// # Errors
    ///
    /// If the csv writer fails to serialize the account to a csv record
    ///
    pub fn print_state(&self) -> Result<()> {
        let lock = std::io::stdout().lock();

        let mut csv_writer = csv::WriterBuilder::default()
            .delimiter(b',')
            .has_headers(true)
            .from_writer(lock);

        for account in self.0.values() {
            csv_writer.serialize(account).with_context(|| {
                format!("Failed to serialize account to csv record: {account:?}")
            })?;
        }

        csv_writer.flush().with_context(|| {
            "Failed to flush csv writer to stdout while attempting to print accounts"
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_account() {
        let account = Account::new(1);

        assert_eq!(account.client, 1);
        assert_eq!(account.available, 0.into());
        assert_eq!(account.held, 0.into());
        assert_eq!(account.total, 0.into());
        assert!(!account.locked);
    }

    #[test]
    fn new_accounts_is_empty() {
        let accounts = Accounts::new();

        assert!(accounts.0.is_empty());
    }

    #[test]
    fn get_mut_account() {
        let mut accounts = Accounts::new();

        let account = accounts.get_mut(1);

        assert_eq!(account.client, 1);
        assert_eq!(account.available, 0.into());
        assert_eq!(account.held, 0.into());
        assert_eq!(account.total, 0.into());
        assert!(!account.locked);
    }

    #[test]
    fn get_mut_account_twice() {
        let mut accounts = Accounts::new();

        let account = accounts.get_mut(1);
        account.available = Decimal::from(100);

        let account = accounts.get_mut(1);

        assert_eq!(account.client, 1);
        assert_eq!(account.available, Decimal::from(100));
        assert_eq!(account.held, 0.into());
        assert!(!account.locked);
    }

    #[test]
    fn get_mut_account_twice_different() {
        let mut accounts = Accounts::new();

        let account = accounts.get_mut(1);
        account.available = Decimal::from(100);

        let account = accounts.get_mut(2);

        assert_eq!(account.client, 2);
        assert_eq!(account.available, 0.into());
        assert_eq!(account.held, 0.into());
        assert_eq!(account.total, 0.into());
        assert!(!account.locked);
    }
}
