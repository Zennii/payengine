use super::{into_processable, Processable};
use crate::account::{Account, Accounts};
use crate::transaction::{Transaction, TransactionLog};
use anyhow::{Context, Error, Result};

pub struct Resolve;
impl Processable for Resolve {
    fn process(
        &self,
        transaction: Transaction,
        accounts: &mut Accounts,
        log: &mut TransactionLog,
    ) -> Result<()> {
        let in_question = log.get_mut(&transaction.tx).context(format!(
            "[resolve] Invalid transaction reference {}",
            transaction.tx
        ))?;

        if in_question.client != transaction.client {
            return Err(Error::msg(format!(
                "[resolve] Client value {} did not match reference client {} for transaction {}",
                transaction.client, in_question.client, transaction.tx
            )));
        }

        if !in_question.disputed {
            // We're already disputing this
            return Err(Error::msg(format!(
                "[resolve] Transaction {} not disputed",
                transaction.tx
            )));
        }

        let amount = in_question.amount.context(format!(
            "[resolve] Transaction {} did not specify amount",
            transaction.tx
        ))?;

        let account = accounts
            .entry(transaction.client)
            .or_insert_with(|| Account::new(transaction.client));

        in_question.disputed = false;
        account.available += amount;
        account.held -= amount;
        Ok(())
    }
}

into_processable!(Resolve);
