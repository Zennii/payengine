use crate::account::Account;
use crate::transaction::{LoggedTransaction, Transaction};
use anyhow::{Context, Error, Result};
use csv::Trim;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::Path;

/// The bank holds the state of accounts and transactions, and is in charge
/// of processing transactions. It will validate that transactions supply
/// sane data, such as references to past transactions, before modifying
/// accounts or inserting into its log.
///
/// Only transactions that supply their own unique transaction ID are
/// logged.
#[derive(Default)]
pub struct Bank {
    accounts: HashMap<u16, Account>,
    transaction_log: HashMap<u32, LoggedTransaction>,
}

impl Display for Bank {
    /// Displays the bank in a CSV format.
    ///
    /// ```
    /// client, available, held, total, locked
    /// 2, 2.0000, 0.1234, 2.1234, false
    /// ```
    /// Note that a bank devoid of accounts will only print a header.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "client, available, held, total, locked")?;
        // Loop through all accounts and print them.
        for account in self.accounts.values() {
            writeln!(f, "{}", account)?;
        }
        Ok(())
    }
}

impl Bank {
    /// Create a new bank with an empty state.
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Returns the number of accounts.
    #[cfg(test)]
    pub fn num_accounts(&self) -> usize {
        self.accounts.len()
    }

    /// Attempts to fetch an account by client ID, returning a
    /// reference to the account if it exists.
    #[cfg(test)]
    pub fn get_account(&self, client: u16) -> Option<&Account> {
        self.accounts.get(&client)
    }

    /// Attempts to fetch an account by client ID, creating a new one
    /// if one does not exist. Returns a mutable reference to the account.
    fn get_or_create_account(&mut self, client: u16) -> &mut Account {
        self.accounts
            .entry(client)
            .or_insert_with(|| Account::new(client))
    }

    /// Returns the number of logged transactions.
    #[cfg(test)]
    pub fn num_logs(&self) -> usize {
        self.transaction_log.len()
    }

    /// Attempts to fetch a logged transaction by transaction ID, returning
    /// a reference to the LoggedTransaction if it exists.
    #[cfg(test)]
    pub fn get_logged_transaction(&self, tx: u32) -> Option<&LoggedTransaction> {
        self.transaction_log.get(&tx)
    }

    /// Sets a logged transaction to be disputed or not.
    ///
    /// Returns an Err if the logged transaction does not exist.
    fn set_disputed(&mut self, tx: u32, disputed: bool) -> Result<()> {
        // Get the transaction referenced by this transaction, returning
        // early if that transaction does not exist.
        let in_question = self
            .transaction_log
            .get_mut(&tx)
            .context(format!("Invalid transaction reference {}", tx))?;
        in_question.disputed = disputed;

        Ok(())
    }

    /// Attempts to validate the transaction referenced by the supplied
    /// transaction, returning a reference to it if it is valid.
    ///
    /// Returns an Err if the transaction fails to validate.
    fn validate_transaction_reference(
        &self,
        transaction: &Transaction,
        disputed: bool,
    ) -> Result<&LoggedTransaction> {
        // Get the transaction referenced by this transaction, returning
        // early if that transaction does not exist.
        let in_question = self
            .transaction_log
            .get(&transaction.tx)
            .context(format!("Invalid transaction reference {}", transaction.tx))?;

        if in_question.client != transaction.client {
            // The supplied client does not match the referenced client,
            // it may be erroneous.
            return Err(Error::msg(format!(
                "Client value {} did not match reference client {} for transaction {}",
                transaction.client, in_question.client, transaction.tx
            )));
        }

        if in_question.disputed == disputed {
            // We're already disputing this transaction
            return Err(Error::msg(format!(
                "Transaction {} disputed is not {}",
                transaction.tx, disputed
            )));
        }

        Ok(in_question)
    }

    /// Converts a transaction to a LoggedTransaction and inserts it into
    /// the log, keyed by its transaction ID.
    fn log_transaction(&mut self, transaction: Transaction) -> Result<()> {
        // Turn into a LoggedTransaction which strips off the transaction ID.
        self.transaction_log
            .insert(transaction.tx, LoggedTransaction::try_from(transaction)?);
        Ok(())
    }

    /// Attempts to parse the passed transaction_path as a CSV file and
    /// deserialize them into Transactions, handling them in order from
    /// top to bottom.
    ///
    /// This function returns an Err if the file fails to open.
    ///
    /// If a line fails to parse as a Transaction or fails to be handled,
    /// the line is ignored.
    pub fn process_transactions<P: AsRef<Path>>(&mut self, transaction_path: P) -> Result<()> {
        // Read the file as csv, only requiring read permissions on the file.
        // The CSV is trimmed of any whitespaces and allows a variable number
        // of fields to allow amounts to be ignored.
        //
        // Note: The csv library does not support UTF16.
        let mut transactions = csv::ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(true)
            .from_reader(File::options().read(true).open(transaction_path)?);

        // `deserialize()` will read the data in chunks (BufReader under the hood)
        // so that the entire file isn't loaded into memory at once.
        for transaction_result in transactions.deserialize() {
            // Type hint for deserialize to know we want Transaction.
            // This could also be turbofished in like
            // `deserialize::<Transaction>()` instead.
            let transaction: Transaction = match transaction_result {
                Ok(transaction) => transaction,
                Err(err) => {
                    // Skip entries that fail to parse as transactions.
                    eprintln!("{:?}, skipping...", err);
                    continue;
                }
            };

            // Handle the transaction. Note that this moves `transaction`.
            if let Err(err) = self.handle_transaction(transaction) {
                // The transaction has failed!
                eprintln!("{:?}, skipping...", err);
            }
        }

        Ok(())
    }

    /// Attempts to handle a transaction based on its type.
    ///
    /// Returns an Err if the transaction fails (eg. invalid
    /// transaction reference) or if the type is not implemented.
    fn handle_transaction(&mut self, transaction: Transaction) -> Result<()> {
        // Handle all capitalizations of the type.
        match transaction.get_type().as_str() {
            "deposit" => self.deposit(transaction),
            "withdrawal" => self.withdrawal(transaction),
            "dispute" => self.dispute(transaction),
            "resolve" => self.resolve(transaction),
            "chargeback" => self.chargeback(transaction),
            unknown => {
                return Err(Error::msg(format!(
                    "Transaction {} type '{}' not implemented",
                    transaction.tx, unknown
                )))
            }
        }
    }

    /// Attempts to perform a deposit into a related account.
    ///
    /// Returns an Err if the transaction exists already or
    /// an amount is not specified.
    ///
    /// This function does not validate transaction type and
    /// assumes all transactions passed to it are to be treated
    /// as deposits.
    fn deposit(&mut self, transaction: Transaction) -> Result<()> {
        // If the transaction already exists, return.
        if self.transaction_log.contains_key(&transaction.tx) {
            return Err(Error::msg(format!(
                "[deposit] Transaction {} already exists",
                transaction.tx
            )));
        }

        // Get the relevant account or create a new one so we can manipulate it.
        let account = self.get_or_create_account(transaction.client);

        // Return early if an amount isn't specified on the transaction.
        let amount = transaction.amount.context(format!(
            "[deposit] Transaction {} did not specify amount",
            transaction.tx
        ))?;

        // Deposit the funds
        account.deposit(amount);

        // Log for future reference. This shouldn't error if above amount didn't
        self.log_transaction(transaction)?;
        Ok(())
    }

    /// Attempts to perform a withdrawal from a related account.
    ///
    /// Returns an Err if the transaction exists already,
    /// an amount is not specified, the account does not
    /// have sufficient funds, or the account is locked.
    ///
    /// This function does not validate transaction type and
    /// assumes all transactions passed to it are to be treated
    /// as withdrawals.
    fn withdrawal(&mut self, transaction: Transaction) -> Result<()> {
        // If the transaction already exists, return.
        if self.transaction_log.contains_key(&transaction.tx) {
            return Err(Error::msg(format!(
                "[withdrawal] Transaction {} already exists",
                transaction.tx
            )));
        }

        // Return early if an amount isn't specified on the transaction.
        let amount = transaction.amount.context(format!(
            "[withdrawal] Transaction {} did not specify amount",
            transaction.tx
        ))?;

        // Get the relevant account or create a new one so we can manipulate it.
        let account = self.get_or_create_account(transaction.client);

        // Attempts to withdraw from the account, returning early if the
        // withdrawal fails due to lack of funds or the account is locked.
        account.withdraw(amount).context(format!(
            "[withdrawal] Transaction {} failed",
            transaction.tx
        ))?;

        // Log for future reference. This shouldn't error if above amount didn't
        self.log_transaction(transaction)?;
        Ok(())
    }

    /// Attempts to dispute a related transaction.
    ///
    /// Returns an Err if the related transaction is invalid
    /// or already disputed, or the account is locked.
    ///
    /// This function does not validate transaction type and
    /// assumes all transactions passed to it are to be treated
    /// as disputes.
    fn dispute(&mut self, transaction: Transaction) -> Result<()> {
        // Check referenced transaction for sanity and grab the amount.
        let amount = self
            .validate_transaction_reference(&transaction, true)
            .context("[dispute] Bad reference")?.amount;

        // Get the account for manipulation.
        let account = self.get_or_create_account(transaction.client);

        // Attempt to process the dispute, failing if the account is locked.
        account
            .dispute(amount)
            .context(format!("[dispute] Transaction {} failed", transaction.tx))?;

        // Mark the transaction for dispute.
        self.set_disputed(transaction.tx, true)
            .context("[dispute] Can't set dispute")?;
        Ok(())
    }

    /// Attempts to resolve a related transaction.
    ///
    /// Returns an Err if the related transaction is invalid
    /// or not disputed, or the account is locked.
    ///
    /// This function does not validate transaction type and
    /// assumes all transactions passed to it are to be treated
    /// as a resolve.
    fn resolve(&mut self, transaction: Transaction) -> Result<()> {
        // Check referenced transaction for sanity and grab the amount.
        let amount = self
            .validate_transaction_reference(&transaction, false)
            .context("[resolve] Bad reference")?
            .amount;

        let account = self.get_or_create_account(transaction.client);

        // Attempt to resolve disputed funds, failing if the account is locked.
        account
            .resolve(amount)
            .context(format!("[resolve] Transaction {} failed", transaction.tx))?;

        // The transaction is no longer disputed.
        self.set_disputed(transaction.tx, false)
            .context("[resolve] Can't set dispute")?;
        Ok(())
    }

    /// Attempts to chargeback a related transaction. This does
    /// not clear the transaction of its disputed status.
    ///
    /// Returns an Err if the related transaction is invalid
    /// or not disputed, or the account is locked.
    ///
    /// This function does not validate transaction type and
    /// assumes all transactions passed to it are to be treated
    /// as a chargeback.
    fn chargeback(&mut self, transaction: Transaction) -> Result<()> {
        // Check referenced transaction for sanity and grab the amount.
        let amount = self
            .validate_transaction_reference(&transaction, false)
            .context("[chargeback] Bad reference")?
            .amount;

        // Get the account for manipulation.
        let account = self.get_or_create_account(transaction.client);

        // Attempt to chargeback funds, failing if the account is locked.
        account.chargeback(amount).context(format!(
            "[chargeback] Transaction {} failed",
            transaction.tx
        ))?;

        Ok(())
    }
}
