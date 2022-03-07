use crate::account::Account;

/// Deposit into an account
#[test]
fn deposit() {
    let mut account = Account::new(1);
    account.deposit(1.0);
    assert_eq!(account.available, 1.0);
    assert_eq!(account.held, 0.0);
}

/// Withdraw from an account
#[test]
fn withdraw() {
    let mut account = Account::new(1);
    account.deposit(1.0);
    // Normal
    assert!(account.withdraw(1.0).is_ok());
    assert_eq!(account.available, 0.0);
    assert_eq!(account.held, 0.0);
    // Insufficient funds
    assert!(account.withdraw(1.0).is_err());

    // Locked
    account.deposit(1.0);
    account.locked = true;
    assert!(account.withdraw(1.0).is_err());
}

/// A dispute on an account
#[test]
fn dispute() {
    let mut account = Account::new(1);
    account.deposit(1.0);
    assert!(account.dispute(1.0).is_ok());
    assert_eq!(account.held, 1.0);
    // Insufficient funds
    assert!(account.dispute(1.0).is_err());
    assert_eq!(account.held, 1.0);

    // Locked
    account.deposit(1.0);
    account.locked = true;
    assert!(account.dispute(1.0).is_err());
}

/// Resolving disputes
#[test]
fn resolve() {
    let mut account = Account::new(1);
    account.held = 1.0;
    assert!(account.resolve(1.0).is_ok());
    assert_eq!(account.available, 1.0);
    // Insufficient funds
    assert!(account.resolve(1.0).is_err());
    assert_eq!(account.available, 1.0);

    // Locked
    account.held = 1.0;
    account.locked = true;
    assert!(account.resolve(1.0).is_err());
}

/// Chargebacks
#[test]
fn chargeback() {
    let mut account = Account::new(1);
    account.held = 1.0;
    assert!(account.chargeback(1.0).is_ok());
    assert_eq!(account.held, 0.0);
    // Insufficient funds
    assert!(account.chargeback(1.0).is_err());
    assert_eq!(account.held, 0.0);

    // Locked
    account.held = 1.0;
    account.locked = true;
    assert!(account.chargeback(1.0).is_err());
}

/// Test the total calculation
#[test]
fn get_total() {
    let mut account = Account::new(1);
    account.deposit(2.0);
    assert!(account.dispute(1.0).is_ok());
    assert_eq!(account.get_total(), 2.0);
}
