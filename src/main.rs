mod processable;
#[cfg(test)]
mod test;
mod transaction;

use std::collections::HashMap;
use transaction::Transaction;

use std::env::args;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;

use crate::transaction::LoggedTransaction;
use csv::Trim;

pub type TransactionLog = HashMap<u32, LoggedTransaction>;

#[derive(Default)]
pub struct Account {
    client_id: u16,
    available: f32,
    held: f32,
    locked: bool,
}

impl Account {
    pub fn new(client_id: u16) -> Self {
        Self {
            client_id,
            ..Default::default()
        }
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {:.4}, {:.4}, {:.4}, {}",
            self.client_id,
            self.available,
            self.held,
            self.available + self.held,
            self.locked,
        )
    }
}

#[derive(Default)]
struct Worker {
    accounts: HashMap<u16, Account>,
    transaction_log: TransactionLog,
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

    pub fn process_transactions(&mut self, transaction_file: String) -> Result<(), Box<dyn Error>> {
        let transactions = csv::ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(true)
            .from_reader(File::options().read(true).open(transaction_file)?);

        // Read the data in chunks (BufReader under the hood)
        for transaction_result in transactions.into_deserialize() {
            let transaction: Transaction = transaction_result?;
            let account = self
                .accounts
                .entry(transaction.client)
                .or_insert_with(|| Account::new(transaction.client));
            // Ignore transactions that failed,
            // we are guaranteed nothing happened to the account
            transaction.handle(account, &mut self.transaction_log).ok();
        }

        Ok(())
    }
}

fn read_lines<P>(path: P) -> io::Result<BufReader<File>>
where
    P: AsRef<Path>,
{
    let file = File::options().read(true).open(path)?;
    Ok(BufReader::new(file))
}

fn main() -> Result<(), Box<dyn Error>> {
    let transaction_file = args()
        .nth(1)
        .ok_or("No file specified as first argument. Please specify a file.")?;
    let mut worker = Worker::new();

    worker.process_transactions(transaction_file)?;

    //println!("{}", worker);
    /*println!("type, client, tx, amount");
    for i in (0..u32::MAX).step_by(7) {
        println!("deposit, 2, {}, 2.0\n\
        deposit, 1, {}, 2.0\n\
        deposit, 2, {}, 3.0\n\
        withdrawal, 1, {},1.56789\n\
        withdrawal, 2, {}, 3.0\n\
        deposit, {}, {}, 0.538724\n\
        dispute, {}, {},", i, i+1, i+2, i+3, i+4, i % u16::MAX as u32, i+6, i % u16::MAX as u32, i+6)
    }*/
    println!("{}", worker.transaction_log.len() * 4 * 2 * 4 * 1);

    Ok(())
}
