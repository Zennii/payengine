use anyhow::{Context, Result};
use bank::Bank;
use std::env::args;

mod account;
mod bank;
#[cfg(test)]
mod test;
mod transaction;

fn main() -> Result<()> {
    let transaction_file = args()
        .nth(1)
        .context("No file specified as first argument. Please specify a file.")?;

    let mut bank = Bank::new();
    bank.process_transactions(transaction_file)?;

    println!("{}", bank);

    Ok(())
}
