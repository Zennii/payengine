use super::{into_processable, Processable};
use crate::{Account, Transaction, TransactionLog};
use std::error::Error;

pub struct Chargeback;
impl Processable for Chargeback {
    fn process(
        &self,
        transaction: Transaction,
        account: &mut Account,
        log: &mut TransactionLog,
    ) -> Result<(), Box<dyn Error>> {
        let in_question = log
            .get_mut(&transaction.tx)
            .ok_or("[chargeback] Invalid transaction reference")?;

        if !in_question.disputed {
            // We're already disputing this
            return Err("[chargeback] Transaction not disputed".into());
        }

        let amount = in_question
            .amount
            .ok_or("[chargeback] Original transaction did not specify amount")?;

        account.locked = true;
        account.held -= amount;
        Ok(())
    }
}

into_processable!(Chargeback);
