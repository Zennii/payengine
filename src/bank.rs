use crate::account::Account;
use crate::transaction::{LoggedTransaction, Transaction};
use anyhow::{Context, Error, Result};
use csv::Trim;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::Path;

#[derive(Default)]
pub struct Bank {
    accounts: HashMap<u16, Account>,
    transaction_log: HashMap<u32, LoggedTransaction>,
}

impl Display for Bank {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "client, available, held, total, locked")?;
        for account in self.accounts.values() {
            writeln!(f, "{}", account)?;
        }
        Ok(())
    }
}

impl Bank {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    #[cfg(test)]
    pub fn num_accounts(&self) -> usize {
        self.accounts.len()
    }

    #[cfg(test)]
    pub fn get_account(&self, client: u16) -> Option<&Account> {
        self.accounts.get(&client)
    }

    fn get_or_create_account(&mut self, client: u16) -> &mut Account {
        self.accounts
            .entry(client)
            .or_insert_with(|| Account::new(client))
    }

    #[cfg(test)]
    pub fn num_logs(&self) -> usize {
        self.transaction_log.len()
    }

    #[cfg(test)]
    pub fn get_logged_transaction(&self, tx: u32) -> Option<&LoggedTransaction> {
        self.transaction_log.get(&tx)
    }

    fn log_transaction(&mut self, transaction: Transaction) {
        self.transaction_log
            .insert(transaction.tx, LoggedTransaction::from(transaction));
    }

    pub fn process_transactions<P: AsRef<Path>>(&mut self, transaction_path: P) -> Result<()> {
        let mut transactions = csv::ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(true)
            .from_reader(File::options().read(true).open(transaction_path)?);

        // Read the data in chunks (BufReader under the hood)
        for transaction_result in transactions.deserialize() {
            let transaction: Transaction = match transaction_result {
                Ok(transaction) => transaction,
                Err(err) => {
                    // Skip entries that fail to parse as transactions.
                    eprintln!("{:?}, skipping...", err);
                    continue;
                }
            };

            if let Err(err) = self.handle_transaction(transaction) {
                // The transaction has failed!
                eprintln!("{:?}, skipping...", err);
            }
        }

        Ok(())
    }

    fn handle_transaction(&mut self, transaction: Transaction) -> Result<()> {
        match transaction.r#type.to_lowercase().as_str() {
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

    fn deposit(&mut self, transaction: Transaction) -> Result<()> {
        if self.transaction_log.contains_key(&transaction.tx) {
            return Err(Error::msg(format!(
                "[deposit] Transaction {} already exists",
                transaction.tx
            )));
        }

        let account = self.get_or_create_account(transaction.client);

        let amount = transaction.amount.context(format!(
            "[deposit] Transaction {} did not specify amount",
            transaction.tx
        ))?;

        account.deposit(amount);

        self.log_transaction(transaction);
        Ok(())
    }

    fn withdrawal(&mut self, transaction: Transaction) -> Result<()> {
        if self.transaction_log.contains_key(&transaction.tx) {
            return Err(Error::msg(format!(
                "[withdrawal] Transaction {} already exists",
                transaction.tx
            )));
        }

        let amount = transaction.amount.context(format!(
            "[withdrawal] Transaction {} did not specify amount",
            transaction.tx
        ))?;

        let account = self.get_or_create_account(transaction.client);

        account.withdraw(amount).context(format!(
            "[withdrawal] Transaction {} failed",
            transaction.tx
        ))?;

        self.log_transaction(transaction);
        Ok(())
    }

    fn dispute(&mut self, transaction: Transaction) -> Result<()> {
        let in_question = self
            .transaction_log
            .get_mut(&transaction.tx)
            .context(format!(
                "[dispute] Invalid transaction reference {}",
                transaction.tx
            ))?;

        if in_question.client != transaction.client {
            return Err(Error::msg(format!(
                "[resolve] Client value {} did not match reference client {} for transaction {}",
                transaction.client, in_question.client, transaction.tx
            )));
        }

        if in_question.disputed {
            // We're already disputing this transaction
            return Err(Error::msg(format!(
                "[dispute] Transaction {} already disputed",
                transaction.tx
            )));
        }

        let amount = in_question.amount.context(format!(
            "[dispute] Transaction {} did not specify amount",
            transaction.tx
        ))?;

        in_question.disputed = true;

        let account = self.get_or_create_account(transaction.client);

        account.dispute(amount);
        Ok(())
    }

    fn resolve(&mut self, transaction: Transaction) -> Result<()> {
        let in_question = self
            .transaction_log
            .get_mut(&transaction.tx)
            .context(format!(
                "[resolve] Invalid transaction reference {}",
                transaction.tx
            ))?;

        if in_question.client != transaction.client {
            return Err(Error::msg(format!(
                "[resolve] Client value {} did not match reference client {} for transaction {}",
                transaction.client, in_question.client, transaction.tx
            )));
        }

        if !in_question.disputed {
            // We're not disputing this transaction
            return Err(Error::msg(format!(
                "[resolve] Transaction {} not disputed",
                transaction.tx
            )));
        }

        in_question.disputed = false;

        let amount = in_question.amount.context(format!(
            "[resolve] Transaction {} did not specify amount",
            transaction.tx
        ))?;

        let account = self.get_or_create_account(transaction.client);

        account.resolve(amount);
        Ok(())
    }

    fn chargeback(&mut self, transaction: Transaction) -> Result<()> {
        let in_question = self
            .transaction_log
            .get_mut(&transaction.tx)
            .context(format!(
                "[chargeback] Invalid transaction reference {}",
                transaction.tx
            ))?;

        if in_question.client != transaction.client {
            return Err(Error::msg(format!(
                "[resolve] Client value {} did not match reference client {} for transaction {}",
                transaction.client, in_question.client, transaction.tx
            )));
        }

        if !in_question.disputed {
            // We're not disputing this transaction
            return Err(Error::msg(format!(
                "[chargeback] Transaction {} not disputed",
                transaction.tx
            )));
        }

        let amount = in_question.amount.context(format!(
            "[chargeback] Transaction {} did not specify amount",
            transaction.tx
        ))?;

        let account = self.get_or_create_account(transaction.client);

        account.chargeback(amount);

        Ok(())
    }
}
