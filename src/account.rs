use anyhow::{Error, Result};
use std::fmt::{Display, Formatter};

#[derive(Default)]
pub struct Account {
    client_id: u16,
    pub available: f32,
    pub held: f32,
    pub locked: bool,
}

impl Account {
    pub fn new(client_id: u16) -> Self {
        Self {
            client_id,
            ..Default::default()
        }
    }

    pub fn deposit(&mut self, amount: f32) {
        self.available += amount;
    }

    pub fn withdraw(&mut self, amount: f32) -> Result<()> {
        if self.available < amount {
            return Err(Error::msg(format!(
                "Insufficient funds: has {} wants {}",
                self.available, amount
            )));
        }

        self.available -= amount;
        Ok(())
    }

    pub fn dispute(&mut self, amount: f32) {
        self.available -= amount;
        self.held += amount;
    }

    pub fn resolve(&mut self, amount: f32) {
        self.available += amount;
        self.held -= amount;
    }

    pub fn chargeback(&mut self, amount: f32) {
        self.locked = true;
        self.held -= amount;
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {:.4}, {:.4}, {:.4}, {}",
            self.client_id,
            self.available,
            self.held,
            self.available + self.held,
            self.locked,
        )
    }
}
