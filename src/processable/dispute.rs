use super::{into_processable, Processable};
use crate::account::{Account, Accounts};
use crate::transaction::{Transaction, TransactionLog};
use anyhow::{Context, Error, Result};

pub struct Dispute;
impl Processable for Dispute {
    fn process(
        &self,
        transaction: Transaction,
        accounts: &mut Accounts,
        log: &mut TransactionLog,
    ) -> Result<()> {
        let in_question = log.get_mut(&transaction.tx).context(format!(
            "[dispute] Invalid transaction reference {}",
            transaction.tx
        ))?;

        if in_question.client != transaction.client {
            return Err(Error::msg(format!(
                "[resolve] Client value {} did not match reference client {} for transaction {}",
                transaction.client, in_question.client, transaction.tx
            )));
        }

        if in_question.disputed {
            // We're already disputing this
            return Err(Error::msg(format!(
                "[dispute] Transaction {} already disputed",
                transaction.tx
            )));
        }

        let amount = in_question.amount.context(format!(
            "[dispute] Transaction {} did not specify amount",
            transaction.tx
        ))?;

        let account = accounts
            .entry(transaction.client)
            .or_insert_with(|| Account::new(transaction.client));

        in_question.disputed = true;
        account.available -= amount;
        account.held += amount;
        Ok(())
    }
}

into_processable!(Dispute);
