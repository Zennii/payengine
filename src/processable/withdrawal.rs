use super::{into_processable, Processable};
use crate::{Account, LoggedTransaction, Transaction, TransactionLog};
use std::error::Error;

pub struct Withdrawal;
impl Processable for Withdrawal {
    fn process(
        &self,
        transaction: Transaction,
        account: &mut Account,
        log: &mut TransactionLog,
    ) -> Result<(), Box<dyn Error>> {
        if log.contains_key(&transaction.tx) {
            return Err("[withdrawal] Transaction already exists".into());
        }

        let amount = transaction
            .get_amount()
            .ok_or("[withdrawal] Transaction did not specify amount")?;

        if account.available < amount {
            return Err("[withdrawal] Insufficient funds".into());
        }

        account.available -= amount;

        log.insert(transaction.tx, LoggedTransaction::from(transaction));
        Ok(())
    }
}

into_processable!(Withdrawal);
