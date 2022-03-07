use anyhow::{Error, Result};
use std::fmt::{Display, Formatter};

const LOCKED_ERROR: &'static str = "Account is locked";

/// An account holds funds and is represented by a
/// unique ID. Funds can be available, which means
/// the user has immediate access to these funds.
/// Funds can also be held, which means that they
/// are under some sort of dispute. Accounts can
/// be locked by a chargeback, meaning no new
/// transactions will succeed.
#[derive(Default)]
pub struct Account {
    client_id: u16,
    pub available: f32,
    pub held: f32,
    pub locked: bool,
}

impl Account {
    /// Create a new account with a client ID.
    pub fn new(client_id: u16) -> Self {
        Self {
            client_id,
            ..Default::default()
        }
    }

    /// Attempt to deposit funds into the available funds.
    pub fn deposit(&mut self, amount: f32) {
        self.available += amount;
    }

    /// Attempts to withdraw funds from the available funds.
    ///
    /// Returns an Err if there are not enough available
    /// funds or the account is locked.
    pub fn withdraw(&mut self, amount: f32) -> Result<()> {
        if self.locked {
            return Err(Error::msg(LOCKED_ERROR));
        }
        if self.available < amount {
            return Err(Error::msg(format!(
                "Insufficient funds: has {} wants {}",
                self.available, amount
            )));
        }

        self.available -= amount;
        Ok(())
    }

    /// Attempts to mark funds as disputed, moving the
    /// funds from available -> hold.
    ///
    /// Returns an Err if there are not enough available
    /// funds or the account is locked.
    pub fn dispute(&mut self, amount: f32) -> Result<()> {
        if self.locked {
            return Err(Error::msg(LOCKED_ERROR));
        }
        if self.available < amount {
            return Err(Error::msg(format!(
                "Insufficient funds: has {} wants {}",
                self.available, amount
            )));
        }

        self.available -= amount;
        self.held += amount;
        Ok(())
    }

    /// Attempts to mark funds as resolved, moving the
    /// funds from hold -> available.
    ///
    /// Returns an Err if there are not enough held
    /// funds or the account is locked.
    pub fn resolve(&mut self, amount: f32) -> Result<()> {
        if self.locked {
            return Err(Error::msg(LOCKED_ERROR));
        }
        if self.held < amount {
            return Err(Error::msg(format!(
                "Insufficient funds: has {} wants {}",
                self.held, amount
            )));
        }

        self.available += amount;
        self.held -= amount;
        Ok(())
    }

    /// Attempts to chargeback funds, removing held funds
    /// from the account.
    ///
    /// Returns an Err if there are not enough held
    /// funds or the account is locked.
    pub fn chargeback(&mut self, amount: f32) -> Result<()> {
        if self.locked {
            return Err(Error::msg(LOCKED_ERROR));
        }
        if self.held < amount {
            return Err(Error::msg(format!(
                "Insufficient funds: has {} wants {}",
                self.held, amount
            )));
        }

        self.locked = true;
        self.held -= amount;
        Ok(())
    }

    /// Calculates the total balance of the account.
    pub fn get_total(&self) -> f32 {
        self.available + self.held
    }
}

impl Display for Account {
    /// Displays the account is a CSV format.
    /// ```
    /// 2, 2.0000, 0.1234, 2.1234, false
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {:.4}, {:.4}, {:.4}, {}",
            self.client_id,
            self.available,
            self.held,
            self.get_total(),
            self.locked,
        )
    }
}
