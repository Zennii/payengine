use crate::Bank;
use std::path::PathBuf;
use std::str::FromStr;

macro_rules! test_file {
    ($file:expr) => {
        // PathBuf from_str is infallible
        PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))
            .unwrap()
            .join("src/test/csv/")
            .join($file)
    };
}

/// Creates a new bank and processes transactions from a
/// file in the src/test/csv/ directory.
fn process(test_csv: &'static str) -> Bank {
    let mut bank = Bank::new();
    assert!(!bank.process_transactions(test_file!(test_csv)).is_err());
    bank
}

/// Test regular processing
#[test]
fn processing() {
    process("processing.csv");
}

/// A chargeback attempted with no dispute
#[test]
fn chargeback() {
    let bank = process("chargeback.csv");

    let account = bank.get_account(1).unwrap();
    let tx = bank.get_logged_transaction(1).unwrap();

    assert_ne!(account.available, 0.0);
    assert_eq!(account.held, 0.0);
    assert!(!account.locked);
    assert!(!tx.disputed);
}

/// A chargeback attempted with a dispute
#[test]
fn chargeback_dispute() {
    let bank = process("chargeback_dispute.csv");

    let account = bank.get_account(1).unwrap();
    let tx = bank.get_logged_transaction(1).unwrap();

    assert_eq!(account.available, 0.0);
    assert_eq!(account.held, 0.0);
    assert!(account.locked);
    assert!(tx.disputed);
}

/// A chargeback attempted with no valid transaction
#[test]
fn chargeback_no_tx() {
    let bank = process("chargeback_no_tx.csv");

    let account = bank.get_account(1).unwrap();
    let tx = bank.get_logged_transaction(1).unwrap();

    assert_eq!(account.available, 0.0);
    assert_eq!(account.held, 1.0);
    assert!(!account.locked);
    assert!(tx.disputed);
}

/// Test various decimal formats for sanity
#[test]
fn decimals() {
    let bank = process("decimals.csv");

    let account = bank.get_account(1).unwrap();
    let tx_1 = bank.get_logged_transaction(1).unwrap();
    let tx_2 = bank.get_logged_transaction(2).unwrap();

    assert!(account.available - 0.5575 < f32::EPSILON);
    assert_eq!(tx_1.amount, 0.5555);
    assert_eq!(tx_2.amount, 0.002);
}

/// A deposit into an account, this would create an account
/// and add funds
#[test]
fn deposit() {
    let bank = process("deposit.csv");

    let account = bank.get_account(1).unwrap();

    assert_eq!(account.available, 3.2345);
}

/// A dispute on a transaction
#[test]
fn dispute() {
    let bank = process("dispute.csv");

    let account = bank.get_account(1).unwrap();
    let tx = bank.get_logged_transaction(1).unwrap();

    assert_eq!(account.available, 0.0);
    assert_eq!(account.held, 1.0);
    assert!(tx.disputed);
}

/// A dispute attempted with no transaction
#[test]
fn dispute_no_tx() {
    let bank = process("dispute_no_tx.csv");

    let account = bank.get_account(1).unwrap();
    let tx = bank.get_logged_transaction(1).unwrap();

    assert_eq!(account.available, 1.0);
    assert_eq!(account.held, 0.0);
    assert!(!tx.disputed);
}

/// A duplicate transaction, where the IDs are the same
#[test]
fn duplicate_tx() {
    let bank = process("duplicate_tx.csv");

    let account = bank.get_account(1).unwrap();
    let tx_1 = bank.get_logged_transaction(1).unwrap();
    let tx_2 = bank.get_logged_transaction(2).unwrap();

    assert_eq!(account.available, 1.5);
    assert_eq!(tx_1.amount, 2.0);
    assert_eq!(tx_2.amount, 0.5);
}

/// Lines that fail to parse should do nothing
#[test]
fn failed_parse() {
    let bank = process("failed_parse.csv");

    assert_eq!(bank.num_accounts(), 0);
    assert_eq!(bank.num_logs(), 0);
}

/// Lines that don't have valid types should do nothing
#[test]
fn bad_tx() {
    let bank = process("bad_tx.csv");

    assert_eq!(bank.num_accounts(), 0);
    assert_eq!(bank.num_logs(), 0);
}

/// Types should allow any capitalization
#[test]
fn capitalization() {
    let bank = process("capitalization.csv");

    assert_eq!(bank.num_logs(), 3);
}

/// Client IDs can come in any order
#[test]
fn misordered_client() {
    let bank = process("misordered_client.csv");

    let account_1 = bank.get_account(1).unwrap();
    let account_2 = bank.get_account(2).unwrap();

    assert_eq!(account_1.available, 3.0);
    assert_eq!(account_2.available, 2.0);
}

/// Transaction IDs can come in any order
#[test]
fn misordered_tx() {
    let bank = process("misordered_tx.csv");

    let account_1 = bank.get_account(1).unwrap();
    let account_2 = bank.get_account(2).unwrap();

    assert_eq!(account_1.available, 2.0);
    assert_eq!(account_2.available, 3.0);
}

/// Amounts are optional and lines without are still valid
#[test]
fn optional_amount() {
    use crate::transaction::Transaction;
    use csv::Trim;
    use std::fs::File;

    let mut transactions = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_reader(
            File::options()
                .read(true)
                .open(test_file!("optional_amount.csv"))
                .expect("Missing test file"),
        );

    let mut count = 0;

    for transaction_result in transactions.deserialize() {
        let transaction: Transaction = match transaction_result {
            Ok(transaction) => transaction,
            Err(_) => panic!(),
        };

        assert!(transaction.amount.is_none());
        count += 1;
    }
    assert_eq!(count, 2);
}

/// A resolve attempted with no dispute
#[test]
fn resolve() {
    let bank = process("resolve.csv");

    let account = bank.get_account(1).unwrap();
    let tx = bank.get_logged_transaction(1).unwrap();

    assert_eq!(account.available, 2.0);
    assert_eq!(account.held, 0.0);
    assert!(!tx.disputed);
}

/// A resolve attempted with no transaction
#[test]
fn resolve_no_tx() {
    let bank = process("resolve_no_tx.csv");

    let account = bank.get_account(1).unwrap();
    let tx = bank.get_logged_transaction(1).unwrap();

    assert_eq!(account.available, 0.0);
    assert_eq!(account.held, 2.0);
    assert!(tx.disputed);
}

/// A resolve attempted with a dispute
#[test]
fn resolved_dispute() {
    let bank = process("resolved_dispute.csv");

    let account = bank.get_account(1).unwrap();
    let tx = bank.get_logged_transaction(1).unwrap();

    assert_eq!(account.available, 2.0);
    assert_eq!(account.held, 0.0);
    assert!(!tx.disputed);
}

/// Sample dataset
#[test]
fn sample() {
    let bank = process("sample.csv");

    let account_1 = bank.get_account(1).unwrap();
    let account_2 = bank.get_account(2).unwrap();

    assert_eq!(account_1.available, 1.5);
    assert_eq!(account_1.held, 0.0);
    assert!(!account_1.locked);
    assert_eq!(account_2.available, 2.0);
    assert_eq!(account_2.held, 0.0);
    assert!(!account_2.locked);
    assert_eq!(bank.num_logs(), 4)
}

/// A withdrawal that succeeds
#[test]
fn withdrawal() {
    let bank = process("withdrawal.csv");

    let account = bank.get_account(1).unwrap();

    assert!(account.available - 0.4322 < f32::EPSILON);
    assert_eq!(account.held, 0.0);
}

/// A withdrawal that fails due to insufficient funds
#[test]
fn withdrawal_insufficient() {
    let bank = process("withdrawal_insufficient.csv");

    let account = bank.get_account(1).unwrap();

    assert_eq!(account.available, 1.0);
    assert_eq!(account.held, 0.0);
}

/// Total value calculation from an account
#[test]
fn total() {
    let bank = process("total.csv");

    let account = bank.get_account(1).unwrap();

    assert!(account.get_total() - 2.6666 < f32::EPSILON);
}

/// A locked account attempting to be used
#[test]
fn locked() {
    let bank = process("locked.csv");

    let account = bank.get_account(1).unwrap();
    let tx = bank.get_logged_transaction(2).unwrap();

    assert!(account.locked);
    assert_eq!(account.available, 1.5);
    assert_eq!(account.held, 0.0);
    assert!(!tx.disputed);
}

/// Displaying the bank info
#[test]
fn output() {
    let bank = process("deposit.csv");
    let expected = "client, available, held, total, locked\n1, 3.2345, 0.0000, 3.2345, false\n";
    assert_eq!(format!("{}", bank), expected);
}
