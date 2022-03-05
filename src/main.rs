mod processable;
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

use crate::transaction::TransactionStatus;
use csv::Trim;

pub type TransactionLog = HashMap<u32, TransactionStatus>;

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
            .from_reader(read_lines(transaction_file)?);

        for transaction_result in transactions.into_deserialize() {
            // todo optimize
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

    println!("{}", worker);
    /*
        for i in (0..40000).step_by(4) {
            println!(r#"deposit, 2, {}, 2.0
    deposit, 1, {}, 2.0
    withdrawal, {}, 4,1.56789
    withdrawal, {}, 5, 3.0"#, i, i+1, i+2, i+3)
        }
    */
    Ok(())
}
