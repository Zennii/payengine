use super::{into_processable, Processable};
use crate::{Account, Transaction, TransactionLog};
use std::error::Error;

pub struct Dispute;
impl Processable for Dispute {
    fn process(
        &self,
        transaction: Transaction,
        account: &mut Account,
        log: &mut TransactionLog,
    ) -> Result<(), Box<dyn Error>> {
        let in_question = log
            .get_mut(&transaction.tx)
            .ok_or("[dispute] Invalid transaction reference")?;

        if in_question.disputed {
            // We're already disputing this
            return Err("[dispute] Transaction already disputed".into());
        }

        let amount = in_question
            .transaction
            .get_amount()
            .ok_or("[dispute] Original transaction did not specify amount")?;

        in_question.disputed = true;
        account.available -= amount;
        account.held += amount;
        Ok(())
    }
}

into_processable!(Dispute);
