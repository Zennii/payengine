use serde::{Deserialize, Serialize};

/// A transaction contains a type, client, tx ID, and
/// amount which could possibly not exist and will
/// default to None. This allows for a small variety
/// of formats to be accepted for deserialization:
/// ```
/// deposit, 1, 1, 1.0
/// DePosit, 1, 1,
/// DEPOSIT, 1, 1
/// ```
/// Transaction aims to be accepting of a variety
/// wide enough to allow for some runtime checks,
/// such as types being in any capitalization, and
/// missing amounts.
///
/// Amounts should be checked for existence
/// when necessary as there is no sanity checks
/// here for circumstances like if a deposit does
/// not have an amount, or a dispute does have one.
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub r#type: String,
    pub client: u16,
    pub tx: u32,
    // Handle missing field
    #[serde(default)]
    pub amount: Option<f32>,
}

/// A LoggedTransaction is essentially a stripped
/// down version of a Transaction to save on memory.
/// Its sole purpose is to be stored as an entry
/// into a log of transactions that are keyed by tx,
/// which is why the tx field is discarded.
///
/// Note that this type is lossy and can't be
/// transformed back into a Transaction without
/// recovering the tx from elsewhere.
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
