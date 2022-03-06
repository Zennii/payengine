use anyhow::{Context, Result};
use bank::Bank;
use std::env::args;

mod account;
mod bank;
#[cfg(test)]
mod test;
mod transaction;

fn main() -> Result<()> {
    // Grab the file path from the 1st argument
    let transaction_file = args()
        .nth(1)
        .context("No file specified as first argument. Please specify a file.")?;

    // Create a bank to run the transactions through.
    let mut bank = Bank::new();
    bank.process_transactions(transaction_file)?;

    // Display the bank, printing a CSV format with header
    // of all available accounts after transactions.
    println!("{}", bank);

    Ok(())
}
