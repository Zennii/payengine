use anyhow::{Context, Result};
use std::env::args;
use worker::Worker;

mod account;
mod processable;
#[cfg(test)]
mod test;
mod transaction;
mod worker;

fn main() -> Result<()> {
    let transaction_file = args()
        .nth(1)
        .context("No file specified as first argument. Please specify a file.")?;
    let mut worker = Worker::new();

    worker.process_transactions(transaction_file)?;

    println!("{}", worker);
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
    //println!("{}", worker.transaction_log.len() * 4 * 2 * 4 * 1);

    Ok(())
}
