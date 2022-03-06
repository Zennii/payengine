use crate::Worker;
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

fn process_worker(test_csv: &'static str) -> Worker {
    let mut worker = Worker::new();
    assert!(!worker.process_transactions(test_file!(test_csv)).is_err());
    worker
}

#[test]
fn chargeback() {
    let worker = process_worker("chargeback.csv");

    let account = worker.accounts.get(&1).unwrap();
    let tx = worker.transaction_log.get(&1).unwrap();

    assert_ne!(account.available, 0.0);
    assert_eq!(account.held, 0.0);
    assert!(!account.locked);
    assert!(!tx.disputed);
}

#[test]
fn chargeback_dispute() {
    let worker = process_worker("chargeback_dispute.csv");

    let account = worker.accounts.get(&1).unwrap();
    let tx = worker.transaction_log.get(&1).unwrap();

    assert_eq!(account.available, 0.0);
    assert_eq!(account.held, 0.0);
    assert!(account.locked);
    assert!(tx.disputed);
}

#[test]
fn chargeback_no_tx() {
    let worker = process_worker("chargeback_no_tx.csv");

    let account = worker.accounts.get(&1).unwrap();
    let tx = worker.transaction_log.get(&1).unwrap();

    assert_eq!(account.available, 0.0);
    assert_eq!(account.held, 1.0);
    assert!(!account.locked);
    assert!(tx.disputed);
}

#[test]
fn decimals() {
    let worker = process_worker("decimals.csv");

    let account = worker.accounts.get(&1).unwrap();
    let tx_1 = worker.transaction_log.get(&1).unwrap();
    let tx_2 = worker.transaction_log.get(&2).unwrap();

    assert_eq!(account.available, 0.5555);
    assert_eq!(tx_1.amount, Some(0.5555));
    assert_eq!(tx_2.amount, Some(0.0));
}

#[test]
fn deposit() {
    let worker = process_worker("deposit.csv");

    let account = worker.accounts.get(&1).unwrap();

    assert_eq!(account.available, 3.2345);
}

#[test]
fn dispute() {
    let worker = process_worker("dispute.csv");

    let account = worker.accounts.get(&1).unwrap();
    let tx = worker.transaction_log.get(&1).unwrap();

    assert_eq!(account.available, 0.0);
    assert_eq!(account.held, 1.0);
    assert!(tx.disputed);
}

#[test]
fn dispute_no_tx() {
    let worker = process_worker("dispute_no_tx.csv");

    let account = worker.accounts.get(&1).unwrap();
    let tx = worker.transaction_log.get(&1).unwrap();

    assert_eq!(account.available, 1.0);
    assert_eq!(account.held, 0.0);
    assert!(!tx.disputed);
}

#[test]
fn duplicate_tx() {
    let worker = process_worker("duplicate_tx.csv");

    let account = worker.accounts.get(&1).unwrap();
    let tx_1 = worker.transaction_log.get(&1).unwrap();
    let tx_2 = worker.transaction_log.get(&2).unwrap();

    assert_eq!(account.available, 1.5);
    assert_eq!(tx_1.amount, Some(2.0));
    assert_eq!(tx_2.amount, Some(0.5));
}

#[test]
fn failed_parse() {
    let worker = process_worker("failed_parse.csv");

    assert_eq!(worker.accounts.len(), 0);
    assert_eq!(worker.transaction_log.len(), 0);
}

#[test]
fn misordered_client() {
    let worker = process_worker("misordered_client.csv");

    let account_1 = worker.accounts.get(&1).unwrap();
    let account_2 = worker.accounts.get(&2).unwrap();

    assert_eq!(account_1.available, 3.0);
    assert_eq!(account_2.available, 2.0);
}

#[test]
fn misordered_tx() {
    let worker = process_worker("misordered_tx.csv");

    let account_1 = worker.accounts.get(&1).unwrap();
    let account_2 = worker.accounts.get(&2).unwrap();

    assert_eq!(account_1.available, 2.0);
    assert_eq!(account_2.available, 3.0);
}

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

        assert!(transaction.get_amount().is_none());
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn resolve() {
    let worker = process_worker("resolve.csv");

    let account = worker.accounts.get(&1).unwrap();
    let tx = worker.transaction_log.get(&1).unwrap();

    assert_eq!(account.available, 2.0);
    assert_eq!(account.held, 0.0);
    assert!(!tx.disputed);
}

#[test]
fn resolve_no_tx() {
    let worker = process_worker("resolve_no_tx.csv");

    let account = worker.accounts.get(&1).unwrap();
    let tx = worker.transaction_log.get(&1).unwrap();

    assert_eq!(account.available, 0.0);
    assert_eq!(account.held, 2.0);
    assert!(tx.disputed);
}

#[test]
fn resolved_dispute() {
    let worker = process_worker("resolved_dispute.csv");

    let account = worker.accounts.get(&1).unwrap();
    let tx = worker.transaction_log.get(&1).unwrap();

    assert_eq!(account.available, 2.0);
    assert_eq!(account.held, 0.0);
    assert!(!tx.disputed);
}

#[test]
fn sample() {
    let worker = process_worker("sample.csv");

    let account_1 = worker.accounts.get(&1).unwrap();
    let account_2 = worker.accounts.get(&2).unwrap();

    assert_eq!(account_1.available, 1.5);
    assert_eq!(account_1.held, 0.0);
    assert!(!account_1.locked);
    assert_eq!(account_2.available, 2.0);
    assert_eq!(account_2.held, 0.0);
    assert!(!account_2.locked);
    assert_eq!(worker.transaction_log.len(), 4)
}

#[test]
fn withdrawal() {
    let worker = process_worker("withdrawal.csv");

    let account = worker.accounts.get(&1).unwrap();

    assert!(account.available - 0.4322 < f32::EPSILON);
    assert_eq!(account.held, 0.0);
}

#[test]
fn withdrawal_insufficient() {
    let worker = process_worker("withdrawal_insufficient.csv");

    let account = worker.accounts.get(&1).unwrap();

    assert_eq!(account.available, 1.0);
    assert_eq!(account.held, 0.0);
}