use super::{into_processable, Processable};
use crate::{Account, LoggedTransaction, Transaction, TransactionLog};
use std::error::Error;

pub struct Deposit;
impl Processable for Deposit {
    fn process(
        &self,
        transaction: Transaction,
        account: &mut Account,
        log: &mut TransactionLog,
    ) -> Result<(), Box<dyn Error>> {
        if log.contains_key(&transaction.tx) {
            return Err("[deposit] Transaction already exists".into());
        }
        account.available += transaction
            .get_amount()
            .ok_or("[deposit] Transaction did not specify amount")?;
        log.insert(transaction.tx, LoggedTransaction::from(transaction));
        Ok(())
    }
}

into_processable!(Deposit);
