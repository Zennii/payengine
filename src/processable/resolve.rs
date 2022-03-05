use super::{into_processable, Processable};
use crate::{Account, Transaction, TransactionLog};
use std::error::Error;

pub struct Resolve;
impl Processable for Resolve {
    fn process(
        &self,
        transaction: Transaction,
        account: &mut Account,
        log: &mut TransactionLog,
    ) -> Result<(), Box<dyn Error>> {
        let in_question = log
            .get_mut(&transaction.tx)
            .ok_or("[resolve] Invalid transaction reference")?;

        if !in_question.disputed {
            // We're already disputing this
            return Err("[resolve] Transaction not disputed".into());
        }

        let amount = in_question
            .amount
            .ok_or("[resolve] Original transaction did not specify amount")?;

        in_question.disputed = false;
        account.available += amount;
        account.held -= amount;
        Ok(())
    }
}

into_processable!(Resolve);
