use serde::{Deserialize, Serialize};

/// A transaction contains a type, client, tx ID, and amount which could possibly not exist.
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub r#type: String,
    pub client: u16,
    pub tx: u32,
    // Handle missing field
    #[serde(default)]
    pub amount: Option<f32>,
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
            amount: transaction.amount,
            disputed: false,
        }
    }
}
