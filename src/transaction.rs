use crate::processable::*;
use crate::{Account, TransactionLog};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Clamps precision of an f32 to 4 decimal places.
fn clamp_precision(f: f32) -> f32 {
    (f * 10000.0).floor() * (1.0 / 10000.0)
}

/// A transaction contains a type, client, tx ID, and amount which could possibly not exist.
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub r#type: TransactionType,
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
        macro_rules! match_processable {
            ($($ident:ident),*) => {
                match self.r#type {
                    $(
                        TransactionType::$ident => $ident.into(),
                    )*
                }
            }
        }

        let processable: Box<dyn Processable> = match_processable!(
            Deposit, Withdrawal, Dispute, Resolve, Chargeback
        );

        processable.process(self, account, log)
    }

    pub fn get_amount(&self) -> Option<f32> {
        self.amount.map(clamp_precision)
    }
}

pub struct TransactionStatus {
    pub transaction: Transaction,
    pub disputed: bool,
}

impl TransactionStatus {
    pub fn new(transaction: Transaction) -> Self {
        Self {
            transaction,
            disputed: false,
        }
    }
}
