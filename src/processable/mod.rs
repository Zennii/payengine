mod chargeback;
mod deposit;
mod dispute;
mod resolve;
mod withdrawal;

pub use chargeback::Chargeback;
pub use deposit::Deposit;
pub use dispute::Dispute;
pub use resolve::Resolve;
pub use withdrawal::Withdrawal;

use crate::account::Accounts;
use crate::transaction::{Transaction, TransactionLog};

use anyhow::Result;

pub trait Processable {
    fn process(
        &self,
        transaction: Transaction,
        accounts: &mut Accounts,
        log: &mut TransactionLog,
    ) -> Result<()>;
}

macro_rules! into_processable {
    ($ident:ident) => {
        impl From<$ident> for Box<dyn Processable> {
            fn from(p: $ident) -> Self {
                Box::new(p) as Box<dyn Processable>
            }
        }
    };
}
pub(crate) use into_processable;
