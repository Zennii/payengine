use crate::account::Accounts;
use crate::transaction::{Transaction, TransactionLog};
use anyhow::Result;
use csv::Trim;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::Path;

#[derive(Default)]
pub struct Worker {
    pub accounts: Accounts,
    pub transaction_log: TransactionLog,
}

impl Display for Worker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "client, available, held, total, locked")?;
        for account in self.accounts.values() {
            writeln!(f, "{}", account)?;
        }
        Ok(())
    }
}

impl Worker {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
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

            if let Err(err) = transaction.handle(&mut self.accounts, &mut self.transaction_log) {
                // The transaction has failed!
                eprintln!("{:?}, skipping...", err);
            }
        }

        Ok(())
    }
}
