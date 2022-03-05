use crate::processable::*;
use crate::{Account, TransactionLog};
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Clamps precision of an f32 to 4 decimal places.
fn clamp_precision(f: f32) -> f32 {
    (f * 10000.0).floor() * (1.0 / 10000.0)
}

/// A transaction contains a type, client, tx ID, and amount which could possibly not exist.
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub r#type: String,
    pub client: u16,
    pub tx: u32,
    // Handle missing field
    #[serde(default)]
    amount: Option<f32>,
}

impl Transaction {
    /// Consumes itself to attempt to handle the transaction, returning an Err() if the transaction
    /// failed to process for some reason.
    pub fn handle(
        self,
        account: &mut Account,
        log: &mut TransactionLog,
    ) -> Result<(), Box<dyn Error>> {
        let processable: Box<dyn Processable> = match self.r#type.to_lowercase().as_str() {
            "deposit" => Deposit.into(),
            "withdrawal" => Withdrawal.into(),
            "dispute" => Dispute.into(),
            "resolve" => Resolve.into(),
            "chargeback" => Chargeback.into(),
            _ => return Err("a".into()),
        };

        processable.process(self, account, log)
    }

    pub fn get_amount(&self) -> Option<f32> {
        self.amount.map(clamp_precision)
    }
}

#[derive(Debug, Default)]
pub struct LoggedTransaction {
    pub client: u16,
    pub amount: Option<f32>,
    pub disputed: bool,
}

impl From<Transaction> for LoggedTransaction {
    fn from(transaction: Transaction) -> Self {
        Self {
            client: transaction.client,
            amount: transaction.get_amount(),
            disputed: false,
        }
    }
}
