use super::{into_processable, Processable};
use crate::account::{Account, Accounts};
use crate::transaction::{LoggedTransaction, Transaction, TransactionLog};
use anyhow::{Context, Error, Result};

pub struct Withdrawal;
impl Processable for Withdrawal {
    fn process(
        &self,
        transaction: Transaction,
        accounts: &mut Accounts,
        log: &mut TransactionLog,
    ) -> Result<()> {
        if log.contains_key(&transaction.tx) {
            return Err(Error::msg(format!(
                "[withdrawal] Transaction {} already exists",
                transaction.tx
            )));
        }

        let amount = transaction.get_amount().context(format!(
            "[withdrawal] Transaction {} did not specify amount",
            transaction.tx
        ))?;

        let account = accounts
            .entry(transaction.client)
            .or_insert_with(|| Account::new(transaction.client));

        if account.available < amount {
            return Err(Error::msg(format!(
                "[withdrawal] Insufficient funds for transaction {}: has {} wants {}",
                transaction.tx, account.available, amount
            )));
        }

        account.available -= amount;

        log.insert(transaction.tx, LoggedTransaction::from(transaction));
        Ok(())
    }
}

into_processable!(Withdrawal);
