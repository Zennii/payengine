use super::{into_processable, Processable};
use crate::account::{Account, Accounts};
use crate::transaction::{LoggedTransaction, Transaction, TransactionLog};
use anyhow::{Context, Error, Result};

pub struct Deposit;
impl Processable for Deposit {
    fn process(
        &self,
        transaction: Transaction,
        accounts: &mut Accounts,
        log: &mut TransactionLog,
    ) -> Result<()> {
        if log.contains_key(&transaction.tx) {
            return Err(Error::msg(format!(
                "[deposit] Transaction {} already exists",
                transaction.tx
            )));
        }

        let account = accounts
            .entry(transaction.client)
            .or_insert_with(|| Account::new(transaction.client));

        account.available += transaction.get_amount().context(format!(
            "[deposit] Transaction {} did not specify amount",
            transaction.tx
        ))?;

        log.insert(transaction.tx, LoggedTransaction::from(transaction));
        Ok(())
    }
}

into_processable!(Deposit);
